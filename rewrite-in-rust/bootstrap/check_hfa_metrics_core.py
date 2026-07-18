"""Check HubertFA metrics fixtures against legacy Python."""

from __future__ import annotations

import argparse
import copy
import json
import math
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "hfa_metrics_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

import textgrid as tg  # noqa: E402
from inference.HubertFA.tools.metrics import (  # noqa: E402
    BoundaryEditDistance,
    BoundaryEditRatio,
    BoundaryEditRatioWeighted,
    CustomPointTier,
    IntersectionOverUnion,
    VlabelerEditRatio,
    VlabelerEditsCount,
    compute_lcs_matches,
    get_matched_pairs,
)


def point(data: dict[str, Any]) -> tg.Point:
    return tg.Point(float(data["time"]), data["mark"])


def tier(points: list[dict[str, Any]]) -> CustomPointTier:
    result = CustomPointTier(name="fixture", minTime=0.0, maxTime=1.0)
    for item in points:
        result.addPoint(point(item))
    return result


def encode_point(value: tg.Point) -> dict[str, Any]:
    return {"time": value.time, "mark": value.mark}


def encode_error(error: Exception) -> dict[str, Any]:
    return {"type": type(error).__name__, "message": str(error)}


def project_call(callable_) -> dict[str, Any]:
    try:
        value = callable_()
    except Exception as error:  # noqa: BLE001 - fixture captures exact legacy error surface.
        return {"error": encode_error(error)}
    return {"ok": value}


def project_compute(metric: Any, request: Any = None) -> dict[str, Any]:
    if request is None:
        return project_call(metric.compute)
    if isinstance(request, dict) and request.get("$kind") == "str":
        return project_call(lambda: metric.compute(request["value"]))
    if isinstance(request, dict) and request.get("$kind") == "list":
        return project_call(lambda: metric.compute(request["items"]))
    raise AssertionError(f"unknown compute request {request!r}")


def boundary_state(metric: BoundaryEditDistance) -> dict[str, Any]:
    return {
        "distance": metric.distance,
        "phonemes": metric.phonemes,
        "error_phonemes": metric.error_phonemes,
    }


def run_custom_point_order(case: dict[str, Any]) -> Any:
    result = CustomPointTier(name="fixture", minTime=0.0, maxTime=1.0)
    observations = []
    for item in case["points"]:
        result.addPoint(point(item))
        observations.append([encode_point(value) for value in result.points])
    return {"observations": observations, "final": [encode_point(value) for value in result.points]}


def run_vlabeler_count(case: dict[str, Any]) -> Any:
    metric = VlabelerEditsCount(
        move_min_frames=case.get("move_min_frames", 1),
        move_max_frames=case.get("move_max_frames", 2),
    )
    observations = []
    for operation in case["operations"]:
        if operation["op"] == "update":
            metric.update(tier(operation["pred"]), tier(operation["target"]))
            observations.append({"op": "update", "compute": metric.compute()})
        elif operation["op"] == "compute":
            observations.append({"op": "compute", "compute": metric.compute()})
        elif operation["op"] == "reset":
            metric.reset()
            observations.append({"op": "reset", "compute": metric.compute()})
        else:
            raise AssertionError(f"unknown operation {operation['op']!r}")
    return observations


def run_vlabeler_ratio(case: dict[str, Any]) -> Any:
    metric = VlabelerEditRatio(
        move_min_frames=case.get("move_min_frames", 1),
        move_max_frames=case.get("move_max_frames", 2),
    )
    observations = []
    for operation in case["operations"]:
        if operation["op"] == "update":
            metric.update(tier(operation["pred"]), tier(operation["target"]))
            observations.append(
                {
                    "op": "update",
                    "distance": metric.edit_distance.compute(),
                    "total": metric.total,
                    "compute": metric.compute(),
                }
            )
        elif operation["op"] == "compute":
            observations.append({"op": "compute", "compute": metric.compute()})
        elif operation["op"] == "reset":
            metric.reset()
            observations.append(
                {
                    "op": "reset",
                    "distance": metric.edit_distance.compute(),
                    "total": metric.total,
                    "compute": metric.compute(),
                }
            )
        else:
            raise AssertionError(f"unknown operation {operation['op']!r}")
    return observations


def run_iou(case: dict[str, Any]) -> Any:
    metric = IntersectionOverUnion()
    observations = []
    for operation in case["operations"]:
        if operation["op"] == "update":
            metric.update(tier(operation["pred"]), tier(operation["target"]))
            observations.append(
                {
                    "op": "update",
                    "intersection": copy.deepcopy(metric.intersection),
                    "sum": copy.deepcopy(metric.sum),
                }
            )
        elif operation["op"] == "compute":
            observations.append({"op": "compute", "request": operation.get("request"), **project_compute(metric, operation.get("request"))})
        elif operation["op"] == "reset":
            metric.reset()
            observations.append(
                {
                    "op": "reset",
                    "intersection": copy.deepcopy(metric.intersection),
                    "sum": copy.deepcopy(metric.sum),
                }
            )
        else:
            raise AssertionError(f"unknown operation {operation['op']!r}")
    return observations


def run_lcs(case: dict[str, Any]) -> Any:
    pred = tier(case["pred"])
    target = tier(case["target"])
    pred_matched, target_matched = get_matched_pairs(pred, target)
    return {
        "matches": [list(item) for item in compute_lcs_matches(pred, target)],
        "pred_matched": [encode_point(value) for value in pred_matched],
        "target_matched": [encode_point(value) for value in target_matched],
    }


def run_boundary_distance(case: dict[str, Any]) -> Any:
    metric = BoundaryEditDistance()
    observations = []
    for operation in case["operations"]:
        if operation["op"] == "update":
            result = metric.update(tier(operation["pred"]), tier(operation["target"]))
            observations.append(
                {
                    "op": "update",
                    "ok": result,
                    "state": boundary_state(metric),
                    "compute": metric.compute(),
                }
            )
        elif operation["op"] == "reset":
            metric.reset()
            observations.append(
                {
                    "op": "reset",
                    "state": boundary_state(metric),
                    "compute": metric.compute(),
                }
            )
        elif operation["op"] == "compute":
            observations.append({"op": "compute", "compute": metric.compute()})
        else:
            raise AssertionError(f"unknown operation {operation['op']!r}")
    return observations


def run_boundary_ratio(case: dict[str, Any], weighted: bool) -> Any:
    metric = BoundaryEditRatioWeighted() if weighted else BoundaryEditRatio()
    observations = []
    for operation in case["operations"]:
        if operation["op"] == "update":
            projected = project_call(lambda: metric.update(tier(operation["pred"]), tier(operation["target"])))
            observations.append(
                {
                    "op": "update",
                    **projected,
                    "duration": metric.duration,
                    "distance_state": boundary_state(metric.distance_metric),
                    "compute": metric.compute(),
                    **(
                        {"counts": metric.counts, "error": metric.error}
                        if weighted
                        else {}
                    ),
                }
            )
        elif operation["op"] == "compute":
            observations.append({"op": "compute", "compute": metric.compute()})
        elif operation["op"] == "reset":
            observations.append({"op": "reset", **project_call(metric.reset)})
        else:
            raise AssertionError(f"unknown operation {operation['op']!r}")
    return observations


def run_case(case: dict[str, Any]) -> Any:
    kind = case["kind"]
    if kind == "custom_point_order":
        return run_custom_point_order(case)
    if kind == "vlabeler_count":
        return run_vlabeler_count(case)
    if kind == "vlabeler_ratio":
        return run_vlabeler_ratio(case)
    if kind == "iou":
        return run_iou(case)
    if kind == "lcs":
        return run_lcs(case)
    if kind == "boundary_distance":
        return run_boundary_distance(case)
    if kind == "boundary_ratio":
        return run_boundary_ratio(case, weighted=False)
    if kind == "boundary_ratio_weighted":
        return run_boundary_ratio(case, weighted=True)
    raise AssertionError(f"unknown fixture kind {kind!r}")


CASES: list[dict[str, Any]] = [
    {
        "case_id": "custom_point_order_duplicate_and_bypass_validation",
        "kind": "custom_point_order",
        "points": [
            {"time": 0.5, "mark": "first"},
            {"time": 0.25, "mark": "early"},
            {"time": 0.5, "mark": "duplicate_before"},
            {"time": -0.1, "mark": "before_min"},
            {"time": 1.5, "mark": "after_max"},
        ],
    },
    {
        "case_id": "vlabeler_count_empty_and_reset",
        "kind": "vlabeler_count",
        "operations": [
            {"op": "compute"},
            {"op": "update", "pred": [], "target": []},
            {"op": "update", "pred": [], "target": [{"time": 0.0, "mark": "a"}]},
            {"op": "update", "pred": [{"time": 0.0, "mark": "a"}], "target": []},
            {"op": "reset"},
        ],
    },
    {
        "case_id": "vlabeler_count_costs_truncation_and_repeated_targets",
        "kind": "vlabeler_count",
        "operations": [
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "a"}, {"time": 1.0, "mark": "b"}],
                "target": [{"time": 0.0, "mark": "a"}, {"time": 1.0, "mark": "b"}],
            },
            {
                "op": "update",
                "pred": [{"time": 1.0, "mark": "a"}],
                "target": [{"time": 0.0, "mark": "a"}],
            },
            {
                "op": "update",
                "pred": [{"time": 2.0, "mark": "a"}],
                "target": [{"time": 0.0, "mark": "a"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "x"}],
                "target": [{"time": 0.0, "mark": "a"}],
            },
            {
                "op": "update",
                "pred": [{"time": 1.0, "mark": "x"}],
                "target": [{"time": 0.0, "mark": "a"}],
            },
            {
                "op": "update",
                "pred": [
                    {"time": 0.0, "mark": "a"},
                    {"time": 0.1, "mark": "b"},
                    {"time": 0.2, "mark": "extra"},
                ],
                "target": [
                    {"time": 0.0, "mark": "a"},
                    {"time": 0.1, "mark": "c"},
                ],
            },
            {
                "op": "update",
                "pred": [{"time": 5.0, "mark": "z"}],
                "target": [
                    {"time": 0.0, "mark": "a"},
                    {"time": 0.1, "mark": "a"},
                    {"time": 0.2, "mark": "b"},
                ],
            },
        ],
    },
    {
        "case_id": "vlabeler_ratio_denominator_rounding_and_reset",
        "kind": "vlabeler_ratio",
        "operations": [
            {"op": "compute"},
            {
                "op": "update",
                "pred": [{"time": 1.0, "mark": "x"}],
                "target": [
                    {"time": 0.0, "mark": "a"},
                    {"time": 0.1, "mark": "a"},
                    {"time": 0.2, "mark": "b"},
                ],
            },
            {
                "op": "update",
                "pred": [{"time": 1.0, "mark": "a"}],
                "target": [{"time": 0.0, "mark": "a"}],
            },
            {"op": "reset"},
        ],
    },
    {
        "case_id": "iou_empty_overlap_list_and_reset",
        "kind": "iou",
        "operations": [
            {"op": "update", "pred": [{"time": 0.0, "mark": "solo"}], "target": [{"time": 0.0, "mark": "solo"}]},
            {"op": "compute"},
            {
                "op": "update",
                "pred": [
                    {"time": 0.0, "mark": "a"},
                    {"time": 1.0, "mark": "b"},
                    {"time": 2.0, "mark": "a"},
                    {"time": 3.0, "mark": "end"},
                ],
                "target": [
                    {"time": 0.5, "mark": "a"},
                    {"time": 1.5, "mark": "c"},
                    {"time": 2.0, "mark": "a"},
                    {"time": 3.0, "mark": "end"},
                ],
            },
            {"op": "compute"},
            {"op": "compute", "request": {"$kind": "str", "value": "missing"}},
            {"op": "compute", "request": {"$kind": "list", "items": ["c", "a", "missing", "b"]}},
            {"op": "reset"},
        ],
    },
    {
        "case_id": "iou_string_zero_union_error",
        "kind": "iou",
        "operations": [
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "z"}, {"time": 0.0, "mark": "end"}],
                "target": [{"time": 0.0, "mark": "z"}, {"time": 0.0, "mark": "end"}],
            },
            {"op": "compute"},
            {"op": "compute", "request": {"$kind": "str", "value": "end"}},
        ],
    },
    {
        "case_id": "lcs_empty_and_no_matches",
        "kind": "lcs",
        "pred": [],
        "target": [{"time": 0.0, "mark": "a"}],
    },
    {
        "case_id": "lcs_repeated_label_tie_pairs",
        "kind": "lcs",
        "pred": [{"time": 0.0, "mark": "a"}, {"time": 1.0, "mark": "b"}, {"time": 2.0, "mark": "a"}],
        "target": [{"time": 0.0, "mark": "a"}, {"time": 1.0, "mark": "a"}, {"time": 2.0, "mark": "b"}],
    },
    {
        "case_id": "boundary_distance_state_mismatch_lcs_reset",
        "kind": "boundary_distance",
        "operations": [
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "a"}, {"time": 1.2, "mark": "b"}],
                "target": [{"time": 0.1, "mark": "a"}, {"time": 1.0, "mark": "b"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "x"}],
                "target": [{"time": 0.0, "mark": "y"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "x"}, {"time": 1.0, "mark": "y"}],
                "target": [{"time": 0.0, "mark": "a"}, {"time": 1.0, "mark": "b"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "x"}, {"time": 1.0, "mark": "y"}],
                "target": [{"time": 0.0, "mark": "a"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "a"}, {"time": 1.4, "mark": "c"}],
                "target": [
                    {"time": 0.0, "mark": "a"},
                    {"time": 1.0, "mark": "b"},
                    {"time": 1.5, "mark": "c"},
                ],
            },
            {"op": "reset"},
        ],
    },
    {
        "case_id": "boundary_distance_half_even_rounding_tie",
        "kind": "boundary_distance",
        "operations": [
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "a"}],
                "target": [{"time": 0.0000005, "mark": "a"}],
            }
        ],
    },
    {
        "case_id": "boundary_ratio_duration_failure_empty_target_and_reset",
        "kind": "boundary_ratio",
        "operations": [
            {"op": "compute"},
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "a"}, {"time": 1.2, "mark": "b"}],
                "target": [{"time": 0.0, "mark": "a"}, {"time": 1.0, "mark": "b"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "x"}],
                "target": [{"time": 0.0, "mark": "y"}],
            },
            {"op": "update", "pred": [], "target": []},
            {"op": "reset"},
        ],
    },
    {
        "case_id": "boundary_ratio_weighted_penalty_defaults_and_reset",
        "kind": "boundary_ratio_weighted",
        "operations": [
            {"op": "compute"},
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "x"}],
                "target": [{"time": 0.0, "mark": "y"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "a"}, {"time": 1.2, "mark": "b"}],
                "target": [{"time": 0.0, "mark": "a"}, {"time": 1.0, "mark": "b"}],
            },
            {
                "op": "update",
                "pred": [{"time": 0.0, "mark": "a"}, {"time": 1.4, "mark": "c"}],
                "target": [
                    {"time": 0.0, "mark": "a"},
                    {"time": 1.0, "mark": "b"},
                    {"time": 1.5, "mark": "c"},
                ],
            },
            {"op": "reset"},
        ],
    },
]


def write_fixtures() -> None:
    lines = []
    for case in CASES:
        fixture = dict(case)
        fixture["expect"] = run_case(case)
        lines.append(json.dumps(fixture, ensure_ascii=False, separators=(",", ":")))
    FIXTURE_PATH.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(lines)} hfa_metrics_core fixtures to {FIXTURE_PATH}")


def assert_close(case_id: str, actual: Any, expected: Any) -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict) or set(actual) != set(expected):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
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
            float(actual), expected, rel_tol=1e-12, abs_tol=1e-12
        ):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
        return
    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def validate_fixtures() -> None:
    cases = [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf-8").splitlines()
        if line and not line.startswith("#")
    ]
    for case in cases:
        assert_close(case["case_id"], run_case(case), case["expect"])
    print(f"validated {len(cases)} hfa_metrics_core fixtures")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        write_fixtures()
    else:
        validate_fixtures()


if __name__ == "__main__":
    main()
