"""Task Manager for Vocal2Midi Web UI.

Manages pipeline execution lifecycle, replacing PyQt5's QThread-based
WorkerThread with a threading + WebSocket approach.
"""

import json
import os
import sys
import tempfile
import traceback
import uuid
import pathlib
from dataclasses import dataclass, field
from datetime import datetime
from typing import Dict, Optional, List, Any, Callable

from application.config import (
    PipelineConfig,
    validate_slice_bounds,
    DEFAULT_SLICE_MIN_SEC,
    DEFAULT_SLICE_MAX_SEC,
)
from inference.device_utils import normalize_runtime_device


@dataclass
class Task:
    """Represents a single pipeline execution task."""

    id: str
    status: str  # pending, running, completed, failed, cancelled
    progress: int  # 0-100
    stage: str  # idle, loading, slicing, asr, alignment, export, done
    config: dict
    audio_file_path: str
    created_at: datetime
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    error: Optional[str] = None
    output_files: List[str] = field(default_factory=list)
    thread: Optional[Any] = None  # threading.Thread
    stop_event: Optional[Any] = None  # threading.Event
    logs: List[dict] = field(default_factory=list)


class TaskManager:
    """Manages pipeline tasks with thread-safe operations and WebSocket integration."""

    def __init__(self):
        # Import threading first
        import threading
        
        self.tasks: Dict[str, Task] = {}
        self._lock = threading.Lock()
        self.threading = threading

    def create_task(self, config: dict, audio_path: str) -> str:
        """Create a new task and return its ID.

        Args:
            config: Dictionary of pipeline parameters from frontend
            audio_path: Path to the uploaded audio file

        Returns:
            task_id: UUID string identifying the task
        """
        task_id = str(uuid.uuid4())
        task = Task(
            id=task_id,
            status='pending',
            progress=0,
            stage='idle',
            config=config,
            audio_file_path=audio_path,
            created_at=datetime.now(),
            started_at=None,
            completed_at=None,
            error=None,
            output_files=[],
            thread=None,
            stop_event=self.threading.Event()
        )

        with self._lock:
            self.tasks[task_id] = task

        return task_id

    def start_task(self, task_id: str, socketio_instance) -> bool:
        """Start executing a task in a background thread.

        Args:
            task_id: The task to start
            socketio_instance: Flask-SocketIO instance for emitting events

        Returns:
            True if task was started, False otherwise
        """
        task = self.get_task(task_id)
        if not task or task.status != 'pending':
            return False

        task.status = 'running'
        task.started_at = datetime.now()

        def run_pipeline_thread():
            self._execute_pipeline(task, socketio_instance)

        task.thread = self.threading.Thread(
            target=run_pipeline_thread,
            daemon=True,
            name=f"Pipeline-{task_id[:8]}"
        )
        task.thread.start()
        return True

    def stop_task(self, task_id: str) -> bool:
        """Request stopping a running task.

        Args:
            task_id: The task to stop

        Returns:
            True if stop was requested, False otherwise
        """
        task = self.get_task(task_id)
        if not task or task.status != 'running':
            return False
        task.stop_event.set()
        return True

    def get_task(self, task_id: str) -> Optional[Task]:
        """Get a task by ID (thread-safe).

        Args:
            task_id: The task ID to look up

        Returns:
            Task object or None if not found
        """
        with self._lock:
            return self.tasks.get(task_id)

    def list_tasks(self) -> List[dict]:
        """List all tasks with their current status.

        Returns:
            List of task summary dictionaries
        """
        with self._lock:
            return [
                {
                    'id': t.id,
                    'status': t.status,
                    'progress': t.progress,
                    'stage': t.stage,
                    'created_at': t.created_at.isoformat(),
                    'started_at': t.started_at.isoformat() if t.started_at else None,
                    'completed_at': t.completed_at.isoformat() if t.completed_at else None,
                }
                for t in self.tasks.values()
            ]

    def _execute_pipeline(self, task: Task, socketio):
        """Execute the actual pipeline in a background thread.

        This replaces the run() method in gui/fluent_worker.py's WorkerThread.
        Instead of emitting Qt Signals, it emits SocketIO events.

        Args:
            task: The task to execute
            socketio: Flask-SocketIO instance for real-time updates
        """
        try:
            # Define callback functions for real-time logging
            def log_callback(message: str, level: str = 'info'):
                log_entry = {
                    'task_id': task.id,
                    'message': message,
                    'level': level,
                    'timestamp': datetime.now().isoformat()
                }
                task.logs.append(log_entry)
                try:
                    socketio.emit('log', log_entry, room=task.id)
                except Exception as e:
                    print(f"[WebSocket Error] Failed to emit log: {e}")

            def progress_callback(progress: int, stage: str):
                task.progress = progress
                task.stage = stage
                try:
                    socketio.emit('progress', {
                        'task_id': task.id,
                        'progress': progress,
                        'stage': stage
                    }, room=task.id)
                except Exception as e:
                    print(f"[WebSocket Error] Failed to emit progress: {e}")

            # Build PipelineConfig from frontend parameters
            log_callback('正在构建配置参数...', 'info')
            config = self._build_config(task.config, task.audio_file_path)

            # Set cancel checker
            config.cancel_checker = lambda: task.stop_event.is_set()

            # Redirect stdout/stderr to WebSocket
            from web_stream_redirector import WebStreamRedirector
            old_stdout = sys.stdout
            old_stderr = sys.stderr
            sys.stdout = WebStreamRedirector(old_stdout, log_callback)
            sys.stderr = WebStreamRedirector(old_stderr, log_callback)

            try:
                # Import and run the pipeline
                log_callback('=== 开始全自动提取流程 ===', 'success')
                log_callback(f'处理文件: {pathlib.Path(task.audio_file_path).name}', 'info')
                log_callback(f'目标语言: {config.language}', 'info')
                log_callback(f'计算设备: {config.device}', 'info')
                log_callback(f'保存目录: {str(config.output_dir)}', 'info')
                log_callback('', 'info')

                # Stage 1: Loading models
                progress_callback(5, 'loading')
                log_callback('正在加载模型...', 'info')

                from application.pipeline import run_auto_lyric_job
                run_auto_lyric_job(config)

                # Check if completed successfully
                if not task.stop_event.is_set():
                    task.status = 'completed'
                    task.progress = 100
                    task.stage = 'done'
                    task.completed_at = datetime.now()

                    # Collect output files
                    output_dir = config.output_dir
                    if output_dir.exists():
                        for ext in ['*.mid', '*.ustx', '*.txt', '*.csv']:
                            task.output_files.extend([
                                str(f) for f in output_dir.glob(ext)
                            ])

                    log_callback('', 'info')
                    log_callback('=============================', 'info')
                    log_callback('✓ 全自动提取完成！', 'success')
                    log_callback(f'输出目录: {output_dir}', 'success')

                    socketio.emit('status_change', {
                        'task_id': task.id,
                        'status': 'completed',
                        'result': {
                            'output_dir': str(output_dir),
                            'files': task.output_files
                        }
                    }, room=task.id)
                else:
                    task.status = 'cancelled'
                    log_callback('', 'warning')
                    log_callback('⚠ 任务已被用户取消', 'warning')

                    socketio.emit('status_change', {
                        'task_id': task.id,
                        'status': 'cancelled'
                    }, room=task.id)

            except KeyboardInterrupt:
                task.status = 'cancelled'
                log_callback('任务被中断', 'warning')

                socketio.emit('status_change', {
                    'task_id': task.id,
                    'status': 'cancelled'
                }, room=task.id)

            except Exception as e:
                # Distinguish between cancellation and actual errors
                if task.stop_event.is_set():
                    task.status = 'cancelled'
                    log_callback(f'任务已被停止', 'warning')
                else:
                    task.status = 'failed'
                    task.error = str(e)
                    log_callback(f'发生错误:', 'error')
                    log_callback(str(e), 'error')
                    log_callback('', 'error')
                    log_callback(traceback.format_exc(), 'error')

                socketio.emit('status_change', {
                    'task_id': task.id,
                    'status': task.status,
                    'error': task.error
                }, room=task.id)

            finally:
                # Restore original stdout/stderr
                sys.stdout = old_stdout
                sys.stderr = old_stderr

        except Exception as e:
            # Catch any errors in the task manager itself
            task.status = 'failed'
            task.error = f"Task manager error: {str(e)}"
            print(f"[Task Manager Error] {traceback.format_exc()}")

            try:
                socketio.emit('status_change', {
                    'task_id': task.id,
                    'status': 'failed',
                    'error': task.error
                }, room=task.id)
            except Exception:
                pass  # If even this fails, just give up

    def _build_config(self, frontend_config: dict, audio_path: str) -> PipelineConfig:
        """Build a PipelineConfig object from frontend form data.

        Maps the JSON configuration from the web UI to the PipelineConfig
        dataclass expected by the inference pipeline.

        Args:
            frontend_config: Dictionary of parameters from the frontend
            audio_path: Path to the uploaded audio file

        Returns:
            Configured PipelineConfig object
        """
        # Extract values with defaults
        slicing_method = frontend_config.get('slicing_method', 'auto')
        language = frontend_config.get('language', 'zh')
        device_raw = frontend_config.get('device', 'cpu')
        device = normalize_runtime_device(device_raw)

        tempo = float(frontend_config.get('tempo', 120))
        save_dir = pathlib.Path(frontend_config.get('save_dir', './output'))

        # Ensure output directory exists
        save_dir.mkdir(parents=True, exist_ok=True)

        # Lyric options
        lyric_output_mode = frontend_config.get('lyric_output_mode', 'auto')
        enable_lyrics_match = frontend_config.get('enable_lyrics_match', False)
        output_lyrics = frontend_config.get('output_lyrics', True)
        lyrics_text = frontend_config.get('lyrics', '')

        # Export options
        export_ustx = frontend_config.get('export_ustx', False)
        output_pitch_curve = frontend_config.get('output_pitch_curve', False) if export_ustx else False

        # Debug options
        debug_txt = frontend_config.get('debug_txt', False)
        debug_csv = frontend_config.get('debug_csv', False)
        debug_chunks = frontend_config.get('debug_chunks', False)
        pitch_format = frontend_config.get('pitch_format', 'name')
        round_pitch = frontend_config.get('round_pitch', True)

        # Advanced params
        seg_threshold = float(frontend_config.get('seg_threshold', 0.2))
        seg_radius = float(frontend_config.get('seg_radius', 0.02))
        est_threshold = float(frontend_config.get('est_threshold', 0.2))
        t0_value = float(frontend_config.get('t0', 0.0))
        nsteps_value = int(frontend_config.get('nsteps', 8))
        game_batch = int(frontend_config.get('game_batch', 1))
        asr_batch = int(frontend_config.get('asr_batch', 2))

        slice_min = float(frontend_config.get('slice_min', DEFAULT_SLICE_MIN_SEC))
        slice_max = float(frontend_config.get('slice_max', DEFAULT_SLICE_MAX_SEC))

        # Validate slice bounds
        try:
            validate_slice_bounds(slice_min, slice_max)
        except ValueError as e:
            print(f"[Warning] Invalid slice bounds: {e}, using defaults")
            slice_min = DEFAULT_SLICE_MIN_SEC
            slice_max = DEFAULT_SLICE_MAX_SEC

        # Build output formats list
        output_formats = ['mid']
        if debug_txt:
            output_formats.append('txt')
        if debug_csv:
            output_formats.append('csv')
        if debug_chunks:
            output_formats.append('chunks')
        if export_ustx:
            output_formats.append('ustx')

        # Build timestamp list (simplified - would normally come from UI)
        ts_list = [float(t0_value + i * (1.0 - t0_value) / nsteps_value) for i in range(nsteps_value)]

        # Determine output filename
        audio_filename = pathlib.Path(audio_path).name
        output_filename = pathlib.Path(audio_filename).stem

        # Create and return PipelineConfig
        config = PipelineConfig(
            audio_path=audio_path,
            output_filename=output_filename,
            output_dir=save_dir,
            # Model paths will be set from settings
            game_model_dir=frontend_config.get('game_model_path', ''),
            hfa_model_dir=frontend_config.get('hfa_model_path', ''),
            asr_model_path=frontend_config.get('asr_model_path', ''),
            device=device,
            language=language,
            ts=ts_list,
            # Options
            lyric_output_mode=lyric_output_mode,
            original_lyrics=lyrics_text if enable_lyrics_match else '',
            output_formats=output_formats,
            slicing_method=slicing_method,
            slice_min_sec=slice_min,
            slice_max_sec=slice_max,
            tempo=tempo,
            pitch_format=pitch_format,
            round_pitch=round_pitch,
            seg_threshold=seg_threshold,
            seg_radius=seg_radius,
            est_threshold=est_threshold,
            batch_size=game_batch,
            asr_batch_size=asr_batch,
            output_lyrics=output_lyrics,
            rmvpe_model_path=frontend_config.get('rmvpe_model_path', ''),
            phoneme_asr_model_path=frontend_config.get('phoneme_asr_model_path', ''),
            output_pitch_curve=output_pitch_curve,
        )

        return config
