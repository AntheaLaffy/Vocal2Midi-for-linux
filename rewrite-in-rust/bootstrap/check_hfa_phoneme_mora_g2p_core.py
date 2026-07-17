"""Check HubertFA phoneme and mora G2P fixtures against legacy Python."""

from __future__ import annotations

import hashlib
import json
import sys
import unicodedata
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = REPO_ROOT / "rewrite-in-rust/fixtures/hfa_phoneme_mora_g2p_core.jsonl"
sys.path.insert(0, str(REPO_ROOT))

from inference.HubertFA.tools.g2p import (  # noqa: E402
    BaseG2P,
    JapanesePhonemeMoraG2P,
    PhonemeG2P,
)


class FixtureBaseG2P(BaseG2P):
    def __init__(self, language: str | None, output: tuple[list[str], list[str], list[int]]):
        super().__init__(language)
        self.output = output

    def _g2p(self, input_text: str) -> tuple[list[str], list[str], list[int]]:
        return self.output


def load_cases() -> list[dict[str, Any]]:
    return [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf-8").splitlines()
        if line and not line.startswith("#")
    ]


def encode_value(value: tuple[list[str], list[str], list[int]]) -> dict[str, Any]:
    phonemes, words, phoneme_to_word = value
    return {
        "value": {
            "phonemes": phonemes,
            "words": words,
            "phoneme_to_word": phoneme_to_word,
        }
    }


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    kind = case["kind"]
    if kind == "lowercase_scalar_digest":
        digest = hashlib.md5(usedforsecurity=False)
        scalar_count = 0
        for codepoint in range(0x110000):
            if 0xD800 <= codepoint <= 0xDFFF:
                continue
            digest.update(codepoint.to_bytes(4, "big"))
            digest.update(chr(codepoint).lower().encode("utf-8"))
            digest.update(b"\0")
            scalar_count += 1
        return {
            "value": {
                "unicode_version": unicodedata.unidata_version,
                "scalar_count": scalar_count,
                "md5": digest.hexdigest(),
            }
        }

    language = case["language"]
    if kind == "phoneme":
        converter: BaseG2P = PhonemeG2P(language)
    elif kind == "mora":
        converter = JapanesePhonemeMoraG2P(language)
    elif kind == "base_contract":
        output = case["output"]
        converter = FixtureBaseG2P(
            language,
            (
                output["phonemes"],
                output["words"],
                output["phoneme_to_word"],
            ),
        )
    else:
        raise AssertionError(f"unsupported fixture kind: {kind}")

    try:
        return encode_value(converter(case.get("text", "ignored")))
    except BaseException as error:
        return {
            "error": {
                "type": type(error).__name__,
                "message": str(error),
            }
        }


def main() -> None:
    cases = load_cases()
    for case in cases:
        actual = run_case(case)
        if actual != case["expect"]:
            raise AssertionError(
                f"{case['case_id']}:\n"
                f"actual={json.dumps(actual, ensure_ascii=False, sort_keys=True)}\n"
                f"expect={json.dumps(case['expect'], ensure_ascii=False, sort_keys=True)}"
            )

    repeated_converter = JapanesePhonemeMoraG2P("ja")
    first = repeated_converter("ka SP shi")
    second = repeated_converter("ka SP shi")
    if first != second:
        raise AssertionError("JapanesePhonemeMoraG2P repeated call changed output")
    print(f"validated {len(cases)} hfa_phoneme_mora_g2p_core fixtures")


if __name__ == "__main__":
    main()
