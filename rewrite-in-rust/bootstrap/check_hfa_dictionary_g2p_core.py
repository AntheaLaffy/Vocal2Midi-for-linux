"""Check HubertFA dictionary G2P fixtures against legacy Python."""

from __future__ import annotations

import json
import locale
import sys
import tempfile
import warnings
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = REPO_ROOT / "rewrite-in-rust/fixtures/hfa_dictionary_g2p_core.jsonl"
sys.path.insert(0, str(REPO_ROOT))

from inference.HubertFA.tools.g2p import DictionaryG2P  # noqa: E402


def load_cases() -> list[dict[str, Any]]:
    return [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf-8").splitlines()
        if line and not line.startswith("#")
    ]


def normalize_error(error: BaseException, temp_root: Path) -> dict[str, str]:
    return {
        "type": type(error).__name__,
        "message": str(error).replace(str(temp_root), "<TMP>"),
    }


def run_call(converter: DictionaryG2P, text: str, temp_root: Path) -> dict[str, Any]:
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        try:
            phonemes, words, phoneme_to_word = converter(text)
            result: dict[str, Any] = {
                "value": {
                    "phonemes": phonemes,
                    "words": words,
                    "phoneme_to_word": phoneme_to_word,
                }
            }
        except BaseException as error:  # noqa: BLE001 - legacy errors are fixture data.
            result = {"error": normalize_error(error, temp_root)}

        result["warnings"] = [
            {"category": item.category.__name__, "message": str(item.message)}
            for item in caught
        ]
        return result


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    with tempfile.TemporaryDirectory() as directory:
        temp_root = Path(directory)
        dictionary_spec = case["dictionary"]
        dictionary_path = temp_root / dictionary_spec.get("path_name", "dictionary.txt")
        path_kind = dictionary_spec.get("path_kind", "file")

        if path_kind == "directory":
            dictionary_path.mkdir()
        elif path_kind == "file":
            if "bytes_hex" in dictionary_spec:
                dictionary_path.write_bytes(bytes.fromhex(dictionary_spec["bytes_hex"]))
            else:
                dictionary_path.write_text(
                    dictionary_spec["content"],
                    encoding="utf-8",
                    newline="",
                )
        elif path_kind != "missing":
            raise AssertionError(f"unsupported path_kind: {path_kind!r}")

        try:
            converter = DictionaryG2P(case["language"], dictionary_path)
        except BaseException as error:  # noqa: BLE001 - legacy errors are fixture data.
            return {"error": normalize_error(error, temp_root)}

        result: dict[str, Any] = {"dictionary": converter.dictionary}
        if "after_construct_content" in dictionary_spec:
            dictionary_path.write_text(
                dictionary_spec["after_construct_content"],
                encoding="utf-8",
                newline="",
            )
        result["calls"] = [
            run_call(converter, text, temp_root) for text in case["texts"]
        ]
        return result


def main() -> None:
    if sys.version_info[:2] != (3, 12):
        raise AssertionError(f"fixtures require Python 3.12, got {sys.version.split()[0]}")
    if locale.getencoding().lower().replace("_", "-") != "utf-8":
        raise AssertionError(
            "fixtures require the Linux UTF-8 default open() encoding, got "
            f"{locale.getencoding()}"
        )

    cases = load_cases()
    for case in cases:
        actual = run_case(case)
        if actual != case["expect"]:
            raise AssertionError(
                f"{case['case_id']}:\n"
                f"actual={json.dumps(actual, ensure_ascii=False, sort_keys=True)}\n"
                f"expect={json.dumps(case['expect'], ensure_ascii=False, sort_keys=True)}"
            )

    print(f"validated {len(cases)} hfa_dictionary_g2p_core fixtures")


if __name__ == "__main__":
    main()
