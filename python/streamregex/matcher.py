from typing import Callable, List, Optional, Union
from dataclasses import dataclass
import threading
from concurrent.futures import ThreadPoolExecutor

# Import the Rust module
from .streamregex_rust import PyStreamMatcher


@dataclass
class MatchResult:
    """Container for pattern match results"""
    pattern_id: str
    position: int
    length: int


class StreamMatcher:
    """High-level Python interface to StreamRegex"""

    def __init__(self):
        self._matcher = PyStreamMatcher()
        self._callbacks: List[Callable[[MatchResult], None]] = []
        self._lock = threading.Lock()
        self._executor = ThreadPoolExecutor(max_workers=1)

    def add_pattern(self, pattern: str, pattern_id: Optional[str] = None) -> str:
        """
        Add a pattern to the matcher.

        Args:
            pattern: The pattern to match
            pattern_id: Optional identifier for the pattern

        Returns:
            The pattern ID (either provided or auto-generated)
        """
        with self._lock:
            return self._matcher.add_pattern(pattern, pattern_id)

    def add_callback(self, callback: Callable[[MatchResult], None]):
        """
        Add a callback to be called when patterns match.

        Args:
            callback: Function to call with MatchResult when patterns are found
        """
        self._callbacks.append(callback)

    def process_chunk(self, data: Union[bytes, bytearray, memoryview]):
        """
        Process a chunk of streaming data.

        Args:
            data: Bytes-like object containing the data to process
        """
        if not isinstance(data, (bytes, bytearray, memoryview)):
            raise TypeError("Data must be bytes, bytearray, or memoryview")

        with self._lock:
            self._matcher.process_chunk(bytes(data))

    def process_stream(self, stream, chunk_size: int = 64 * 1024):
        """
        Process a stream of data in chunks.

        Args:
            stream: File-like object supporting read()
            chunk_size: Size of chunks to read and process
        """
        while True:
            chunk = stream.read(chunk_size)
            if not chunk:
                break
            self.process_chunk(chunk)

    async def process_stream_async(self, stream, chunk_size: int = 64 * 1024):
        """
        Process a stream asynchronously.

        Args:
            stream: AsyncIO stream supporting read()
            chunk_size: Size of chunks to read and process
        """
        while True:
            chunk = await stream.read(chunk_size)
            if not chunk:
                break
            await self._executor.submit(self.process_chunk, chunk)

    def memory_usage(self) -> int:
        """Get current memory usage in bytes"""
        return self._matcher.memory_usage()

    def __del__(self):
        """Clean up resources"""
        if self._executor:
            self._executor.shutdown()


# Example security pattern sets
class SecurityPatterns:
    """Common security-related pattern sets"""

    OWASP_TOP_10 = [
        r"(?i)(?:\b(?:union\s+all\s+select|union\s+select)\b)",  # SQL Injection
        r"(?i)(?:<script[^>]*>[\s\S]*?</script>)",  # XSS
        r"(?i)(?:\.\./|\%2e\%2e\%2f)",  # Path Traversal
        # ... (rest of the patterns)
    ]