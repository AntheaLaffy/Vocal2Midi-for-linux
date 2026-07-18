from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

import numpy as np

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT))

from inference.romaji_asr.common import (
    chunked,
    decode_logits,
    decode_outputs,
    decode_pred_ids,
    load_vocab,
)


FIXTURE_PATH = Path("rewrite-in-rust/fixtures/asr_romaji_vocab_ctc_decode_core.jsonl")


def encode_id2token(id2token: dict[int, str]) -> dict[str, str]:
    return {str(key): value for key, value in id2token.items()}


def id2token_from_case(case: dict) -> dict[int, str]:
    return {int(key): value for key, value in case["id2token"].items()}


def parse_float_sentinels(value):
    if isinstance(value, list):
        return [parse_float_sentinels(item) for item in value]
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return value


def result_for(case: dict) -> dict:
    try:
        match case["call"]:
            case "load_vocab":
                with tempfile.TemporaryDirectory() as temp_dir:
                    path = Path(temp_dir) / "vocab.json"
                    path.write_text(case["vocab_json"], encoding="utf-8")
                    id2token, blank_id = load_vocab(path)
                return {
                    "ok": True,
                    "id2token": encode_id2token(id2token),
                    "blank_id": blank_id,
                }
            case "decode_pred_ids":
                pred_ids = np.array(case["pred_ids"], dtype=case["dtype"])
                tokens = decode_pred_ids(pred_ids, id2token_from_case(case), int(case["blank_id"]))
                return {"ok": True, "tokens": tokens}
            case "decode_logits":
                logits = np.array(parse_float_sentinels(case["logits"]), dtype=case["dtype"])
                tokens = decode_logits(logits, id2token_from_case(case), int(case["blank_id"]))
                return {"ok": True, "tokens": tokens}
            case "decode_outputs":
                outputs = np.array(parse_float_sentinels(case["outputs"]), dtype=case["dtype"])
                predictions = decode_outputs(outputs, id2token_from_case(case), int(case["blank_id"]))
                return {"ok": True, "predictions": predictions}
            case "chunked":
                return {"ok": True, "chunks": list(chunked(case["items"], case["chunk_size"]))}
            case other:
                raise ValueError(f"unknown fixture call: {other}")
    except Exception as exc:  # noqa: BLE001 - fixture captures legacy exception projection.
        return {"ok": False, "error_type": type(exc).__name__, "message": str(exc)}


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

    print(f"asr_romaji_vocab_ctc_decode_core fixtures ok: {count} cases")


if __name__ == "__main__":
    main()
