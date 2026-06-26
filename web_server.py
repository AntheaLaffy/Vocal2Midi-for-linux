"""Vocal2Midi Web Server - Flask + SocketIO backend.

Provides REST API and real-time WebSocket communication for the
web-based Vocal2MIDI UI.

Usage:
    python web_server.py

    # Or with gunicorn for production:
    gunicorn --worker-class eventlet -w 1 -b 0.0.0.0:5000 web_server:app
"""

import os
import sys
import json
import tempfile
import pathlib
import copy
from datetime import datetime

from flask import Flask, request, jsonify, send_from_directory, send_file
from flask_socketio import SocketIO, emit, join_room, leave_room, rooms
from flask_cors import CORS

# Add project root to path for imports
PROJECT_ROOT = pathlib.Path(__file__).resolve().parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# Import task manager
from web_task_manager import TaskManager

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
    ping_timeout=60,
    ping_interval=25,
)

# Global task manager instance
task_manager = TaskManager()

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
    }
}

# In-memory settings storage (would use DB/file in production)
# Use deepcopy to avoid modifying DEFAULT_SETTINGS when updating
current_settings = copy.deepcopy(DEFAULT_SETTINGS)


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

        # Merge with current settings (model paths, params, debug options)
        merged_config = {
            **config,
            **current_settings['models'],
            **{f'_{k}': v for k, v in current_settings['params'].items()},
            **current_settings['debug']
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
        # Update each section if provided
        if 'models' in data:
            current_settings['models'].update(data['models'])

        if 'params' in data:
            current_settings['params'].update(data['params'])

        if 'debug' in data:
            current_settings['debug'].update(data['debug'])

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

    return jsonify({
        'success': True,
        'message': 'Settings reset to defaults',
        'settings': current_settings
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


# ==================== Download API ====================

@app.route('/api/download/<path:filepath>')
def download_file(filepath):
    """Download an output file.

    Args:
        filepath: Relative path to the file within output directory

    Returns:
        File download response
    """
    # Security: prevent directory traversal attacks
    safe_path = pathlib.Path(filepath).resolve()
    if '..' in str(filepath) or not safe_path.is_file():
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
    print(f'[WebSocket] Client disconnected: {request.sid}')


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
        emit('stop_response', {
            'task_id': task_id,
            'success': success,
            'message': 'Stop request sent' if success else 'Failed to stop task'
        }, room=request.sid)


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

if __name__ == '__main__':
    print("""
╔═══════════════════════════════════════════════════════════╗
║          Vocal2Midi Web Server v1.0.0                    ║
║                                                          ║
║   Starting server at http://0.0.0.0:5000               ║
║   Press CTRL+C to stop                                  ║
╚═══════════════════════════════════════════════════════════╝
    """)
    
    # Run the server
    socketio.run(
        app,
        host='0.0.0.0',
        port=5000,
        debug=True,
        use_reloader=False,  # Disable reloader for stability
        log_output=True
    )
