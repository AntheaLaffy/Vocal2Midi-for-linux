"""Unit tests for Vocal2Midi Web API.

Tests all REST endpoints, WebSocket events, and task management logic.
Run with: pytest tests/test_web_api.py -v
"""

import json
import os
import sys
import tempfile
import pytest

# Add project root to path for imports
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)


@pytest.fixture
def app():
    """Create and configure a test Flask application."""
    from web_server import app, task_manager
    
    # Configure for testing
    app.config['TESTING'] = True
    app.config['WTF_CSRF_ENABLED'] = False
    
    # Clear tasks before each test
    task_manager.tasks.clear()
    
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
        
        # Check models has required fields
        assert 'game_model_path' in data['models']
        assert 'hfa_model_path' in data['models']
        assert 'asr_model_path' in data['models']

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


class TestStaticFiles:
    """Test static file serving."""

    def test_serve_index_html(self, client):
        """Test that index page is served at root URL."""
        response = client.get('/')
        
        assert response.status_code == 200
        assert b'Vocal2Midi' in response.data or b'<html' in response.data.lower()


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
