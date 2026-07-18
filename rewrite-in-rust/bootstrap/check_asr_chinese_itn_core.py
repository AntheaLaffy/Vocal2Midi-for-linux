from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from inference.qwen3asr_dml.chinese_itn import chinese_to_num


FIXTURE_PATH = Path("rewrite-in-rust/fixtures/asr_chinese_itn_core.jsonl")


def main() -> None:
    failures: list[str] = []
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf-8").splitlines(), 1):
        if not line:
            continue
        case = json.loads(line)
        actual = chinese_to_num(case["input"])
        expected = case["expected"]
        if actual != expected:
            failures.append(
                f"{line_number} {case['category']}: {case['input']!r} -> {actual!r}, expected {expected!r}"
            )

    if failures:
        raise AssertionError("\n".join(failures))

    print(f"asr_chinese_itn_core fixtures ok: {line_number} cases")


if __name__ == "__main__":
    main()
