"""Check lyric matching file/state fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import io
import json
import math
import pathlib
import sys
import tempfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "lyric_matching_file_contract_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.LyricFA.tools.language_processors import LyricData  # noqa: E402
from inference.LyricFA.tools.lyric_matcher import LyricMatcher, LyricMatchingPipeline, ProcessResult  # noqa: E402


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


class FakeHighlighter:
    def highlight_differences(self, asr_result: str, match_phonetic: str, match_text: str):
        return asr_result, match_phonetic, match_text, 0


class FakeMatcher:
    def __init__(self, fake: dict[str, Any]) -> None:
        self.fake = fake
        self.highlighter = FakeHighlighter()

    def process_lyric_file(self, lyric_path: str) -> LyricData:
        lyric_data = self.fake.get("lyric_data")
        if lyric_data is None:
            stem = pathlib.Path(lyric_path).stem
            lyric_data = self.fake.get("lyric_data_by_name", {}).get(stem)
        if lyric_data is None:
            raise IOError(f"fake lyric data missing for {lyric_path}")
        return make_lyric_data(lyric_data)

    def process_asr_content(self, lab_content: str) -> tuple[list[str], list[str]]:
        return list(self.fake.get("asr_text", [])), list(self.fake.get("asr_phonetic", []))

    def align_lyric_with_asr(
        self,
        asr_phonetic: list[str],
        lyric_text: list[str],
        lyric_phonetic: list[str],
    ) -> tuple[str, str, str]:
        return (
            self.fake.get("matched_text", ""),
            self.fake.get("matched_phonetic", ""),
            self.fake.get("reason", ""),
        )

    @staticmethod
    def save_to_json(json_path: str, text: str, phonetic: str) -> None:
        LyricMatcher.save_to_json(json_path, text, phonetic)


def make_lyric_data(value: dict[str, Any]) -> LyricData:
    return LyricData(
        text_list=list(value.get("text_list", [])),
        phonetic_list=list(value.get("phonetic_list", [])),
        raw_text=value.get("raw_text", ""),
    )


def encode_process_result(result: ProcessResult | None) -> dict[str, Any] | None:
    if result is None:
        return None
    return {
        "lab_name": result.lab_name,
        "matched_text": result.matched_text,
        "matched_phonetic": result.matched_phonetic,
        "asr_phonetic": result.asr_phonetic,
        "asr_text": result.asr_text,
        "reason": result.reason,
    }


def make_process_result(value: dict[str, Any]) -> ProcessResult:
    return ProcessResult(
        lab_name=value["lab_name"],
        matched_text=value["matched_text"],
        matched_phonetic=value["matched_phonetic"],
        asr_phonetic=list(value["asr_phonetic"]),
        asr_text=list(value["asr_text"]),
        reason=value.get("reason", ""),
    )


def encode_state(pipeline: LyricMatchingPipeline) -> dict[str, Any]:
    return {
        "total_files": pipeline.total_files,
        "success_count": pipeline.success_count,
        "diff_count": pipeline.diff_count,
        "no_match_count": pipeline.no_match_count,
        "missing_lyrics": list(pipeline.missing_lyrics),
    }


def make_pipeline(language: str, tmp_path: pathlib.Path, fake: dict[str, Any] | None = None, diff_threshold: int = 5) -> LyricMatchingPipeline:
    pipeline = LyricMatchingPipeline(
        str(tmp_path / "lyrics"),
        str(tmp_path / "labs"),
        str(tmp_path / "json"),
        language,
        diff_threshold=diff_threshold,
    )
    pipeline.matcher = FakeMatcher(fake or {})
    return pipeline


def read_json(path: pathlib.Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    kind = case["kind"]
    if kind == "extract_filename":
        return {
            "values": [
                LyricMatchingPipeline._extract_filename_without_extension(path)
                for path in case["paths"]
            ]
        }
    if kind == "lab_to_lyric_name":
        items = []
        for path in case["paths"]:
            lab_name = LyricMatchingPipeline._extract_filename_without_extension(path)
            items.append({"lab_name": lab_name, "lyric_name": lab_name.rsplit("_", 1)[0]})
        return {"items": items}

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = pathlib.Path(tmp)
        for folder in ["lyrics", "labs", "json"]:
            (tmp_path / folder).mkdir()

        if kind == "missing_lyric_dedup":
            pipeline = make_pipeline(case.get("language", "en"), tmp_path)
            results = []
            for relative_path in case["lab_paths"]:
                lab_path = tmp_path / "labs" / relative_path
                lab_path.write_text("unused", encoding="utf-8")
                with contextlib.redirect_stdout(io.StringIO()):
                    results.append(encode_process_result(pipeline.process_single_file(str(lab_path), {})))
            return {"results": results, "state": encode_state(pipeline)}

        if kind == "process_single_file":
            pipeline = make_pipeline(case.get("language", "en"), tmp_path, case.get("fake", {}))
            lab_path = tmp_path / "labs" / case["lab_path"]
            lab_path.write_text(case.get("lab_content", ""), encoding="utf-8")
            lyric_dict = {
                name: make_lyric_data(value)
                for name, value in case.get("lyric_dict", {}).items()
            }
            with contextlib.redirect_stdout(io.StringIO()):
                result = pipeline.process_single_file(str(lab_path), lyric_dict)
            return {"result": encode_process_result(result), "state": encode_state(pipeline)}

        if kind == "compare_result":
            pipeline = make_pipeline(
                case.get("language", "en"),
                tmp_path,
                {},
                diff_threshold=int(case.get("diff_threshold", 5)),
            )
            result = make_process_result(case["result"])
            with contextlib.redirect_stdout(io.StringIO()):
                pipeline.compare_and_save_result(result)
            json_path = tmp_path / "json" / f"{result.lab_name}.json"
            return {"state": encode_state(pipeline), "json": read_json(json_path)}

        if kind == "execute_single":
            pipeline = make_pipeline(case.get("language", "en"), tmp_path, case.get("fake", {}))
            for filename, content in case.get("lyric_files", {}).items():
                (tmp_path / "lyrics" / filename).write_text(content, encoding="utf-8")
            for filename, content in case.get("lab_files", {}).items():
                (tmp_path / "labs" / filename).write_text(content, encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                pipeline.execute()
            json_files = {
                path.name: read_json(path)
                for path in sorted((tmp_path / "json").glob("*.json"))
            }
            return {"state": encode_state(pipeline), "json_files": json_files}

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
