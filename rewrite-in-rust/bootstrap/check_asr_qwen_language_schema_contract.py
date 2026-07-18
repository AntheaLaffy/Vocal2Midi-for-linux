from __future__ import annotations

import json
import sys
from dataclasses import asdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from inference.qwen3asr_dml.schema import (  # noqa: E402
    ASREngineConfig,
    DecodeResult,
    MsgType,
    StreamingMessage,
    TranscribeResult,
)
from inference.qwen3asr_dml.utils import (  # noqa: E402
    SUPPORTED_LANGUAGES,
    normalize_language_name,
    validate_language,
)


FIXTURE_PATH = Path("rewrite-in-rust/fixtures/asr_qwen_language_schema_contract.jsonl")


def result(fn, *args):
    try:
        value = fn(*args)
        return {"ok": True, "value": value}
    except Exception as exc:  # noqa: BLE001 - fixture captures legacy exception projection.
        return {"ok": False, "error_type": type(exc).__name__, "message": str(exc)}


def streaming_snapshot(message: StreamingMessage) -> dict:
    return {
        "msg_type_name": message.msg_type.name,
        "msg_type_value": message.msg_type.value,
        "data": message.data,
        "is_last": message.is_last,
        "encode_time": message.encode_time,
        "repr": repr(message),
    }


def actual_for(case: dict):
    kind = case["kind"]
    category = case["category"]
    if kind == "normalize_language_name":
        return result(normalize_language_name, case["input"])
    if kind == "supported_languages":
        return SUPPORTED_LANGUAGES
    if kind == "validate_language":
        return result(validate_language, case["input"])
    if kind == "msg_types":
        return [
            {"name": item.name, "value": item.value, "str": str(item), "repr": repr(item)}
            for item in MsgType
        ]
    if kind == "streaming_message":
        if category == "defaults":
            return streaming_snapshot(StreamingMessage(MsgType.CMD_ENCODE))
        if category == "custom":
            return streaming_snapshot(
                StreamingMessage(MsgType.MSG_DONE, data={"k": [1, 2]}, is_last=True, encode_time=1.25)
            )
    if kind == "decode_result":
        if category == "defaults":
            value = DecodeResult()
            return {"snapshot": asdict(value), "repr": repr(value)}
        if category == "custom":
            value = DecodeResult(
                text="abc",
                stable_tokens=[1, 2],
                t_prefill=0.5,
                t_generate=1.5,
                n_prefill=3,
                n_generate=4,
                is_aborted=True,
            )
            return {"snapshot": asdict(value), "repr": repr(value)}
        if category == "stable_tokens_independent":
            return {
                "same_object": DecodeResult().stable_tokens is DecodeResult().stable_tokens,
                "first": DecodeResult().stable_tokens,
                "second": DecodeResult(stable_tokens=[1, 2]).stable_tokens,
            }
    if kind == "asr_engine_config":
        if category == "defaults":
            value = ASREngineConfig("model")
        elif category == "custom":
            value = ASREngineConfig(
                "m",
                encoder_frontend_fn="front.onnx",
                encoder_backend_fn="back.onnx",
                llm_fn="llm.gguf",
                use_dml=False,
                n_ctx=1024,
                chunk_size=12.5,
                memory_num=2,
                max_decode_tokens=42,
                llama_backend="cpu",
                verbose=False,
            )
        else:
            raise AssertionError(f"unknown ASREngineConfig category {category}")
        return {"snapshot": asdict(value), "repr": repr(value)}
    if kind == "transcribe_result":
        if category == "defaults":
            value = TranscribeResult("txt")
        elif category == "custom":
            value = TranscribeResult("txt", performance={"rtf": 0.5})
        else:
            raise AssertionError(f"unknown TranscribeResult category {category}")
        return {"snapshot": asdict(value), "repr": repr(value)}
    if kind == "constructor_error":
        constructors = {
            "StreamingMessage": StreamingMessage,
            "ASREngineConfig": ASREngineConfig,
            "TranscribeResult": TranscribeResult,
        }
        return result(constructors[category])
    raise AssertionError(f"unknown fixture kind {kind}")


def expected_for(case: dict):
    kind = case["kind"]
    if kind in {"normalize_language_name", "validate_language", "constructor_error"}:
        return case["result"]
    if kind == "supported_languages":
        return case["languages"]
    if kind == "msg_types":
        return case["items"]
    if kind in {"streaming_message"}:
        return case["snapshot"]
    if kind == "decode_result" and case["category"] == "stable_tokens_independent":
        return {
            "same_object": case["same_object"],
            "first": case["first"],
            "second": case["second"],
        }
    if kind in {"decode_result", "asr_engine_config", "transcribe_result"}:
        return {"snapshot": case["snapshot"], "repr": case["repr"]}
    return {
        "same_object": case.get("same_object"),
        "first": case.get("first"),
        "second": case.get("second"),
    }


def main() -> None:
    failures: list[str] = []
    count = 0
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf-8").splitlines(), 1):
        if not line:
            continue
        count += 1
        case = json.loads(line)
        actual = actual_for(case)
        expected = expected_for(case)
        if actual != expected:
            failures.append(
                f"{line_number} {case['kind']} {case['category']}: actual={actual!r}, expected={expected!r}"
            )

    if failures:
        raise AssertionError("\n".join(failures))

    print(f"asr_qwen_language_schema_contract fixtures ok: {count} cases")


if __name__ == "__main__":
    main()
