from __future__ import annotations

import json
import math
from pathlib import Path

import numpy as np
from scipy.signal import resample_poly


FIXTURE_PATH = Path("rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl")
TOLERANCE = 1e-7
SUM_TOLERANCE = 1e-5


def parse_float(value):
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return value


def encode_float(value: float):
    if math.isnan(value):
        return "nan"
    if math.isinf(value):
        return "inf" if value > 0 else "-inf"
    return float(value)


def input_for(case: dict) -> np.ndarray:
    if "input_spec" not in case:
        return np.array([parse_float(value) for value in case["input"]], dtype=np.float32)

    spec = case["input_spec"]
    if spec["kind"] != "dual_sine":
        raise ValueError(f"unsupported input_spec kind: {spec['kind']}")

    source_rate = np.float32(spec["source_rate"])
    t = np.arange(spec["length"], dtype=np.float32) / source_rate
    x = np.zeros(spec["length"], dtype=np.float32)
    for component in spec["components"]:
        amplitude = np.float32(component["amplitude"])
        phase_step = np.float32(2 * np.pi * component["frequency"])
        x = (x + amplitude * np.sin(phase_step * t)).astype(np.float32)
    return x


def result_for(case: dict) -> dict:
    x = input_for(case)
    try:
        y = resample_poly(x, case["target_rate"], case["source_rate"]).astype(np.float32, copy=False)
    except Exception as exc:  # noqa: BLE001 - fixture captures legacy exception projection.
        return {"ok": False, "error_type": type(exc).__name__, "message": str(exc)}

    result = {
        "ok": True,
        "dtype": str(y.dtype),
        "shape": list(y.shape),
    }
    if "selected_values" in case["expected"]:
        finite = y[np.isfinite(y)]
        result["selected_values"] = [
            [int(index), encode_float(float(y[index]))]
            for index, _value in case["expected"]["selected_values"]
        ]
        result["finite_sum"] = encode_float(float(np.sum(finite, dtype=np.float64)))
        result["finite_abs_sum"] = encode_float(float(np.sum(np.abs(finite), dtype=np.float64)))
        return result

    return {
        **result,
        "values": [encode_float(float(value)) for value in y.tolist()],
    }


def values_close(actual: list, expected: list) -> bool:
    if len(actual) != len(expected):
        return False
    for got, want in zip(actual, expected):
        if got == "nan" or want == "nan":
            if got != want:
                return False
        elif got != "inf" and got != "-inf" and want != "inf" and want != "-inf":
            if abs(float(got) - float(want)) > TOLERANCE:
                return False
        elif got != want:
            return False
    return True


def selected_values_close(actual: list, expected: list) -> bool:
    if len(actual) != len(expected):
        return False
    for got, want in zip(actual, expected):
        if got[0] != want[0] or not values_close([got[1]], [want[1]]):
            return False
    return True


def close_enough(actual: dict, expected: dict) -> bool:
    if actual.get("ok") != expected.get("ok"):
        return False
    if not expected.get("ok"):
        return actual == expected
    if actual["dtype"] != expected["dtype"] or actual["shape"] != expected["shape"]:
        return False
    if "selected_values" in expected:
        if not selected_values_close(actual["selected_values"], expected["selected_values"]):
            return False
        for key in ("finite_sum", "finite_abs_sum"):
            if abs(float(actual[key]) - float(expected[key])) > SUM_TOLERANCE:
                return False
        return True
    return values_close(actual["values"], expected["values"])


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

    print(f"asr_resample_poly_contract fixtures ok: {count} cases")


if __name__ == "__main__":
    main()
