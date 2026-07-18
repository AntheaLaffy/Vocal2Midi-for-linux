from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT))

import inference.romaji_asr.common as common


FIXTURE_PATH = Path("rewrite-in-rust/fixtures/asr_romaji_batch_metadata_contract.jsonl")


class FakeMeta:
    def __init__(self, name: str, type: str, shape: list):
        self.name = name
        self.type = type
        self.shape = shape


class FakeSession:
    def __init__(self, inputs: list[dict]):
        self._inputs = [FakeMeta(**item) for item in inputs]

    def get_inputs(self):
        return self._inputs


def encode_array(array: np.ndarray) -> dict:
    return {
        "dtype": str(array.dtype),
        "shape": list(array.shape),
        "values": array.tolist(),
    }


def result_for(case: dict) -> dict:
    session = FakeSession(case.get("inputs", []))
    try:
        match case["call"]:
            case "metadata":
                return {
                    "ok": True,
                    "fixed_batch_size": common.get_fixed_batch_size(session),
                    "fixed_num_samples": common.get_fixed_num_samples(session),
                }
            case "ort_type_to_numpy_dtype":
                return {
                    "ok": True,
                    "dtype": np.dtype(common.ort_type_to_numpy_dtype(case["ort_type"])).name,
                }
            case "prepare_batch":
                calls: list[dict] = []
                old_load_audio = common.load_audio

                def fake_load_audio(path, sample_rate=common.DEFAULT_SAMPLE_RATE):
                    calls.append({"path": str(path), "sample_rate": sample_rate})
                    return np.asarray(case["waveforms"][str(path)], dtype=np.float32)

                common.load_audio = fake_load_audio
                try:
                    feeds, used_lengths = common.prepare_batch(
                        session,
                        case["audio_paths"],
                        sample_rate=case.get("sample_rate", common.DEFAULT_SAMPLE_RATE),
                    )
                finally:
                    common.load_audio = old_load_audio
                return {
                    "ok": True,
                    "feeds": {name: encode_array(value) for name, value in feeds.items()},
                    "used_lengths": used_lengths,
                    "load_audio_calls": calls,
                }
            case other:
                raise ValueError(f"unknown fixture call: {other}")
    except Exception as exc:  # noqa: BLE001 - fixture captures legacy exception projection.
        result = {"ok": False, "error_type": type(exc).__name__, "message": str(exc)}
        if case["call"] == "prepare_batch":
            result["load_audio_calls"] = locals().get("calls", [])
        return result


def main() -> None:
    failures: list[str] = []
    count = 0
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf-8").splitlines(), 1):
        if not line:
            continue
        count += 1
        case = json.loads(line)
        actual = result_for(case)
        expected = case["expected"]
        if actual != expected:
            failures.append(f"{line_number} {case['category']}: actual={actual!r}, expected={expected!r}")

    if failures:
        raise AssertionError("\n".join(failures))

    print(f"asr_romaji_batch_metadata_contract fixtures ok: {count} cases")


if __name__ == "__main__":
    main()
