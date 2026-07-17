"""Check lyric sequence alignment fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "lyric_sequence_alignment_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.LyricFA.tools.sequence_aligner import (  # noqa: E402
    SequenceAligner,
    SmartHighlighter,
    calculate_difference_count,
)


def assert_close(case_id: str, actual: Any, expected: Any) -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {actual!r} is not a dict")
        if set(actual) != set(expected):
            raise AssertionError(f"{case_id}: keys {set(actual)!r} != {set(expected)!r}")
        for key in expected:
            assert_close(f"{case_id}.{key}", actual[key], expected[key])
        return

    if isinstance(expected, list):
        if not isinstance(actual, list) or len(actual) != len(expected):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
        for index, (actual_item, expected_item) in enumerate(zip(actual, expected, strict=True)):
            assert_close(f"{case_id}[{index}]", actual_item, expected_item)
        return

    if isinstance(expected, float):
        if not isinstance(actual, (float, int)) or not math.isclose(
            float(actual),
            expected,
            rel_tol=1e-6,
            abs_tol=1e-6,
        ):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
        return

    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def make_aligner(case: dict[str, Any]) -> SequenceAligner:
    costs = case.get("costs", {})
    return SequenceAligner(
        deletion_cost=costs.get("deletion", 1),
        insertion_cost=costs.get("insertion", 1),
        substitution_cost=costs.get("substitution", 1),
    )


def encode_match_result(value: tuple[str, int, int, list[str] | None, list[str] | None, str]) -> dict[str, Any]:
    matched_text, start, end, matched_phonetic_list, matched_text_list, reason = value
    return {
        "matched_text": matched_text,
        "start": start,
        "end": end,
        "matched_phonetic_list": matched_phonetic_list,
        "matched_text_list": matched_text_list,
        "reason": reason,
    }


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    aligner = make_aligner(case)
    kind = case["kind"]

    if kind == "alignment":
        distance, aligned1, aligned2 = aligner.compute_alignment(case["seq1"], case["seq2"])
        return {"distance": distance, "aligned1": aligned1, "aligned2": aligned2}

    if kind == "find_best_match":
        return encode_match_result(
            aligner.find_best_match(
                case["input_seq"],
                case["reference_seq"],
                case.get("reference_text"),
                max_window_scale=case.get("max_window_scale", 1.3),
                extra_window=case.get("extra_window", 8),
            )
        )

    if kind == "scan_windows":
        best_start, min_edit_dist = aligner._scan_windows(  # noqa: SLF001
            case["input_seq"],
            case["reference_seq"],
            case["window_size"],
            case["input_len"],
        )
        if math.isinf(min_edit_dist):
            min_edit_dist = None
        return {"best_start": best_start, "min_edit_dist": min_edit_dist}

    if kind == "build_match":
        return encode_match_result(
            aligner._build_match_from_alignment(  # noqa: SLF001
                case["input_seq"],
                case["reference_seq"],
                case.get("reference_text"),
                case["best_start"],
                case["window_size"],
            )
        )

    if kind == "return_lyrics":
        matched_text, matched_phonetic, start, end, reason = aligner.find_best_match_and_return_lyrics(
            case["input_pronunciation"],
            case["reference_text"],
            case["reference_pronunciation"],
        )
        return {
            "matched_text": matched_text,
            "matched_phonetic": matched_phonetic,
            "start": start,
            "end": end,
            "reason": reason,
        }

    if kind == "lcs":
        return {"length": aligner.compute_lcs_length(case["seq1"], case["seq2"])}

    if kind == "edit_distance":
        return {"distance": aligner.compute_edit_distance(case["seq1"], case["seq2"])}

    if kind == "difference_count":
        return {"count": calculate_difference_count(case["seq1"], case["seq2"])}

    if kind == "highlight":
        highlighter = SmartHighlighter(aligner)
        asr_highlighted, phonetic_highlighted, text_highlighted, edit_distance = (
            highlighter.highlight_differences(
                case["asr_result"],
                case["match_phonetic"],
                case["match_text"],
            )
        )
        return {
            "asr_highlighted": asr_highlighted,
            "phonetic_highlighted": phonetic_highlighted,
            "text_highlighted": text_highlighted,
            "edit_distance": edit_distance,
        }

    raise AssertionError(f"unknown kind {kind!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        assert_close(case_id, run_case(case), case["expect"])


if __name__ == "__main__":
    main()
