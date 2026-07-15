"""Opt-in JSON bridge proof for Rust quantization.

The production quantization owner remains ``inference.quant.quantization``.
This module is selected only by explicit backend choice while the Rust rewrite
validates payload, error, and rollback behavior.
"""

from __future__ import annotations

import json
import math
import os
import pathlib
import subprocess
import time
from typing import Any

from inference.quant.quantization import quantize_notes as legacy_quantize_notes


class QuantizationBridgeError(RuntimeError):
    """Raised when the opt-in Rust JSON quantization bridge fails."""


def quantize_notes_with_backend(
    notes: list[Any],
    tempo: float,
    quantization_step: int,
    mode: str = "simple",
    *,
    backend: str | None = None,
    executable: str | os.PathLike[str] | None = None,
    timeout_sec: float = 30.0,
    cancel_checker=None,
) -> None:
    """Mutate notes through the selected quantization backend.

    ``legacy`` remains the default. ``rust-json`` is a subprocess proof that
    returns timing mutations keyed by original Python note indexes.
    """

    selected_backend = backend or os.environ.get("V2M_QUANT_BACKEND", "legacy")
    if selected_backend == "legacy":
        _check_cancel(cancel_checker)
        legacy_quantize_notes(notes, tempo, quantization_step, mode=mode)
        _check_cancel(cancel_checker)
        return None

    if selected_backend != "rust-json":
        raise QuantizationBridgeError(f"unsupported quantization backend: {selected_backend}")

    _check_cancel(cancel_checker)
    bridge_path = _resolve_bridge_executable(executable)
    payload = _build_payload(notes, tempo, quantization_step, mode)
    response = _run_bridge(bridge_path, payload, timeout_sec, cancel_checker=cancel_checker)
    _apply_response(notes, response)
    _check_cancel(cancel_checker)
    return None


def _resolve_bridge_executable(
    executable: str | os.PathLike[str] | None,
) -> pathlib.Path:
    if executable is not None:
        return pathlib.Path(executable)

    env_path = os.environ.get("V2M_QUANT_BRIDGE_BIN")
    if env_path:
        return pathlib.Path(env_path)

    raise QuantizationBridgeError(
        "rust-json backend requires V2M_QUANT_BRIDGE_BIN or executable="
    )


def _build_payload(
    notes: list[Any],
    tempo: float,
    quantization_step: int,
    mode: str,
) -> dict[str, Any]:
    tempo_value = _coerce_tempo(tempo)
    step_value = _coerce_step(quantization_step)
    return {
        "version": 1,
        "mode": mode,
        "tempo": tempo_value,
        "quantization_step": step_value,
        "notes": [
            {
                "index": index,
                "onset": _coerce_note_float(index, "onset", note.onset),
                "offset": _coerce_note_float(index, "offset", note.offset),
                "pitch": _coerce_optional_pitch(note),
                "lyric": getattr(note, "lyric", "") or "",
            }
            for index, note in enumerate(notes)
        ],
    }


def _run_bridge(
    executable: pathlib.Path,
    payload: dict[str, Any],
    timeout_sec: float,
    *,
    cancel_checker=None,
) -> dict[str, Any]:
    payload_json = json.dumps(payload, allow_nan=False)
    timeout_value = _coerce_timeout(timeout_sec)

    try:
        process = subprocess.Popen(
            [str(executable)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except FileNotFoundError as exc:
        raise QuantizationBridgeError(f"bridge executable not found: {executable}") from exc
    except OSError as exc:
        raise QuantizationBridgeError(
            f"bridge process failed to start: {executable}: {exc}"
        ) from exc

    deadline = time.monotonic() + timeout_value
    communicate_input: str | None = payload_json
    while True:
        if cancel_checker and cancel_checker():
            _kill_process(process)
            raise InterruptedError("任务已取消")

        remaining = deadline - time.monotonic()
        if remaining <= 0:
            _kill_process(process)
            raise QuantizationBridgeError(f"bridge timed out after {timeout_value}s")

        try:
            stdout, stderr = process.communicate(
                input=communicate_input,
                timeout=min(0.05, remaining),
            )
            break
        except subprocess.TimeoutExpired:
            communicate_input = None

    try:
        response = json.loads(stdout)
    except json.JSONDecodeError as exc:
        stderr_text = stderr.strip()
        detail = f"; stderr={stderr_text}" if stderr_text else ""
        raise QuantizationBridgeError(
            f"bridge returned non-JSON stdout with code {process.returncode}{detail}"
        ) from exc

    if process.returncode != 0 or not response.get("ok"):
        error = response.get("error") if isinstance(response, dict) else None
        code = error.get("code", "bridge_error") if isinstance(error, dict) else "bridge_error"
        message = error.get("message", "") if isinstance(error, dict) else ""
        raise QuantizationBridgeError(f"{code}: {message}".rstrip())

    return response


def _check_cancel(cancel_checker) -> None:
    if cancel_checker and cancel_checker():
        raise InterruptedError("任务已取消")


def _kill_process(process: subprocess.Popen[str]) -> None:
    try:
        process.kill()
    finally:
        try:
            process.wait(timeout=1.0)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        if process.stdout is not None:
            process.stdout.close()
        if process.stderr is not None:
            process.stderr.close()


def _coerce_tempo(value: float) -> float:
    try:
        tempo = float(value)
    except (TypeError, ValueError) as exc:
        raise QuantizationBridgeError("invalid_tempo: tempo must be numeric") from exc
    if not math.isfinite(tempo) or tempo <= 0.0:
        raise QuantizationBridgeError("invalid_tempo: tempo must be a positive finite number")
    return tempo


def _coerce_step(value: int) -> int:
    try:
        return int(value)
    except (TypeError, ValueError) as exc:
        raise QuantizationBridgeError(
            "invalid_quantization_step: quantization_step must be an integer"
        ) from exc


def _coerce_timeout(value: float) -> float:
    try:
        timeout = float(value)
    except (TypeError, ValueError) as exc:
        raise QuantizationBridgeError("invalid_timeout: timeout_sec must be numeric") from exc
    if not math.isfinite(timeout) or timeout <= 0.0:
        raise QuantizationBridgeError("invalid_timeout: timeout_sec must be a positive finite number")
    return timeout


def _coerce_note_float(index: int, field: str, value: Any) -> float:
    try:
        number = float(value)
    except (TypeError, ValueError) as exc:
        raise QuantizationBridgeError(
            f"invalid_note: note {index} {field} must be numeric"
        ) from exc
    if not math.isfinite(number):
        raise QuantizationBridgeError(
            f"invalid_note: note {index} {field} must be finite"
        )
    return number


def _coerce_optional_pitch(note: Any) -> float:
    if not hasattr(note, "pitch") or getattr(note, "pitch") is None:
        return 0.0
    try:
        pitch = float(note.pitch)
    except (TypeError, ValueError):
        return 0.0
    if not math.isfinite(pitch):
        return 0.0
    return pitch


def _apply_response(notes: list[Any], response: dict[str, Any]) -> None:
    response_notes = response.get("notes")
    if not isinstance(response_notes, list):
        raise QuantizationBridgeError("bridge response missing notes list")

    original_notes = list(notes)
    by_index = {index: note for index, note in enumerate(original_notes)}
    reordered = []

    for item in response_notes:
        if not isinstance(item, dict):
            raise QuantizationBridgeError("bridge note mutation must be an object")
        index = item.get("index")
        if index not in by_index:
            raise QuantizationBridgeError(f"bridge returned unknown note index: {index!r}")
        note = by_index[index]
        note.onset = float(item["onset"])
        note.offset = float(item["offset"])
        reordered.append(note)

    if len(reordered) != len(original_notes):
        raise QuantizationBridgeError("bridge response note count mismatch")

    notes[:] = reordered
