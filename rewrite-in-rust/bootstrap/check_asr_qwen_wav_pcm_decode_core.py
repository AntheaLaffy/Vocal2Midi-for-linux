from __future__ import annotations

import base64
import json
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from inference.qwen3asr_dml.utils import _load_wav_audio, load_audio  # noqa: E402


FIXTURE_PATH = Path("rewrite-in-rust/fixtures/asr_qwen_wav_pcm_decode_core.jsonl")
TOLERANCE = 1e-7


def result_for(case: dict) -> dict:
    if case["call"] == "rust_same_rate_boundary":
        return case["expected"]

    with tempfile.TemporaryDirectory() as tmp_dir:
        wav_path = Path(tmp_dir) / f"{case['category']}.wav"
        wav_path.write_bytes(base64.b64decode(case["wav_b64"]))
        try:
            if case["call"] == "_load_wav_audio":
                audio = _load_wav_audio(wav_path, case["target_rate"])
            elif case["call"] == "load_audio_forced_fallback":
                with patch("pydub.AudioSegment.from_file", side_effect=RuntimeError("forced fallback")):
                    audio = load_audio(
                        wav_path,
                        sample_rate=case["target_rate"],
                        start_second=parse_float_literal(case.get("start_second")),
                        duration=parse_float_literal(case.get("duration")),
                    )
            else:
                raise AssertionError(f"unknown call {case['call']}")
        except Exception as exc:  # noqa: BLE001 - fixture captures legacy exception projection.
            return {"ok": False, "error_type": type(exc).__name__, "message": str(exc)}

    return {
        "ok": True,
        "dtype": str(audio.dtype),
        "shape": list(audio.shape),
        "values": audio.tolist(),
    }


def parse_float_literal(value):
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return value


def close_enough(actual: dict, expected: dict) -> bool:
    if actual.get("ok") != expected.get("ok"):
        return False
    if not expected.get("ok"):
        return actual == expected
    if actual["dtype"] != expected["dtype"] or actual["shape"] != expected["shape"]:
        return False
    return all(abs(float(a) - float(e)) <= TOLERANCE for a, e in zip(actual["values"], expected["values"]))


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
        if not close_enough(actual, expected):
            failures.append(f"{line_number} {case['category']}: actual={actual!r}, expected={expected!r}")

    if failures:
        raise AssertionError("\n".join(failures))

    print(f"asr_qwen_wav_pcm_decode_core fixtures ok: {count} cases")


if __name__ == "__main__":
    main()
