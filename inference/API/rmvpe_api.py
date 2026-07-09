from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import librosa
import numpy as np

from inference.device_utils import resolve_onnx_providers

try:
    import onnxruntime as ort
except ImportError:  # pragma: no cover - handled at runtime
    ort = None


SAMPLE_RATE = 16000
HOP_LENGTH = 160


@dataclass
class RmvpeResult:
    time_step_seconds: float
    midi_pitch: np.ndarray
    voiced_mask: np.ndarray | None = None


class RmvpeTranscriber:
    def __init__(self, model_path: str | Path, device: str = "dml", batch_size: int = 8, threshold: float = 0.03):
        self.model_path = Path(model_path)
        self.requested_device = str(device)
        self.batch_size = max(1, int(batch_size))
        self.threshold = float(threshold)

        if not self.model_path.exists():
            raise FileNotFoundError(f"RMVPE model not found: {self.model_path}")
        if ort is None:
            raise RuntimeError("onnxruntime is required for RMVPE ONNX inference.")

        self.provider_name, providers = self._resolve_providers(self.requested_device)
        self.session = self._create_session(providers)

        self.waveform_input_name = self._resolve_input_name("waveform", 0)
        self.threshold_input_name = self._resolve_input_name("threshold", 1)
        self.f0_output_name = self._resolve_output_name("f0", 0)
        self.uv_output_name = self._resolve_output_name("uv", 1)

    @staticmethod
    def _resolve_providers(device: str) -> tuple[str, list[str]]:
        return resolve_onnx_providers(device, label="RMVPE ONNX")

    def _create_session(self, providers: list[str]):
        sess_options = ort.SessionOptions()
        sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        sess_options.enable_mem_pattern = False
        sess_options.enable_cpu_mem_arena = False
        return ort.InferenceSession(str(self.model_path), sess_options=sess_options, providers=providers)

    def _resolve_input_name(self, expected: str, fallback_index: int) -> str:
        inputs = self.session.get_inputs()
        for inp in inputs:
            if inp.name.lower() == expected.lower():
                return inp.name
        if 0 <= fallback_index < len(inputs):
            return inputs[fallback_index].name
        raise RuntimeError(f"RMVPE model missing '{expected}' input")

    def _resolve_output_name(self, expected: str, fallback_index: int) -> str:
        outputs = self.session.get_outputs()
        for out in outputs:
            if out.name.lower() == expected.lower():
                return out.name
        if 0 <= fallback_index < len(outputs):
            return outputs[fallback_index].name
        raise RuntimeError(f"RMVPE model missing '{expected}' output")

    def infer(self, waveform: np.ndarray, sample_rate: int, cancel_checker=None) -> RmvpeResult:
        waveform = np.asarray(waveform, dtype=np.float32)
        if waveform.ndim > 1:
            waveform = np.mean(waveform, axis=-1)
        if waveform.size == 0:
            empty = np.zeros((0,), dtype=np.float32)
            return RmvpeResult(
                time_step_seconds=HOP_LENGTH / SAMPLE_RATE,
                midi_pitch=empty,
                voiced_mask=empty.astype(bool),
            )

        if sample_rate != SAMPLE_RATE:
            waveform = librosa.resample(waveform, orig_sr=sample_rate, target_sr=SAMPLE_RATE)
        waveform = np.clip(np.asarray(waveform, dtype=np.float32), -1.0, 1.0)

        if cancel_checker and cancel_checker():
            raise InterruptedError("RMVPE task cancelled")

        f0_hz, uv = self._inference_f0(waveform)
        voiced_mask = (~uv) & (f0_hz > 0)
        midi_pitch = self._f0_to_interpolated_midi(f0_hz, voiced_mask)
        return RmvpeResult(
            time_step_seconds=HOP_LENGTH / SAMPLE_RATE,
            midi_pitch=midi_pitch,
            voiced_mask=voiced_mask,
        )

    def _inference_f0(self, audio: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
        waveform_batch = audio[np.newaxis, :]
        threshold_scalar = np.array(self.threshold, dtype=np.float32)

        outputs = self.session.run(
            None,
            {
                self.waveform_input_name: waveform_batch,
                self.threshold_input_name: threshold_scalar,
            },
        )

        output_map = {out.name: outputs[i] for i, out in enumerate(self.session.get_outputs())}
        f0 = np.asarray(output_map[self.f0_output_name], dtype=np.float32)[0]
        uv = np.asarray(output_map[self.uv_output_name], dtype=bool)[0]
        return f0, uv

    @staticmethod
    def _f0_to_interpolated_midi(f0: np.ndarray, voiced: np.ndarray) -> np.ndarray:
        midi = np.full_like(f0, np.nan, dtype=np.float32)
        midi[voiced] = 69.0 + 12.0 * np.log2(f0[voiced] / 440.0)

        voiced_indices = np.where(~np.isnan(midi))[0]
        if len(voiced_indices) == 0:
            return midi

        first = voiced_indices[0]
        midi[:first] = midi[first]

        prev = first
        idx = first + 1
        n = len(midi)
        while idx < n:
            if not np.isnan(midi[idx]):
                prev = idx
                idx += 1
                continue

            gap_start = idx
            while idx < n and np.isnan(midi[idx]):
                idx += 1

            if idx < n:
                left = midi[prev]
                right = midi[idx]
                gap_len = idx - prev
                for i in range(1, gap_len):
                    ratio = i / gap_len
                    midi[prev + i] = left + (right - left) * ratio
                prev = idx
            else:
                midi[gap_start:] = midi[prev]

        return midi

    def shutdown(self) -> None:
        self.session = None

    release = shutdown
