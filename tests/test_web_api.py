"""Unit tests for Vocal2Midi Web API.

Tests all REST endpoints, WebSocket events, and task management logic.
Run with: pytest tests/test_web_api.py -v
"""

import json
import copy
import os
import pathlib
import sys
import tempfile
import zipfile
from urllib.parse import quote

import pytest

# Add project root to path for imports
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)


@pytest.fixture
def app(tmp_path, monkeypatch):
    """Create and configure a test Flask application."""
    import web_server
    from web_server import app, task_manager, model_download_manager
    
    # Configure for testing
    app.config['TESTING'] = True
    app.config['WTF_CSRF_ENABLED'] = False
    monkeypatch.setattr(web_server, 'SETTINGS_FILE', tmp_path / 'web_settings.json')
    web_server.current_settings = copy.deepcopy(web_server.DEFAULT_SETTINGS)
    
    # Clear tasks before each test
    task_manager.tasks.clear()
    model_download_manager.tasks.clear()
    model_download_manager.active_task_id = None
    
    yield app


@pytest.fixture
def client(app):
    """A test client for the app."""
    return app.test_client()


@pytest.fixture
def sample_audio_file():
    """Create a temporary WAV file for testing."""
    with tempfile.NamedTemporaryFile(suffix='.wav', delete=False, mode='wb') as f:
        # Write minimal WAV header + dummy data (44 bytes header + some data)
        f.write(b'RIFF')
        f.write((100).to_bytes(4, 'little'))  # File size - 8
        f.write(b'WAVE')
        f.write(b'fmt ')
        f.write((16).to_bytes(4, 'little'))  # Chunk size
        f.write((1).to_bytes(2, 'little'))   # PCM format
        f.write((1).to_bytes(2, 'little'))   # Mono
        f.write((44100).to_bytes(4, 'little'))  # Sample rate
        f.write((88200).to_bytes(4, 'little'))  # Byte rate
        f.write((2).to_bytes(2, 'little'))   # Block align
        f.write((16).to_bytes(2, 'little'))  # Bits per sample
        f.write(b'data')
        f.write((56).to_bytes(4, 'little'))  # Data size
        f.write(b'\x00' * 56)  # Dummy audio data
        
        temp_path = f.name
    
    yield temp_path
    
    # Cleanup
    if os.path.exists(temp_path):
        os.unlink(temp_path)


class TestPipelineAPI:
    """Test pipeline-related API endpoints."""

    def test_start_pipeline_no_file(self, client):
        """Test starting pipeline without file returns 400 error."""
        response = client.post('/api/pipeline/start')
        
        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'error' in data
        assert 'audio file' in data['error'].lower()

    def test_start_pipeline_empty_file(self, client):
        """Test starting pipeline with empty filename returns 400 error."""
        with tempfile.SpooledTemporaryFile() as tmp:
            response = client.post('/api/pipeline/start', data={
                'audio_file': (tmp, ''),
                'config': '{}'
            })
        
        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'selected file' in data['error'].lower()

    def test_start_pipeline_invalid_format(self, client):
        """Test starting pipeline with invalid file format returns 400 error."""
        with tempfile.NamedTemporaryFile(suffix='.txt', delete=False) as f:
            f.write(b'this is not an audio file')
            temp_path = f.name
        
        try:
            with open(temp_path, 'rb') as f:
                response = client.post('/api/pipeline/start', data={
                    'audio_file': (f, 'test.txt'),
                    'config': '{}'
                })
            
            assert response.status_code == 400
            data = json.loads(response.data)
            assert data['success'] is False
            assert 'invalid' in data['error'].lower() or 'format' in data['error'].lower()
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_start_pipeline_with_valid_file(self, client, sample_audio_file):
        """Test starting pipeline with valid audio file returns 200 and task_id."""
        config = json.dumps({
            'language': 'zh',
            'device': 'cpu',
            'tempo': 120,
            'save_dir': './test_output'
        })
        
        with open(sample_audio_file, 'rb') as f:
            response = client.post('/api/pipeline/start', data={
                'audio_file': (f, 'test.wav'),
                'config': config
            })
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert 'task_id' in data
        assert len(data['task_id']) > 0  # UUID format
        assert data['status'] == 'running'

    def test_stop_nonexistent_task(self, client):
        """Test stopping a nonexistent task returns 404 error."""
        response = client.post('/api/pipeline/stop', 
                             json={'task_id': 'nonexistent-task-id'})
        
        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'not found' in data['error'].lower()

    def test_stop_task_without_id(self, client):
        """Test stopping without task_id returns 400 error."""
        response = client.post('/api/pipeline/stop', json={})
        
        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'missing' in data['error'].lower() or 'task_id' in data['error'].lower()

    def test_get_task_status_not_found(self, client):
        """Test getting status of nonexistent task returns 404 error."""
        response = client.get('/api/pipeline/status/nonexistent-id')
        
        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['success'] is False

    def test_get_task_status_success(self, client, sample_audio_file):
        """Test getting status of existing task returns correct info."""
        # First create a task
        with open(sample_audio_file, 'rb') as f:
            start_response = client.post('/api/pipeline/start', data={
                'audio_file': (f, 'test.wav'),
                'config': '{}'
            })
        
        start_data = json.loads(start_response.data)
        task_id = start_data['task_id']
        
        # Now get its status
        response = client.get(f'/api/pipeline/status/{task_id}')
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert data['task_id'] == task_id
        assert data['status'] in ['pending', 'running', 'completed', 'failed', 'cancelled']
        assert isinstance(data['progress'], int)

    def test_list_tasks_empty(self, client):
        """Test listing tasks when no tasks exist returns empty list."""
        response = client.get('/api/pipeline/list')
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert data['count'] == 0
        assert isinstance(data['tasks'], list)

    def test_list_tasks_after_creation(self, client, sample_audio_file):
        """Test listing tasks after creating one shows it in the list."""
        # Create a task
        with open(sample_audio_file, 'rb') as f:
            client.post('/api/pipeline/start', data={
                'audio_file': (f, 'test.wav'),
                'config': '{}'
            })
        
        # List tasks
        response = client.get('/api/pipeline/list')
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert data['count'] >= 1
        assert len(data['tasks']) >= 1


class TestSettingsAPI:
    """Test settings-related API endpoints."""

    def test_get_settings_default(self, client):
        """Test getting default settings returns valid structure."""
        response = client.get('/api/settings')
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        
        # Check structure
        assert 'models' in data
        assert 'params' in data
        assert 'debug' in data
        assert 'pipeline' in data
        assert 'downloads' in data
        
        # Check models has required fields
        assert 'game_model_path' in data['models']
        assert 'hfa_model_path' in data['models']
        assert 'asr_model_path' in data['models']
        assert data['pipeline']['save_dir'] == './output'
        assert data['downloads']['qwen_source'] == 'auto'

    def test_update_settings(self, client):
        """Test updating settings succeeds."""
        new_settings = {
            'params': {
                'seg_threshold': 0.5,
                'slice_min': 5.0
            },
            'debug': {
                'export_txt': True
            }
        }
        
        response = client.put('/api/settings', 
                              json=new_settings,
                              content_type='application/json')
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert 'message' in data

    def test_update_settings_persists_to_disk(self, client):
        """Test updating settings writes the persistent Web settings file."""
        import web_server

        response = client.put('/api/settings', json={
            'pipeline': {
                'language': 'ja',
                'lyric_output_mode': 'kana',
                'save_dir': './custom_output',
                'export_ustx': True
            },
            'downloads': {
                'qwen_source': 'huggingface',
                'proxy_mode': 'manual',
                'proxy_url': 'http://127.0.0.1:7890',
                'force': True
            }
        })

        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert web_server.SETTINGS_FILE.is_file()

        saved = json.loads(web_server.SETTINGS_FILE.read_text(encoding='utf-8'))
        assert saved['pipeline']['language'] == 'ja'
        assert saved['pipeline']['save_dir'] == './custom_output'
        assert saved['downloads']['qwen_source'] == 'huggingface'
        assert saved['downloads']['proxy_url'] == 'http://127.0.0.1:7890'

        web_server.current_settings = copy.deepcopy(web_server.DEFAULT_SETTINGS)
        reloaded = web_server._load_settings_from_disk()
        assert reloaded['pipeline']['lyric_output_mode'] == 'kana'
        assert reloaded['downloads']['force'] is True

    def test_update_settings_rejects_non_object_section(self, client):
        """Test malformed settings sections return 400 instead of corrupting settings."""
        response = client.put('/api/settings', json={
            'pipeline': 'not-an-object'
        })

        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'pipeline' in data['error']

    def test_reset_settings(self, client):
        """Test resetting settings to defaults succeeds."""
        # First modify settings
        client.put('/api/settings', json={
            'params': {'seg_threshold': 0.99}
        })
        
        # Now reset
        response = client.post('/api/settings/reset')
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        
        # Verify reset happened
        get_response = client.get('/api/settings')
        get_data = json.loads(get_response.data)
        assert get_data['params']['seg_threshold'] == 0.2  # Default value
        assert get_data['pipeline']['save_dir'] == './output'


class TestSystemInfoAPI:
    """Test system information endpoint."""

    def test_system_info_returns_valid_data(self, client):
        """Test system info endpoint returns expected fields."""
        response = client.get('/api/system/info')
        
        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        
        # Check required fields exist
        assert 'version' in data
        assert 'python_version' in data
        assert 'platform' in data
        assert 'device' in data
        assert 'available_devices' in data
        assert isinstance(data['available_devices'], list)


class TestModelDownloadAPI:
    """Test model download API endpoints."""

    def test_model_status_returns_known_models(self, client):
        """Test model status endpoint returns the expected model catalog."""
        response = client.get('/api/models/status')

        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert 'models' in data
        assert 'installed_count' in data
        assert 'missing_count' in data

        model_ids = {model['id'] for model in data['models']}
        assert {'game', 'hfa', 'rmvpe', 'romaji', 'qwen'} <= model_ids
        for model in data['models']:
            assert 'target_path' in model
            assert 'installed' in model

    def test_start_model_download_with_valid_selection(self, client, monkeypatch):
        """Test starting a model download with explicit models."""
        import web_server

        captured = {}

        class FakeTask:
            id = 'fake-download-task'
            status = 'running'

        def fake_start_task(
            selected_models,
            qwen_source,
            force,
            proxy_mode,
            proxy_url,
            socketio_instance,
        ):
            captured['selected_models'] = selected_models
            captured['qwen_source'] = qwen_source
            captured['force'] = force
            captured['proxy_mode'] = proxy_mode
            captured['proxy_url'] = proxy_url
            captured['socketio_instance'] = socketio_instance
            return FakeTask()

        monkeypatch.setattr(
            web_server.model_download_manager,
            'start_task',
            fake_start_task
        )

        response = client.post('/api/models/download', json={
            'models': ['game', 'qwen'],
            'qwen_source': 'huggingface',
            'force': True,
            'proxy_mode': 'manual',
            'proxy_url': 'http://127.0.0.1:7890'
        })

        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert data['task_id'] == 'fake-download-task'
        assert captured['selected_models'] == ['game', 'qwen']
        assert captured['qwen_source'] == 'huggingface'
        assert captured['force'] is True
        assert captured['proxy_mode'] == 'manual'
        assert captured['proxy_url'] == 'http://127.0.0.1:7890'

    def test_start_model_download_defaults_to_missing_models(self, client, monkeypatch):
        """Test omitting models downloads the missing set."""
        import web_server

        captured = {}

        class FakeTask:
            id = 'missing-download-task'
            status = 'running'

        monkeypatch.setattr(
            web_server.model_download_manager,
            'model_statuses',
            lambda: [
                {'id': 'game', 'installed': False},
                {'id': 'hfa', 'installed': True},
                {'id': 'qwen', 'installed': False},
            ]
        )

        def fake_start_task(
            selected_models,
            qwen_source,
            force,
            proxy_mode,
            proxy_url,
            socketio_instance,
        ):
            captured['selected_models'] = selected_models
            captured['proxy_mode'] = proxy_mode
            captured['proxy_url'] = proxy_url
            return FakeTask()

        monkeypatch.setattr(
            web_server.model_download_manager,
            'start_task',
            fake_start_task
        )

        response = client.post('/api/models/download', json={})

        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert captured['selected_models'] == ['game', 'qwen']
        assert captured['proxy_mode'] == 'system'
        assert captured['proxy_url'] == ''

    def test_start_model_download_rejects_unknown_model(self, client):
        """Test unknown model IDs return 400."""
        response = client.post('/api/models/download', json={
            'models': ['does-not-exist']
        })

        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'unknown model' in data['error'].lower()

    def test_start_model_download_rejects_invalid_qwen_source(self, client):
        """Test invalid qwen_source returns 400."""
        response = client.post('/api/models/download', json={
            'models': ['qwen'],
            'qwen_source': 'invalid-source'
        })

        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'qwen_source' in data['error']

    def test_start_model_download_rejects_invalid_proxy_mode(self, client):
        """Test invalid proxy_mode returns 400."""
        response = client.post('/api/models/download', json={
            'models': ['qwen'],
            'proxy_mode': 'bad-proxy-mode'
        })

        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'proxy_mode' in data['error']

    def test_start_model_download_requires_manual_proxy_url(self, client):
        """Test manual proxy mode requires a proxy URL."""
        response = client.post('/api/models/download', json={
            'models': ['qwen'],
            'proxy_mode': 'manual',
            'proxy_url': ''
        })

        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'proxy_url' in data['error']

    def test_start_model_download_requires_proxy_scheme(self, client):
        """Test manual proxy URL must include a scheme."""
        response = client.post('/api/models/download', json={
            'models': ['qwen'],
            'proxy_mode': 'manual',
            'proxy_url': '127.0.0.1:7890'
        })

        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'scheme' in data['error']

    def test_start_model_download_conflict_when_running(self, client, monkeypatch):
        """Test starting a second download returns 409."""
        import web_server

        def fake_start_task(
            selected_models,
            qwen_source,
            force,
            proxy_mode,
            proxy_url,
            socketio_instance,
        ):
            raise RuntimeError('A model download task is already running.')

        monkeypatch.setattr(
            web_server.model_download_manager,
            'start_task',
            fake_start_task
        )

        response = client.post('/api/models/download', json={
            'models': ['game']
        })

        assert response.status_code == 409
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'already running' in data['error']

    def test_get_model_download_status_not_found(self, client):
        """Test getting a nonexistent download task returns 404."""
        response = client.get('/api/models/download/status/not-a-task')

        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['success'] is False

    def test_stop_model_download(self, client):
        """Test stopping an existing running download task."""
        from web_server import model_download_manager

        task = model_download_manager.create_task(['game'], 'auto', False)
        task.status = 'running'

        response = client.post('/api/models/download/stop', json={
            'task_id': task.id
        })

        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert data['status'] == 'stopping'

    def test_stop_model_download_not_found(self, client):
        """Test stopping a nonexistent download task returns 404."""
        response = client.post('/api/models/download/stop', json={
            'task_id': 'not-a-task'
        })

        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['success'] is False


class TestModelDownloadProxyEnv:
    """Test proxy environment override behavior for model downloads."""

    def test_system_proxy_inherits_environment(self, monkeypatch):
        """System proxy mode leaves existing proxy env vars intact."""
        from web_model_download_manager import ModelDownloadManager

        monkeypatch.setenv('HTTP_PROXY', 'http://system-proxy:8080')
        monkeypatch.setenv('NO_PROXY', 'localhost,127.0.0.1')

        manager = ModelDownloadManager()
        task = manager.create_task(['qwen'], 'auto', False, proxy_mode='system')
        env = manager._build_process_env(task)

        assert env['HTTP_PROXY'] == 'http://system-proxy:8080'
        assert env['NO_PROXY'] == 'localhost,127.0.0.1'
        assert env['PYTHONUNBUFFERED'] == '1'

    def test_none_proxy_clears_proxy_environment(self, monkeypatch):
        """No-proxy mode removes upper and lower case proxy env vars."""
        from web_model_download_manager import ModelDownloadManager, PROXY_ENV_KEYS

        for key in PROXY_ENV_KEYS:
            monkeypatch.setenv(key, f'http://{key.lower()}:8080')

        manager = ModelDownloadManager()
        task = manager.create_task(['qwen'], 'auto', False, proxy_mode='none')
        env = manager._build_process_env(task)

        for key in PROXY_ENV_KEYS:
            assert key not in env
        assert env['PYTHONUNBUFFERED'] == '1'

    def test_manual_proxy_overrides_proxy_environment(self, monkeypatch):
        """Manual proxy mode replaces inherited proxy env vars."""
        from web_model_download_manager import ModelDownloadManager

        monkeypatch.setenv('HTTP_PROXY', 'http://old-proxy:8080')
        monkeypatch.setenv('NO_PROXY', 'localhost,127.0.0.1')

        manager = ModelDownloadManager()
        task = manager.create_task(
            ['qwen'],
            'auto',
            False,
            proxy_mode='manual',
            proxy_url='http://127.0.0.1:7890'
        )
        env = manager._build_process_env(task)

        assert env['HTTP_PROXY'] == 'http://127.0.0.1:7890'
        assert env['HTTPS_PROXY'] == 'http://127.0.0.1:7890'
        assert env['ALL_PROXY'] == 'http://127.0.0.1:7890'
        assert env['http_proxy'] == 'http://127.0.0.1:7890'
        assert env['https_proxy'] == 'http://127.0.0.1:7890'
        assert env['all_proxy'] == 'http://127.0.0.1:7890'
        assert 'NO_PROXY' not in env
        assert 'no_proxy' not in env

    def test_popen_process_group_kwargs_posix(self, monkeypatch):
        """POSIX downloads are started in a new session for group termination."""
        import web_model_download_manager as manager_module
        from web_model_download_manager import ModelDownloadManager

        monkeypatch.setattr(manager_module.os, 'name', 'posix')
        manager = ModelDownloadManager()

        assert manager._popen_process_group_kwargs() == {'start_new_session': True}

    def test_popen_process_group_kwargs_windows(self, monkeypatch):
        """Windows downloads are started in a new process group."""
        import web_model_download_manager as manager_module
        from web_model_download_manager import ModelDownloadManager

        monkeypatch.setattr(manager_module.os, 'name', 'nt')
        monkeypatch.setattr(
            manager_module.subprocess,
            'CREATE_NEW_PROCESS_GROUP',
            512,
            raising=False,
        )
        manager = ModelDownloadManager()

        assert manager._popen_process_group_kwargs() == {'creationflags': 512}

    def test_terminate_process_tree_posix_uses_process_group(self, monkeypatch):
        """POSIX termination targets the spawned process group."""
        import signal
        import web_model_download_manager as manager_module
        from web_model_download_manager import ModelDownloadManager

        calls = []

        class FakeProcess:
            pid = 12345

            def poll(self):
                return None

            def terminate(self):
                raise AssertionError('terminate should not be used on POSIX first')

            def kill(self):
                raise AssertionError('kill should not be used without fallback')

        monkeypatch.setattr(manager_module.os, 'name', 'posix')
        monkeypatch.setattr(
            manager_module.os,
            'killpg',
            lambda pid, sig: calls.append((pid, sig)),
        )

        ModelDownloadManager()._terminate_process_tree(FakeProcess())

        assert calls == [(12345, signal.SIGTERM)]

    def test_terminate_process_tree_windows_uses_taskkill(self, monkeypatch):
        """Windows termination targets the process tree with taskkill."""
        import web_model_download_manager as manager_module
        from web_model_download_manager import ModelDownloadManager

        calls = []

        class FakeProcess:
            pid = 9876

            def poll(self):
                return None

            def terminate(self):
                raise AssertionError('terminate should not be used if taskkill works')

            def kill(self):
                raise AssertionError('kill should not be used if taskkill works')

        def fake_run(command, stdout, stderr, check):
            calls.append((command, stdout, stderr, check))

        monkeypatch.setattr(manager_module.os, 'name', 'nt')
        monkeypatch.setattr(manager_module.subprocess, 'run', fake_run)

        ModelDownloadManager()._terminate_process_tree(FakeProcess(), force=True)

        assert calls
        assert calls[0][0] == ['taskkill', '/PID', '9876', '/T', '/F']


class TestDownloadAPI:
    """Test output file download authorization."""

    def test_download_rejects_unregistered_repo_file(self, client):
        """Existing source files are not downloadable unless registered outputs."""
        response = client.get('/api/download/web_server.py')

        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['error'] == 'File not found'

    def test_download_rejects_traversal_path(self, client):
        """Directory traversal is rejected before path resolution."""
        response = client.get('/api/download/..%2Fweb_server.py')

        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['error'] == 'File not found'

    def test_download_rejects_windows_absolute_path(self, client):
        """Windows drive-letter paths are rejected on every platform."""
        response = client.get('/api/download/C:%5CUsers%5Cme%5Csecret.mid')

        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['error'] == 'File not found'

    def test_download_allows_registered_output_file(self, client):
        """A file listed on a task's output_files can be downloaded."""
        from web_server import task_manager

        root = pathlib.Path(PROJECT_ROOT)
        with tempfile.TemporaryDirectory(prefix='pytest_output_', dir=root) as tmp:
            output_path = pathlib.Path(tmp) / 'result.mid'
            output_path.write_bytes(b'MThd')

            task_id = task_manager.create_task({'save_dir': tmp}, 'audio.wav')
            task = task_manager.get_task(task_id)
            task.output_files.append(str(output_path))

            rel_path = output_path.relative_to(root).as_posix()
            response = client.get('/api/download/' + quote(rel_path, safe='/'))

            assert response.status_code == 200
            assert response.data == b'MThd'


class TestServerBind:
    """Test configurable Web server host/port."""

    def test_get_server_bind_uses_env_port(self, monkeypatch):
        import web_server

        monkeypatch.setenv('V2M_WEB_HOST', '127.0.0.1')
        monkeypatch.setenv('V2M_WEB_PORT', '5055')

        assert web_server.get_server_bind() == ('127.0.0.1', 5055)

    def test_get_server_bind_falls_back_on_invalid_port(self, monkeypatch):
        import web_server

        monkeypatch.setenv('V2M_WEB_PORT', 'not-a-port')

        assert web_server.get_server_bind() == ('0.0.0.0', 5000)


class TestFilesystemBrowserAPI:
    """Test local filesystem browser endpoints used by path picker."""

    def test_filesystem_roots_include_project_root(self, client):
        response = client.get('/api/filesystem/roots')

        assert response.status_code == 200
        data = json.loads(response.data)
        assert data['success'] is True
        assert any(root['label'] == '项目目录' for root in data['roots'])

    def test_filesystem_list_directory_mode_lists_directories_only(self, client):
        root = pathlib.Path(PROJECT_ROOT)
        with tempfile.TemporaryDirectory(prefix='picker_', dir=root) as tmp:
            tmp_path = pathlib.Path(tmp)
            child_dir = tmp_path / 'child-model'
            child_dir.mkdir()
            (tmp_path / 'model.onnx').write_bytes(b'onnx')

            rel_path = tmp_path.relative_to(root).as_posix()
            response = client.get('/api/filesystem/list', query_string={
                'path': rel_path,
                'mode': 'directory',
            })

        assert response.status_code == 200
        data = json.loads(response.data)
        names = {entry['name']: entry['type'] for entry in data['entries']}
        assert names['child-model'] == 'directory'
        assert 'model.onnx' not in names
        assert data['input_path'] == rel_path

    def test_filesystem_list_file_mode_filters_extensions(self, client):
        root = pathlib.Path(PROJECT_ROOT)
        with tempfile.TemporaryDirectory(prefix='picker_', dir=root) as tmp:
            tmp_path = pathlib.Path(tmp)
            (tmp_path / 'rmvpe.onnx').write_bytes(b'onnx')
            (tmp_path / 'notes.txt').write_text('skip')

            rel_path = tmp_path.relative_to(root).as_posix()
            response = client.get('/api/filesystem/list', query_string={
                'path': rel_path,
                'mode': 'file',
                'extensions': '.onnx,.bin',
            })

        assert response.status_code == 200
        data = json.loads(response.data)
        names = {entry['name']: entry['type'] for entry in data['entries']}
        assert names['rmvpe.onnx'] == 'file'
        assert 'notes.txt' not in names

    def test_filesystem_list_rejects_invalid_mode(self, client):
        response = client.get('/api/filesystem/list', query_string={
            'mode': 'bad-mode',
        })

        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False
        assert 'mode' in data['error']


class TestModelZipExtraction:
    """Test model zip extraction path safety."""

    def test_extract_zip_merges_single_top_level_directory(self, tmp_path):
        from download_models import extract_zip

        archive = tmp_path / 'valid.zip'
        target = tmp_path / 'target'
        with zipfile.ZipFile(archive, 'w') as zf:
            zf.writestr('model/encoder.onnx', b'onnx')

        extract_zip(archive, target)

        assert (target / 'encoder.onnx').read_bytes() == b'onnx'

    def test_extract_zip_rejects_traversal_member(self, tmp_path):
        from download_models import extract_zip

        archive = tmp_path / 'bad.zip'
        target = tmp_path / 'target'
        with zipfile.ZipFile(archive, 'w') as zf:
            zf.writestr('../evil.txt', b'bad')

        with pytest.raises(ValueError):
            extract_zip(archive, target)

        assert not (tmp_path / 'evil.txt').exists()

    def test_extract_zip_rejects_windows_absolute_member(self, tmp_path):
        from download_models import extract_zip

        archive = tmp_path / 'bad-windows.zip'
        target = tmp_path / 'target'
        with zipfile.ZipFile(archive, 'w') as zf:
            zf.writestr('C:/Users/me/evil.txt', b'bad')

        with pytest.raises(ValueError):
            extract_zip(archive, target)


class TestStaticFiles:
    """Test static file serving."""

    def test_serve_index_html(self, client):
        """Test that index page is served at root URL."""
        response = client.get('/')
        
        assert response.status_code == 200
        assert b'Vocal2Midi' in response.data or b'<html' in response.data.lower()

    def test_index_html_includes_log_copy_controls(self, client):
        """Test that both log panels expose copy controls."""
        response = client.get('/')
        html = response.data.decode('utf-8')

        assert "copyLogs('logPanel', '运行日志')" in html
        assert "copyLogs('modelDownloadLogPanel', '下载日志')" in html
        assert 'function copyLogs(panelId, label)' in html
        assert "document.querySelectorAll('#logPanel .log-entry')" in html

    def test_index_html_includes_path_picker_controls(self, client):
        """Test that directory/file browse buttons use the path picker."""
        response = client.get('/')
        html = response.data.decode('utf-8')

        assert "openPathPicker('saveDirectory', 'directory', '选择保存目录')" in html
        assert "openPathPicker('gameModelPath', 'directory', '选择 GAME 模型目录')" in html
        assert "openPathPicker('rmvpePath', 'file', '选择 RMVPE 模型文件'" in html
        assert 'function openPathPicker(targetId' in html
        assert '/filesystem/list?' in html


class TestErrorHandling:
    """Test error handling scenarios."""

    def test_404_for_unknown_route(self, client):
        """Test that unknown routes return 404."""
        response = client.get('/api/this-route-does-not-exist')
        
        assert response.status_code == 404
        data = json.loads(response.data)
        assert data['success'] is False

    def test_invalid_json_in_body(self, client):
        """Test that invalid JSON in request body returns 400."""
        response = client.put('/api/settings',
                              data='this is not valid json',
                              content_type='application/json')
        
        assert response.status_code == 400
        data = json.loads(response.data)
        assert data['success'] is False


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
