"""Background model download tasks for the Vocal2Midi Web UI."""

from __future__ import annotations

import os
import pathlib
import re
import subprocess
import sys
import traceback
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any, Dict, List, Optional

from download_models import (
    GITHUB_MODELS,
    QWEN_LOCAL_DIR,
    QWEN_MODEL_ID,
    ROOT_DIR,
    qwen_has_weights,
    target_has_model,
)


MODEL_LABELS = {
    "game": "GAME",
    "hfa": "HubertFA",
    "rmvpe": "RMVPE",
    "romaji": "romajiASR",
    "qwen": "Qwen3-ASR-1.7B",
}

MODEL_DESCRIPTIONS = {
    "game": "note and pitch extraction",
    "hfa": "Chinese/Japanese forced alignment",
    "rmvpe": "pitch curve estimation for slicing and USTX export",
    "romaji": "Japanese mora ASR",
    "qwen": "Chinese ASR transcription backend",
}

MODEL_ROLES_ZH = {
    "game": "音符与音高提取",
    "hfa": "歌词强制对齐",
    "rmvpe": "音高曲线与智能切片",
    "romaji": "日文 mora / 罗马音识别",
    "qwen": "中文语音识别",
}

VALID_MODEL_IDS = [m.name for m in GITHUB_MODELS] + ["qwen"]
VALID_QWEN_SOURCES = {"auto", "modelscope", "huggingface"}
VALID_PROXY_MODES = {"system", "manual", "none"}
PROXY_ENV_KEYS = (
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
)


@dataclass
class ModelDownloadTask:
    """Represents one background model download run."""

    id: str
    selected_models: List[str]
    qwen_source: str
    force: bool
    proxy_mode: str
    proxy_url: str
    status: str
    progress: int
    stage: str
    created_at: datetime
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    error: Optional[str] = None
    returncode: Optional[int] = None
    logs: List[dict] = field(default_factory=list)
    process: Optional[subprocess.Popen] = None
    thread: Optional[Any] = None
    stop_event: Optional[Any] = None
    completed_models: set[str] = field(default_factory=set)
    active_model: Optional[str] = None


class ModelDownloadManager:
    """Runs download_models.py in a cancellable background process."""

    def __init__(self):
        import threading

        self.tasks: Dict[str, ModelDownloadTask] = {}
        self.active_task_id: Optional[str] = None
        self._lock = threading.Lock()
        self.threading = threading

    def model_statuses(self) -> List[dict]:
        """Return the current install status for all known models."""
        statuses = []
        for model in GITHUB_MODELS:
            statuses.append(
                {
                    "id": model.name,
                    "name": MODEL_LABELS[model.name],
                    "role": MODEL_ROLES_ZH[model.name],
                    "description": MODEL_DESCRIPTIONS[model.name],
                    "target_path": _relative(model.target),
                    "marker": model.marker,
                    "source": f"GitHub release {model.tag}",
                    "installed": target_has_model(model),
                    "required": True,
                }
            )

        statuses.append(
            {
                "id": "qwen",
                "name": MODEL_LABELS["qwen"],
                "role": MODEL_ROLES_ZH["qwen"],
                "description": MODEL_DESCRIPTIONS["qwen"],
                "target_path": _relative(QWEN_LOCAL_DIR),
                "marker": "*.safetensors / *.bin",
                "source": f"ModelScope / Hugging Face: {QWEN_MODEL_ID}",
                "installed": qwen_has_weights(QWEN_LOCAL_DIR),
                "required": True,
            }
        )
        return statuses

    def get_task(self, task_id: str) -> Optional[ModelDownloadTask]:
        with self._lock:
            return self.tasks.get(task_id)

    def active_task(self) -> Optional[ModelDownloadTask]:
        with self._lock:
            if not self.active_task_id:
                return None
            task = self.tasks.get(self.active_task_id)
            if task and task.status in {"pending", "running", "stopping"}:
                return task
            return None

    def create_task(
        self,
        selected_models: List[str],
        qwen_source: str,
        force: bool,
        proxy_mode: str = "system",
        proxy_url: str = "",
    ) -> ModelDownloadTask:
        task = ModelDownloadTask(
            id=str(uuid.uuid4()),
            selected_models=list(selected_models),
            qwen_source=qwen_source,
            force=force,
            proxy_mode=proxy_mode,
            proxy_url=proxy_url.strip(),
            status="pending",
            progress=0,
            stage="queued",
            created_at=datetime.now(),
            stop_event=self.threading.Event(),
        )
        with self._lock:
            self.tasks[task.id] = task
        return task

    def start_task(
        self,
        selected_models: List[str],
        qwen_source: str,
        force: bool,
        proxy_mode: str,
        proxy_url: str,
        socketio_instance,
    ) -> ModelDownloadTask:
        """Create and start a new download task.

        Raises RuntimeError if another download is currently active.
        """
        with self._lock:
            active = self.active_task_id and self.tasks.get(self.active_task_id)
            if active and active.status in {"pending", "running", "stopping"}:
                raise RuntimeError("A model download task is already running.")

        task = self.create_task(
            selected_models,
            qwen_source,
            force,
            proxy_mode=proxy_mode,
            proxy_url=proxy_url,
        )

        def run_download_thread():
            self._execute_download(task, socketio_instance)

        task.thread = self.threading.Thread(
            target=run_download_thread,
            daemon=True,
            name=f"ModelDownload-{task.id[:8]}",
        )

        with self._lock:
            self.active_task_id = task.id
            task.status = "running"
            task.started_at = datetime.now()

        task.thread.start()
        return task

    def stop_task(self, task_id: str) -> bool:
        task = self.get_task(task_id)
        if not task or task.status not in {"pending", "running"}:
            return False
        task.status = "stopping"
        if task.stop_event:
            task.stop_event.set()
        process = task.process
        if process and process.poll() is None:
            try:
                process.terminate()
            except OSError:
                return False
        return True

    def serialize_task(self, task: ModelDownloadTask) -> dict:
        return {
            "task_id": task.id,
            "task_type": "model_download",
            "status": task.status,
            "progress": task.progress,
            "stage": task.stage,
            "selected_models": task.selected_models,
            "qwen_source": task.qwen_source,
            "force": task.force,
            "proxy_mode": task.proxy_mode,
            "proxy_url": _redact_proxy_url(task.proxy_url),
            "created_at": task.created_at.isoformat(),
            "started_at": task.started_at.isoformat() if task.started_at else None,
            "completed_at": task.completed_at.isoformat() if task.completed_at else None,
            "error": task.error,
            "returncode": task.returncode,
            "logs": task.logs,
        }

    def _execute_download(self, task: ModelDownloadTask, socketio) -> None:
        try:
            command = self._build_command(task)
            self._emit_log(task, socketio, "准备下载模型...", "info")
            self._emit_log(task, socketio, " ".join(command), "info")
            self._emit_progress(task, socketio, 2, "starting")

            env = self._build_process_env(task)
            process = subprocess.Popen(
                command,
                cwd=str(ROOT_DIR),
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=0,
                env=env,
            )
            task.process = process
            self._read_process_output(task, socketio, process)

            if task.stop_event and task.stop_event.is_set():
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                task.status = "cancelled"
                task.stage = "cancelled"
                task.completed_at = datetime.now()
                self._emit_log(task, socketio, "下载任务已停止", "warning")
                self._emit_status(task, socketio)
                return

            task.returncode = process.wait()
            task.completed_at = datetime.now()
            if task.returncode == 0:
                task.status = "completed"
                task.stage = "done"
                task.progress = 100
                self._emit_progress(task, socketio, 100, "done")
                self._emit_log(task, socketio, "模型下载完成", "success")
            else:
                task.status = "failed"
                task.stage = "failed"
                task.error = f"download_models.py exited with code {task.returncode}"
                self._emit_log(task, socketio, task.error, "error")

            self._emit_status(task, socketio)
        except Exception as exc:
            task.status = "failed"
            task.stage = "failed"
            task.error = str(exc)
            task.completed_at = datetime.now()
            self._emit_log(task, socketio, str(exc), "error")
            self._emit_log(task, socketio, traceback.format_exc(), "error")
            self._emit_status(task, socketio)
        finally:
            with self._lock:
                if self.active_task_id == task.id:
                    self.active_task_id = None

    def _build_command(self, task: ModelDownloadTask) -> List[str]:
        script = pathlib.Path(ROOT_DIR) / "download_models.py"
        command = [sys.executable, str(script)]
        for model_id in task.selected_models:
            command.extend(["--only", model_id])
        if "qwen" in task.selected_models:
            command.extend(["--qwen-source", task.qwen_source])
        if task.force:
            command.append("--force")
        return command

    def _build_process_env(self, task: ModelDownloadTask) -> dict:
        """Build the subprocess environment with proxy overrides applied."""
        env = os.environ.copy()
        env["PYTHONUNBUFFERED"] = "1"

        if task.proxy_mode == "system":
            return env

        for key in PROXY_ENV_KEYS:
            env.pop(key, None)

        if task.proxy_mode == "manual":
            proxy_url = task.proxy_url.strip()
            env["HTTP_PROXY"] = proxy_url
            env["HTTPS_PROXY"] = proxy_url
            env["ALL_PROXY"] = proxy_url
            env["http_proxy"] = proxy_url
            env["https_proxy"] = proxy_url
            env["all_proxy"] = proxy_url

        return env

    def _read_process_output(
        self, task: ModelDownloadTask, socketio, process: subprocess.Popen
    ) -> None:
        if process.stdout is None:
            return

        buffer = ""
        while True:
            if task.stop_event and task.stop_event.is_set():
                break
            char = process.stdout.read(1)
            if char == "" and process.poll() is not None:
                break
            if not char:
                continue
            if char in {"\n", "\r"}:
                line = buffer.strip()
                buffer = ""
                if line:
                    self._handle_output_line(task, socketio, line)
            else:
                buffer += char

        line = buffer.strip()
        if line:
            self._handle_output_line(task, socketio, line)

    def _handle_output_line(self, task: ModelDownloadTask, socketio, line: str) -> None:
        level = "error" if "failed" in line.lower() or "error" in line.lower() else "info"
        if "ready" in line.lower() or "already" in line.lower():
            level = "success"
        self._emit_log(task, socketio, line, level)

        active_model = self._guess_model_from_line(task.selected_models, line)
        if active_model:
            task.active_model = active_model

        pct_match = re.search(r"\b(\d{1,3})%\b", line)
        if pct_match and task.active_model:
            pct = min(100, max(0, int(pct_match.group(1))))
            self._emit_progress_for_model(task, socketio, task.active_model, pct)

        if "ready" in line.lower() or "already present" in line.lower():
            completed = active_model or task.active_model
            if completed:
                task.completed_models.add(completed)
                self._emit_progress_for_model(task, socketio, completed, 100)

    def _emit_progress_for_model(
        self, task: ModelDownloadTask, socketio, model_id: str, model_pct: int
    ) -> None:
        total = max(1, len(task.selected_models))
        try:
            index = task.selected_models.index(model_id)
        except ValueError:
            index = len(task.completed_models)
        progress = int(((index + model_pct / 100.0) / total) * 100)
        progress = min(99, max(task.progress, progress))
        self._emit_progress(task, socketio, progress, MODEL_LABELS.get(model_id, "downloading"))

    def _guess_model_from_line(self, selected_models: List[str], line: str) -> Optional[str]:
        lowered = line.lower()
        github_by_name = {model.name: model for model in GITHUB_MODELS}
        for model_id in selected_models:
            if model_id == "qwen":
                if "qwen" in lowered or str(QWEN_LOCAL_DIR.name).lower() in lowered:
                    return "qwen"
                continue
            model = github_by_name.get(model_id)
            if not model:
                continue
            needles = [model.name, model.asset, model.target.name]
            if any(needle.lower() in lowered for needle in needles):
                return model_id
        return None

    def _emit_log(self, task: ModelDownloadTask, socketio, message: str, level: str) -> None:
        entry = {
            "task_id": task.id,
            "task_type": "model_download",
            "message": message,
            "level": level,
            "timestamp": datetime.now().isoformat(),
        }
        task.logs.append(entry)
        if len(task.logs) > 500:
            task.logs = task.logs[-500:]
        try:
            socketio.emit("log", entry, room=task.id)
        except Exception as exc:
            print(f"[WebSocket Error] Failed to emit model download log: {exc}")

    def _emit_progress(self, task: ModelDownloadTask, socketio, progress: int, stage: str) -> None:
        task.progress = min(100, max(0, int(progress)))
        task.stage = stage
        payload = {
            "task_id": task.id,
            "task_type": "model_download",
            "progress": task.progress,
            "stage": task.stage,
        }
        try:
            socketio.emit("progress", payload, room=task.id)
        except Exception as exc:
            print(f"[WebSocket Error] Failed to emit model download progress: {exc}")

    def _emit_status(self, task: ModelDownloadTask, socketio) -> None:
        payload = self.serialize_task(task)
        try:
            socketio.emit("status_change", payload, room=task.id)
        except Exception as exc:
            print(f"[WebSocket Error] Failed to emit model download status: {exc}")


def validate_model_request(
    model_ids: List[str],
    qwen_source: str,
    proxy_mode: str = "system",
    proxy_url: str = "",
) -> Optional[str]:
    unknown = [model_id for model_id in model_ids if model_id not in VALID_MODEL_IDS]
    if unknown:
        return "Unknown model id(s): " + ", ".join(unknown)
    if qwen_source not in VALID_QWEN_SOURCES:
        return "Invalid qwen_source. Expected one of: auto, modelscope, huggingface."
    if proxy_mode not in VALID_PROXY_MODES:
        return "Invalid proxy_mode. Expected one of: system, manual, none."
    if proxy_mode == "manual":
        proxy_url = proxy_url.strip()
        if not proxy_url:
            return "proxy_url is required when proxy_mode is manual."
        if "://" not in proxy_url:
            return "proxy_url must include a scheme, for example http://127.0.0.1:7890."
    return None


def _relative(path: pathlib.Path) -> str:
    try:
        return str(path.relative_to(ROOT_DIR))
    except ValueError:
        return str(path)


def _redact_proxy_url(proxy_url: str) -> str:
    if not proxy_url:
        return ""
    if "@" not in proxy_url:
        return proxy_url
    scheme, rest = proxy_url.split("://", 1) if "://" in proxy_url else ("", proxy_url)
    host = rest.rsplit("@", 1)[-1]
    return f"{scheme}://***@{host}" if scheme else f"***@{host}"
