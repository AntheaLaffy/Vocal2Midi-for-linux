"""Vocal2Midi Web Server - Flask + SocketIO backend.

Provides REST API and real-time WebSocket communication for the
web-based Vocal2MIDI UI.

Usage:
    python web_server.py
    V2M_WEB_PORT=5001 python web_server.py

    # Or with gunicorn for production:
    gunicorn --worker-class eventlet -w 1 -b 0.0.0.0:5000 web_server:app
"""

import os
import sys
import json
import tempfile
import pathlib
import copy
import traceback
import re
import string
from datetime import datetime
from typing import List, Optional

from flask import Flask, request, jsonify, send_from_directory, send_file
from flask_socketio import SocketIO, emit, join_room, leave_room, rooms
from flask_cors import CORS

# Add project root to path for imports
PROJECT_ROOT = pathlib.Path(__file__).resolve().parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# Import task manager
from web_task_manager import TaskManager
from web_model_download_manager import (
    ModelDownloadManager,
    validate_model_request,
)

# Initialize Flask app
app = Flask(
    __name__,
    static_folder='.',
    static_url_path=''
)
app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'vocal2midi-web-secret-key-2024')
app.config['MAX_CONTENT_LENGTH'] = 500 * 1024 * 1024  # 500MB max file size

# Enable CORS for all routes
CORS(app)

# Initialize SocketIO with threading mode (required for ONNX Runtime compatibility)
socketio = SocketIO(
    app,
    cors_allowed_origins="*",
    async_mode='threading',
    ping_timeout=300,
    ping_interval=60,
)

# Global task manager instance
task_manager = TaskManager()
model_download_manager = ModelDownloadManager()

# Default settings (matching GlobalSettingsInterface defaults)
DEFAULT_SETTINGS = {
    'models': {
        'game_model_path': 'experiments/GAME-1.0.3-medium-onnx',
        'hfa_model_path': 'experiments/1218_hfa_model_new_dict',
        'asr_model_path': 'experiments/Qwen3-ASR-1.7B',
        'phoneme_asr_model_path': 'experiments/romajiASR',
        'rmvpe_model_path': 'experiments/RMVPE/rmvpe.onnx'
    },
    'params': {
        'seg_threshold': 0.2,
        'seg_radius': 0.02,
        'est_threshold': 0.2,
        't0': 0.0,
        'nsteps': 8,
        'game_batch': 1,
        'asr_batch': 2,
        'slice_min': 8.0,
        'slice_max': 22.0
    },
    'debug': {
        'export_txt': False,
        'export_csv': False,
        'export_chunks': False,
        'pitch_format': 'name',
        'round_pitch': True
    },
    'pipeline': {
        'slicing_method': 'auto',
        'language': 'zh',
        'lyric_output_mode': 'pinyin',
        'device': 'dml',
        'tempo': 120,
        'save_dir': './output',
        'quantize_precision': 'none',
        'quantize_algorithm': 'dev',
        'enable_lyrics_match': False,
        'output_lyrics': True,
        'export_ustx': False,
        'output_pitch_curve': True
    },
    'downloads': {
        'qwen_source': 'auto',
        'proxy_mode': 'system',
        'proxy_url': '',
        'force': False
    }
}

SETTINGS_FILE = pathlib.Path(
    os.environ.get('V2M_WEB_SETTINGS_FILE', PROJECT_ROOT / 'settings' / 'web_settings.json')
)


def _merge_settings(defaults: dict, overrides: dict) -> dict:
    """Merge a settings file over defaults while keeping unknown keys out."""
    merged = copy.deepcopy(defaults)
    if not isinstance(overrides, dict):
        return merged

    for section, default_value in defaults.items():
        override_value = overrides.get(section)
        if isinstance(default_value, dict):
            if isinstance(override_value, dict):
                merged[section].update(override_value)
        elif override_value is not None:
            merged[section] = override_value
    return merged


def _load_settings_from_disk() -> dict:
    """Load persisted Web settings, falling back to defaults on any problem."""
    if not SETTINGS_FILE.is_file():
        return copy.deepcopy(DEFAULT_SETTINGS)

    try:
        payload = json.loads(SETTINGS_FILE.read_text(encoding='utf-8'))
    except (OSError, ValueError) as e:
        print(f"[Warning] Failed to load web settings from {SETTINGS_FILE}: {e}")
        return copy.deepcopy(DEFAULT_SETTINGS)
    return _merge_settings(DEFAULT_SETTINGS, payload)


def _save_settings_to_disk(settings: dict) -> None:
    """Persist settings atomically so a crash does not leave a half-written file."""
    SETTINGS_FILE.parent.mkdir(parents=True, exist_ok=True)
    temp_path = SETTINGS_FILE.with_name(f"{SETTINGS_FILE.name}.tmp")
    payload = json.dumps(settings, ensure_ascii=False, indent=2) + "\n"
    temp_path.write_text(payload, encoding='utf-8')
    temp_path.replace(SETTINGS_FILE)


# In-memory settings storage backed by settings/web_settings.json.
current_settings = _load_settings_from_disk()


# ==================== Static File Serving ====================

@app.route('/')
def index():
    """Serve the main HTML page."""
    return send_from_directory('.', 'Vocal2Midi Web.html')


@app.route('/<path:path>')
def serve_static(path):
    """Serve static files (CSS, JS, images, etc.)."""
    if pathlib.Path(path).suffix in ['.html', '.css', '.js', '.png', '.ico', '.svg']:
        return send_from_directory('.', path)
    return f'File not found: {path}', 404


# ==================== Pipeline API Routes ====================

@app.route('/api/pipeline/start', methods=['POST'])
def start_pipeline():
    """Start a new pipeline processing task.

    Expects multipart/form-data with:
    - audio_file: The audio file to process
    - config: JSON string of pipeline parameters

    Returns:
        JSON with task_id and initial status
    """
    try:
        # Check for audio file
        if 'audio_file' not in request.files:
            return jsonify({
                'success': False,
                'error': 'No audio file provided. Please select an audio file.'
            }), 400

        audio_file = request.files['audio_file']
        if not audio_file.filename:
            return jsonify({
                'success': False,
                'error': 'No selected file.'
            }), 400

        # Validate file type
        allowed_extensions = {'.wav', '.m4a', '.flac', '.mp3', '.ogg'}
        ext = pathlib.Path(audio_file.filename).suffix.lower()
        if ext not in allowed_extensions:
            return jsonify({
                'success': False,
                'error': f'Invalid file format: {ext}. Allowed formats: {", ".join(allowed_extensions)}'
            }), 400

        # Save uploaded file to temporary directory
        temp_dir = tempfile.mkdtemp(prefix='vocal2midi_upload_')
        audio_path = os.path.join(temp_dir, audio_file.filename)
        audio_file.save(audio_path)

        # Parse configuration JSON
        config_json = request.form.get('config', '{}')
        try:
            config = json.loads(config_json)
        except json.JSONDecodeError as e:
            # Clean up temp file
            import shutil
            shutil.rmtree(temp_dir, ignore_errors=True)
            return jsonify({
                'success': False,
                'error': f'Invalid configuration JSON: {str(e)}'
            }), 400

        # Merge with persisted settings first, then let request config override
        # per-run options from the extract page.
        settings_debug = {
            'debug_txt': current_settings['debug'].get('export_txt', False),
            'debug_csv': current_settings['debug'].get('export_csv', False),
            'debug_chunks': current_settings['debug'].get('export_chunks', False),
            'pitch_format': current_settings['debug'].get('pitch_format', 'name'),
            'round_pitch': current_settings['debug'].get('round_pitch', True),
        }
        merged_config = {
            **current_settings['pipeline'],
            **current_settings['models'],
            **current_settings['params'],
            **settings_debug,
            **config,
        }

        # Create and start task
        task_id = task_manager.create_task(merged_config, audio_path)
        success = task_manager.start_task(task_id, socketio)

        if not success:
            return jsonify({
                'success': False,
                'error': 'Failed to start task. It may already be running or in an invalid state.'
            }), 500

        return jsonify({
            'success': True,
            'task_id': task_id,
            'status': 'running',
            'message': 'Task started successfully. Processing will begin shortly...'
        })

    except Exception as e:
        print(f"[Error] Failed to start pipeline: {traceback.format_exc()}")
        return jsonify({
            'success': False,
            'error': f'Server error: {str(e)}'
        }), 500


@app.route('/api/pipeline/stop', methods=['POST'])
def stop_pipeline():
    """Stop a running pipeline task.

    Expects JSON body:
    - task_id: The task to stop

    Returns:
        JSON with stop status
    """
    data = request.get_json(silent=True) or {}
    task_id = data.get('task_id')

    if not task_id:
        return jsonify({
            'success': False,
            'error': 'Missing task_id parameter'
        }), 400

    success = task_manager.stop_task(task_id)

    if not success:
        task = task_manager.get_task(task_id)
        if not task:
            return jsonify({
                'success': False,
                'error': 'Task not found'
            }), 404
        else:
            return jsonify({
                'success': False,
                'error': f'Task cannot be stopped (current status: {task.status})'
            }), 400

    return jsonify({
        'success': True,
        'status': 'stopping',
        'message': 'Stop request sent. Task will finish current operation then stop.'
    })


@app.route('/api/pipeline/status/<task_id>')
def get_task_status(task_id):
    """Get the status of a specific task.

    Args:
        task_id: The task ID to query

    Returns:
        JSON with full task status information
    """
    task = task_manager.get_task(task_id)

    if not task:
        return jsonify({
            'success': False,
            'error': 'Task not found'
        }), 404

    return jsonify({
        'success': True,
        'task_id': task.id,
        'status': task.status,
        'progress': task.progress,
        'stage': task.stage,
        'created_at': task.created_at.isoformat(),
        'started_at': task.started_at.isoformat() if task.started_at else None,
        'completed_at': task.completed_at.isoformat() if task.completed_at else None,
        'error': task.error,
        'output_files': task.output_files
    })


@app.route('/api/pipeline/list')
def list_tasks():
    """List all tasks.

    Returns:
        JSON array of all tasks with summary info
    """
    tasks = task_manager.list_tasks()
    return jsonify({
        'success': True,
        'tasks': tasks,
        'count': len(tasks)
    })


# ==================== Settings API Routes ====================

@app.route('/api/settings', methods=['GET'])
def get_settings():
    """Get current settings.

    Returns:
        JSON with models, params, and debug settings
    """
    return jsonify({
        'success': True,
        **current_settings
    })


@app.route('/api/settings', methods=['PUT'])
def update_settings():
    """Update settings.

    Expects JSON body with any combination of:
    - models: Model path configurations
    - params: Advanced parameters
    - debug: Debug options

    Returns:
        JSON with update status
    """
    data = request.get_json(silent=True)

    # Validate JSON parsing
    if data is None:
        return jsonify({
            'success': False,
            'error': 'Invalid JSON in request body'
        }), 400

    try:
        # Update each known section if provided.
        for section in DEFAULT_SETTINGS:
            if section in data:
                if not isinstance(data[section], dict):
                    return jsonify({
                        'success': False,
                        'error': f'{section} must be an object'
                    }), 400
                current_settings[section].update(data[section])

        _save_settings_to_disk(current_settings)

        return jsonify({
            'success': True,
            'message': 'Settings updated successfully',
            'settings': current_settings
        })

    except Exception as e:
        return jsonify({
            'success': False,
            'error': f'Failed to update settings: {str(e)}'
        }), 500


@app.route('/api/settings/reset', methods=['POST'])
def reset_settings():
    """Reset settings to defaults.

    Returns:
        JSON with reset status
    """
    global current_settings
    # Use deepcopy to ensure complete isolation from DEFAULT_SETTINGS
    current_settings = copy.deepcopy(DEFAULT_SETTINGS)
    _save_settings_to_disk(current_settings)

    return jsonify({
        'success': True,
        'message': 'Settings reset to defaults',
        'settings': current_settings
    })


# ==================== Filesystem Browser API ====================

def _resolve_picker_path(path_text: str) -> pathlib.Path:
    """Resolve a browser picker path against the project root."""
    text = (path_text or "").strip()
    if not text:
        return PROJECT_ROOT

    expanded = pathlib.Path(os.path.expanduser(text))
    if not expanded.is_absolute():
        expanded = PROJECT_ROOT / expanded
    return expanded.resolve()


def _input_value_for_path(path: pathlib.Path) -> str:
    """Prefer project-relative paths for values written back to the UI."""
    resolved = path.resolve()
    try:
        rel = resolved.relative_to(PROJECT_ROOT)
    except ValueError:
        return str(resolved)
    return "." if str(rel) == "." else rel.as_posix()


def _filesystem_root_entry(label: str, path: pathlib.Path) -> Optional[dict]:
    try:
        resolved = path.resolve()
    except OSError:
        return None
    if not resolved.exists() or not resolved.is_dir():
        return None
    return {
        "label": label,
        "path": str(resolved),
        "input_path": _input_value_for_path(resolved),
    }


def _filesystem_roots() -> List[dict]:
    roots: List[dict] = []
    seen: set[str] = set()

    candidates = [
        ("项目目录", PROJECT_ROOT),
        ("用户目录", pathlib.Path.home()),
    ]
    if os.name == "nt":
        for drive in string.ascii_uppercase:
            candidates.append((f"{drive}:\\", pathlib.Path(f"{drive}:\\")))
    else:
        candidates.append(("系统根目录", pathlib.Path("/")))

    for label, path in candidates:
        entry = _filesystem_root_entry(label, path)
        if not entry or entry["path"] in seen:
            continue
        seen.add(entry["path"])
        roots.append(entry)
    return roots


def _parse_extensions(raw_extensions: str) -> set[str]:
    extensions = set()
    for item in (raw_extensions or "").split(","):
        ext = item.strip().lower()
        if not ext:
            continue
        extensions.add(ext if ext.startswith(".") else f".{ext}")
    return extensions


def _filesystem_entry(entry: os.DirEntry, mode: str, extensions: set[str]) -> Optional[dict]:
    try:
        is_dir = entry.is_dir(follow_symlinks=False)
        is_file = entry.is_file(follow_symlinks=False)
    except OSError:
        return None

    if not is_dir and not (mode == "file" and is_file):
        return None

    path = pathlib.Path(entry.path)
    if is_file and extensions and path.suffix.lower() not in extensions:
        return None

    return {
        "name": entry.name,
        "type": "directory" if is_dir else "file",
        "path": str(path.resolve()),
        "input_path": _input_value_for_path(path),
    }


@app.route('/api/filesystem/roots')
def filesystem_roots():
    """Return useful filesystem roots for the local path picker."""
    return jsonify({
        'success': True,
        'separator': os.sep,
        'roots': _filesystem_roots(),
    })


@app.route('/api/filesystem/list')
def filesystem_list():
    """List child directories or model files for the local path picker."""
    mode = request.args.get('mode', 'directory')
    if mode not in {'directory', 'file'}:
        return jsonify({
            'success': False,
            'error': 'mode must be directory or file'
        }), 400

    current_path = _resolve_picker_path(request.args.get('path', ''))
    if not current_path.exists():
        return jsonify({
            'success': False,
            'error': 'Path does not exist'
        }), 404
    if not current_path.is_dir():
        current_path = current_path.parent

    extensions = _parse_extensions(request.args.get('extensions', ''))
    entries = []
    try:
        with os.scandir(current_path) as iterator:
            for entry in iterator:
                item = _filesystem_entry(entry, mode, extensions)
                if item:
                    entries.append(item)
    except OSError as e:
        return jsonify({
            'success': False,
            'error': f'Cannot read directory: {e}'
        }), 400

    entries.sort(key=lambda item: (item['type'] != 'directory', item['name'].lower()))
    parent = current_path.parent if current_path.parent != current_path else None

    return jsonify({
        'success': True,
        'mode': mode,
        'path': str(current_path),
        'input_path': _input_value_for_path(current_path),
        'parent': str(parent) if parent else None,
        'parent_input_path': _input_value_for_path(parent) if parent else None,
        'entries': entries,
        'roots': _filesystem_roots(),
    })


# ==================== System Info API ====================

@app.route('/api/system/info')
def system_info():
    """Get system information.

    Returns:
        JSON with version, device info, and system status
    """
    import platform

    # Try to detect available devices
    available_devices = ['cpu']
    try:
        from inference.device_utils import VISIBLE_RUNTIME_DEVICE_CHOICES
        available_devices = list(VISIBLE_RUNTIME_DEVICE_CHOICES.keys())
    except Exception:
        pass

    return jsonify({
        'success': True,
        'version': '1.0.0-web',
        'python_version': platform.python_version(),
        'platform': platform.system(),
        'device': 'cpu',  # Would detect actual device in production
        'available_devices': available_devices,
        'models_loaded': False,  # Would check in production
        'active_tasks': sum(1 for t in task_manager.tasks.values() if t.status == 'running')
    })


# ==================== Model Download API ====================

@app.route('/api/models/status')
def model_status():
    """Return known model assets and their local install status."""
    models = model_download_manager.model_statuses()
    active_task = model_download_manager.active_task()
    installed_count = sum(1 for model in models if model['installed'])

    return jsonify({
        'success': True,
        'models': models,
        'installed_count': installed_count,
        'missing_count': len(models) - installed_count,
        'active_task': (
            model_download_manager.serialize_task(active_task)
            if active_task else None
        ),
    })


@app.route('/api/models/download', methods=['POST'])
def start_model_download():
    """Start a background model download task."""
    data = request.get_json(silent=True) or {}
    force = bool(data.get('force', False))
    qwen_source = data.get('qwen_source', 'auto')
    proxy_mode = data.get('proxy_mode', 'system')
    proxy_url = str(data.get('proxy_url', '') or '')

    model_ids = data.get('models')
    if model_ids is None:
        # Default web behavior: download every missing model.
        model_ids = [
            model['id']
            for model in model_download_manager.model_statuses()
            if not model['installed']
        ]

    if not isinstance(model_ids, list) or not all(isinstance(m, str) for m in model_ids):
        return jsonify({
            'success': False,
            'error': 'models must be a list of model ids'
        }), 400

    # Deduplicate while preserving UI order.
    selected_models = list(dict.fromkeys(model_ids))
    if not selected_models:
        return jsonify({
            'success': False,
            'error': 'No models selected for download'
        }), 400

    validation_error = validate_model_request(
        selected_models,
        qwen_source,
        proxy_mode=proxy_mode,
        proxy_url=proxy_url,
    )
    if validation_error:
        return jsonify({
            'success': False,
            'error': validation_error
        }), 400

    try:
        task = model_download_manager.start_task(
            selected_models=selected_models,
            qwen_source=qwen_source,
            force=force,
            proxy_mode=proxy_mode,
            proxy_url=proxy_url,
            socketio_instance=socketio,
        )
    except RuntimeError as e:
        return jsonify({
            'success': False,
            'error': str(e)
        }), 409

    return jsonify({
        'success': True,
        'task_id': task.id,
        'status': task.status,
        'message': 'Model download task started',
    })


@app.route('/api/models/download/status/<task_id>')
def get_model_download_status(task_id):
    """Return status for a model download task."""
    task = model_download_manager.get_task(task_id)
    if not task:
        return jsonify({
            'success': False,
            'error': 'Download task not found'
        }), 404

    return jsonify({
        'success': True,
        **model_download_manager.serialize_task(task),
    })


@app.route('/api/models/download/stop', methods=['POST'])
def stop_model_download():
    """Stop a running model download task."""
    data = request.get_json(silent=True) or {}
    task_id = data.get('task_id')
    if not task_id:
        return jsonify({
            'success': False,
            'error': 'Missing task_id parameter'
        }), 400

    if not model_download_manager.get_task(task_id):
        return jsonify({
            'success': False,
            'error': 'Download task not found'
        }), 404

    success = model_download_manager.stop_task(task_id)
    if not success:
        task = model_download_manager.get_task(task_id)
        return jsonify({
            'success': False,
            'error': f'Download task cannot be stopped (current status: {task.status})'
        }), 400

    return jsonify({
        'success': True,
        'status': 'stopping',
        'message': 'Stop request sent'
    })


# ==================== Download API ====================

WINDOWS_DRIVE_RE = re.compile(r"^[a-zA-Z]:[\\/]")


def _safe_requested_download_path(filepath: str) -> pathlib.Path | None:
    """Resolve a URL path to a project-relative file path, rejecting traversal."""
    if not filepath or "\x00" in filepath or "\\" in filepath:
        return None
    if WINDOWS_DRIVE_RE.match(filepath):
        return None

    requested = pathlib.PurePosixPath(filepath)
    if requested.is_absolute() or ".." in requested.parts:
        return None

    return (PROJECT_ROOT / pathlib.Path(*requested.parts)).resolve()


def _authorized_output_file(filepath: str) -> pathlib.Path | None:
    """Return the requested file only if it is registered as a task output."""
    requested_path = _safe_requested_download_path(filepath)
    if requested_path is None or not requested_path.is_file():
        return None

    with task_manager._lock:
        tasks = list(task_manager.tasks.values())

    for task in tasks:
        for output_file in task.output_files:
            output_path = pathlib.Path(output_file)
            if not output_path.is_absolute():
                output_path = PROJECT_ROOT / output_path
            try:
                if output_path.resolve() == requested_path:
                    return requested_path
            except OSError:
                continue
    return None


@app.route('/api/download/<path:filepath>')
def download_file(filepath):
    """Download an output file.

    Args:
        filepath: Relative path to a file registered on a task

    Returns:
        File download response
    """
    safe_path = _authorized_output_file(filepath)
    if not safe_path:
        return jsonify({'error': 'File not found'}), 404

    return send_file(
        safe_path,
        as_attachment=True,
        download_name=safe_path.name
    )


# ==================== WebSocket Event Handlers ====================

@socketio.on('connect')
def handle_connect():
    """Handle new WebSocket connection."""
    print(f'[WebSocket] Client connected: {request.sid}')
    emit('connected', {
        'message': 'Connected to Vocal2Midi server',
        'server_time': datetime.now().isoformat()
    })


@socketio.on('disconnect')
def handle_disconnect():
    """Handle WebSocket disconnection."""
    try:
        print(f'[WebSocket] Client disconnected: {request.sid}')
    except Exception:
        pass


@socketio.on('join_task')
def on_join_task(data):
    """Join a task room to receive updates.

    Expects:
        data: dict with 'task_id' key
    """
    task_id = data.get('task_id')
    if task_id:
        join_room(task_id)
        print(f'[WebSocket] Client {request.sid} joined task {task_id}')
        emit('joined', {'task_id': task_id})
        
        # Send current task status if exists
        task = task_manager.get_task(task_id)
        if task:
            emit('status_change', {
                'task_id': task.id,
                'status': task.status,
                'progress': task.progress,
                'stage': task.stage,
                'error': task.error
            })
            
            # Send backlogs if any
            if task.logs:
                emit('backlogs', {
                    'task_id': task.id,
                    'logs': task.logs
                })
        else:
            download_task = model_download_manager.get_task(task_id)
            if download_task:
                emit('status_change', model_download_manager.serialize_task(download_task))
                if download_task.logs:
                    emit('backlogs', {
                        'task_id': download_task.id,
                        'task_type': 'model_download',
                        'logs': download_task.logs
                    })
    else:
        emit('error', {'message': 'Missing task_id'})


@socketio.on('leave_task')
def on_leave_task(data):
    """Leave a task room.

    Expects:
        data: dict with 'task_id' key
    """
    task_id = data.get('task_id')
    if task_id:
        leave_room(task_id)
        print(f'[WebSocket] Client {request.sid} left task {task_id}')
        emit('left', {'task_id': task_id})


@socketio.on('stop_task')
def on_stop_task(data):
    """Stop a task via WebSocket.

    Expects:
        data: dict with 'task_id' key
    """
    task_id = data.get('task_id')
    if task_id:
        success = task_manager.stop_task(task_id)
        task_type = 'pipeline'
        if not success:
            success = model_download_manager.stop_task(task_id)
            if success:
                task_type = 'model_download'
        emit('stop_response', {
            'task_id': task_id,
            'task_type': task_type,
            'success': success,
            'message': 'Stop request sent' if success else 'Failed to stop task'
        }, room=request.sid)
        
        # Also emit status_change to all clients in the task room
        task = task_manager.get_task(task_id)
        if task:
            emit('status_change', {
                'task_id': task.id,
                'status': task.status,
                'progress': task.progress,
                'stage': task.stage,
                'error': task.error
            }, room=task_id)
        else:
            download_task = model_download_manager.get_task(task_id)
            if download_task:
                emit('status_change',
                     model_download_manager.serialize_task(download_task),
                     room=task_id)


# ==================== Error Handlers ====================

@app.errorhandler(404)
def not_found(error):
    """Handle 404 errors."""
    return jsonify({
        'success': False,
        'error': 'Resource not found'
    }), 404


@app.errorhandler(500)
def internal_error(error):
    """Handle 500 errors."""
    print(f"[Server Error] {error}")
    return jsonify({
        'success': False,
        'error': 'Internal server error',
        'details': str(error)
    }), 500


@app.errorhandler(413)
def too_large(error):
    """Handle file too large errors."""
    return jsonify({
        'success': False,
        'error': 'File too large. Maximum size is 500MB.'
    }), 413


# ==================== Main Entry Point ====================

def get_server_bind() -> tuple[str, int]:
    """Return host/port for the development server."""
    host = os.environ.get('V2M_WEB_HOST', '0.0.0.0')
    raw_port = os.environ.get('V2M_WEB_PORT', os.environ.get('PORT', '5000'))
    try:
        port = int(raw_port)
    except ValueError:
        print(f"[Warning] Invalid V2M_WEB_PORT/PORT value: {raw_port!r}; using 5000")
        port = 5000
    return host, port


if __name__ == '__main__':
    server_host, server_port = get_server_bind()
    print(f"""
╔═══════════════════════════════════════════════════════════╗
║          Vocal2Midi Web Server v1.0.0                    ║
║                                                          ║
║   Starting server at http://{server_host}:{server_port:<15}║
║   Press CTRL+C to stop                                  ║
╚═══════════════════════════════════════════════════════════╝
    """)
    
    # Run the server
    socketio.run(
        app,
        host=server_host,
        port=server_port,
        debug=True,
        use_reloader=False,  # Disable reloader for stability
        log_output=True
    )
