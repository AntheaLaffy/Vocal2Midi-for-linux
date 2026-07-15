"""Check Web model download execution-result fixtures against legacy Python."""

from __future__ import annotations

import json
import pathlib
import sys
from datetime import datetime as RealDateTime
from typing import Any, Callable

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = (
    REWRITE_ROOT
    / "fixtures"
    / "web_model_download_execution_result_contract.jsonl"
)

sys.path.insert(0, str(PROJECT_ROOT))

import web_model_download_manager as model_module  # noqa: E402
from web_model_download_manager import ModelDownloadManager, ModelDownloadTask  # noqa: E402


class FakeSocket:
    def __init__(self) -> None:
        self.emits: list[dict[str, Any]] = []

    def emit(self, event: str, payload: dict[str, Any], room: str | None = None) -> None:
        self.emits.append(
            {
                "event": event,
                "room": room,
                "payload": normalize_value(payload),
            }
        )


class FakeStopEvent:
    def __init__(self, is_set: bool) -> None:
        self._is_set = is_set

    def is_set(self) -> bool:
        return self._is_set

    def set(self) -> None:
        self._is_set = True


class FakeStdout:
    def __init__(self, output: str) -> None:
        self.output = output
        self.index = 0

    def read(self, size: int) -> str:
        if self.index >= len(self.output):
            return ""
        char = self.output[self.index : self.index + size]
        self.index += len(char)
        return char


class FakeProcess:
    pid = 4242

    def __init__(self, spec: dict[str, Any]) -> None:
        self.stdout = FakeStdout(spec["stdout"]) if spec.get("stdout") is not None else None
        self.returncode = spec.get("returncode", 0)
        self.poll_after_read = spec.get("poll_after_read", self.returncode)
        self.wait_plan = list(spec.get("wait_plan", []))
        self.wait_calls: list[dict[str, Any]] = []
        self.output_reader_called = False

    def poll(self):
        if self.stdout is not None and self.stdout.index < len(self.stdout.output):
            return None
        return self.poll_after_read

    def wait(self, timeout=None):
        if timeout is None:
            self.wait_calls.append({"timeout": None, "result": self.returncode})
            return self.returncode

        outcome = self.wait_plan.pop(0) if self.wait_plan else {
            "kind": "return",
            "returncode": self.returncode,
        }
        if outcome["kind"] == "timeout":
            self.wait_calls.append({"timeout": timeout, "result": "timeout"})
            raise model_module.subprocess.TimeoutExpired("download_models.py", timeout)

        result = outcome.get("returncode", self.returncode)
        self.wait_calls.append({"timeout": timeout, "result": result})
        return result


class FakePopenFactory:
    def __init__(self, case: dict[str, Any], process: FakeProcess) -> None:
        self.case = case
        self.process = process
        self.calls: list[dict[str, Any]] = []

    def __call__(self, command, cwd, stdout, stderr, text, bufsize, env, **kwargs):
        self.calls.append(
            {
                "command": list(command),
                "cwd": cwd,
                "stdout_pipe": stdout is model_module.subprocess.PIPE,
                "stderr_stdout": stderr is model_module.subprocess.STDOUT,
                "text": text,
                "bufsize": bufsize,
                "env_subset": {
                    key: env[key]
                    for key in self.case.get("env_keys", [])
                    if key in env
                },
                "kwargs": kwargs,
            }
        )
        error = self.case.get("process", {}).get("popen_error")
        if error is not None:
            raise RuntimeError(error)
        return self.process


class FakeManager(ModelDownloadManager):
    def __init__(self, popen_kwargs: dict[str, Any]) -> None:
        super().__init__()
        self._fake_popen_kwargs = popen_kwargs
        self.termination_calls: list[dict[str, Any]] = []

    def _popen_process_group_kwargs(self) -> dict:
        return dict(self._fake_popen_kwargs)

    def _terminate_process_tree(self, process, force: bool = False) -> None:
        self.termination_calls.append({"force": force})

    def _read_process_output(self, task, socketio, process) -> None:
        process.output_reader_called = True
        super()._read_process_output(task, socketio, process)


def parse_datetime(value: str | None):
    if value is None:
        return None
    return RealDateTime.fromisoformat(value)


def make_task(data: dict[str, Any]) -> ModelDownloadTask:
    task = ModelDownloadTask(
        id=data["task_id"],
        selected_models=list(data["selected_models"]),
        qwen_source=data["qwen_source"],
        force=data["force"],
        proxy_mode=data["proxy_mode"],
        proxy_url=data["proxy_url"],
        status=data["status"],
        progress=data["progress"],
        stage=data["stage"],
        created_at=parse_datetime(data["created_at"]),
        started_at=parse_datetime(data.get("started_at")),
        completed_at=parse_datetime(data.get("completed_at")),
        error=data.get("error"),
        returncode=data.get("returncode"),
        logs=list(data.get("logs", [])),
    )
    task.completed_models = set(data.get("completed_models", []))
    task.active_model = data.get("active_model")
    task.stop_event = (
        FakeStopEvent(data.get("stop_event_set", False))
        if data.get("stop_event_present", True)
        else None
    )
    return task


def normalize_value(value: Any) -> Any:
    if isinstance(value, dict):
        normalized = {key: normalize_value(item) for key, item in value.items()}
        if "timestamp" in normalized:
            normalized["timestamp"] = "__timestamp__"
        if normalized.get("completed_at") is not None:
            normalized["completed_at"] = "__timestamp__"
        if normalized.get("message", "").startswith("Traceback"):
            normalized["message"] = "__traceback__"
        return normalized
    if isinstance(value, list):
        return [normalize_value(item) for item in value]
    return value


def completed_at_value(task: ModelDownloadTask) -> str | None:
    return "__timestamp__" if task.completed_at is not None else None


def completed_models(task: ModelDownloadTask) -> list[str]:
    ordered = [model_id for model_id in task.selected_models if model_id in task.completed_models]
    extras = sorted(model_id for model_id in task.completed_models if model_id not in ordered)
    return ordered + extras


def task_state(task: ModelDownloadTask) -> dict[str, Any]:
    logs = normalize_value(task.logs)
    state = {
        "task_id": task.id,
        "selected_models": task.selected_models,
        "qwen_source": task.qwen_source,
        "force": task.force,
        "proxy_mode": task.proxy_mode,
        "proxy_url": task.proxy_url,
        "status": task.status,
        "progress": task.progress,
        "stage": task.stage,
        "created_at": task.created_at.isoformat(),
        "started_at": task.started_at.isoformat() if task.started_at else None,
        "completed_at": completed_at_value(task),
        "error": task.error,
        "returncode": task.returncode,
        "logs": logs,
        "log_count": len(task.logs),
        "completed_models": completed_models(task),
        "active_model": task.active_model,
        "stop_event_present": task.stop_event is not None,
        "stop_event_set": bool(task.stop_event and task.stop_event.is_set()),
        "process_assigned": task.process is not None,
    }
    if logs:
        state["first_log_message"] = logs[0].get("message")
        state["last_log_message"] = logs[-1].get("message")
    return state


def summarize_emits(emits: list[dict[str, Any]]) -> dict[str, Any]:
    summary: dict[str, Any] = {
        "events": [emit["event"] for emit in emits],
        "logs": [],
        "progress": [],
        "status_changes": [],
    }
    for emit in emits:
        payload = emit["payload"]
        if emit["event"] == "log":
            summary["logs"].append(
                {
                    "message": payload["message"],
                    "level": payload["level"],
                }
            )
        elif emit["event"] == "progress":
            summary["progress"].append(
                {
                    "progress": payload["progress"],
                    "stage": payload["stage"],
                }
            )
        elif emit["event"] == "status_change":
            summary["status_changes"].append(
                {
                    "status": payload["status"],
                    "stage": payload["stage"],
                    "progress": payload["progress"],
                    "error": payload["error"],
                    "returncode": payload["returncode"],
                    "completed_at": payload["completed_at"],
                    "proxy_url": payload["proxy_url"],
                    "logs_len": len(payload["logs"]),
                }
            )
    return summary


def assert_subset(case_id: str, actual: Any, expected: Any, path: str = "") -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {path} actual is not object")
        for key, expected_value in expected.items():
            if key not in actual:
                raise AssertionError(f"{case_id}: missing key {path}.{key}")
            assert_subset(case_id, actual[key], expected_value, f"{path}.{key}")
        return
    if isinstance(expected, list):
        if not isinstance(actual, list):
            raise AssertionError(f"{case_id}: {path} actual is not list")
        if len(actual) != len(expected):
            raise AssertionError(
                f"{case_id}: {path} list length {len(actual)} != {len(expected)}"
            )
        for index, (actual_item, expected_item) in enumerate(zip(actual, expected)):
            assert_subset(case_id, actual_item, expected_item, f"{path}[{index}]")
        return
    if actual != expected:
        raise AssertionError(f"{case_id}: {path} {actual!r} != {expected!r}")


def patch_attr(obj: Any, name: str, value: Any) -> Callable[[], None]:
    had_attr = hasattr(obj, name)
    old_value = getattr(obj, name, None)
    setattr(obj, name, value)

    def restore() -> None:
        if had_attr:
            setattr(obj, name, old_value)
        else:
            delattr(obj, name)

    return restore


def run_execute_download(case: dict[str, Any]) -> None:
    manager = FakeManager(case.get("popen_kwargs", {}))
    socket = FakeSocket()
    task = make_task(case["task"])
    manager.tasks[task.id] = task
    manager.active_task_id = case.get("active_task_id")

    process = FakeProcess(case["process"])
    popen_factory = FakePopenFactory(case, process)

    old_env = dict(model_module.os.environ)
    restores = [
        patch_attr(model_module.sys, "executable", case["python_executable"]),
        patch_attr(model_module, "ROOT_DIR", pathlib.Path(case["root_dir"])),
        patch_attr(model_module.subprocess, "Popen", popen_factory),
    ]
    try:
        model_module.os.environ.clear()
        model_module.os.environ.update(case.get("base_env", {}))
        manager._execute_download(task, socket)
    finally:
        model_module.os.environ.clear()
        model_module.os.environ.update(old_env)
        for restore in reversed(restores):
            restore()

    actual = {
        "task": task_state(task),
        "active_task_id": manager.active_task_id,
        "process_assigned": task.process is not None,
        "output_reader_called": process.output_reader_called,
        "popen_calls": popen_factory.calls,
        "wait_calls": process.wait_calls,
        "termination_calls": manager.termination_calls,
        "emits": summarize_emits(socket.emits),
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "execute_download":
        run_execute_download(case)
    else:
        raise AssertionError(f"unknown operation {operation!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        try:
            case = json.loads(line)
        except json.JSONDecodeError as exc:
            raise AssertionError(
                f"fixture line {line_number} is invalid JSON: {exc}"
            ) from exc
        run_case(case)


if __name__ == "__main__":
    main()
