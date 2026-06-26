"""Web-compatible stream redirector for Vocal2Midi.

Replaces PyQt5 Signal-based StreamRedirector from gui/fluent_worker.py
with a callback-based approach suitable for WebSocket logging.
"""


class WebStreamRedirector:
    """Redirects stream output to a callback function.

    This class replaces the StreamRedirector from gui/fluent_worker.py
    which used PyQt5 Signals. Instead, it uses a simple callback function
    that can send messages via WebSocket or any other transport.

    Usage:
        def my_log_callback(message: str, level: str = 'info'):
            socketio.emit('log', {'message': message, 'level': level})

        import sys
        sys.stdout = WebStreamRedirector(sys.stdout, my_log_callback)
    """

    def __init__(self, stream, callback):
        """
        Args:
            stream: The original stream object (e.g., sys.stdout)
            callback: A callable that accepts (message: str, level: str)
        """
        self.stream = stream
        self.callback = callback

    def write(self, text):
        """Write text to both original stream and callback."""
        if text.strip() and self.callback:
            try:
                self.callback(text.strip(), 'info')
            except Exception:
                pass  # Don't let logging errors break the pipeline
        self.stream.write(text)

    def flush(self):
        """Flush the underlying stream."""
        self.stream.flush()

    def __getattr__(self, name):
        """Delegate attribute access to the underlying stream."""
        return getattr(self.stream, name)
