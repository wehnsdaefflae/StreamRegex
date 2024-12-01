import unittest
import asyncio
import io
from streamregex import StreamMatcher, SecurityPatterns
from concurrent.futures import ThreadPoolExecutor
import threading
import time


class TestStreamMatcher(unittest.TestCase):
    def setUp(self):
        self.matcher = StreamMatcher()

    def test_basic_matching(self):
        matches = []

        def on_match(result):
            matches.append(result)

        pattern_id = self.matcher.add_pattern("test")
        self.matcher.add_callback(on_match)
        self.matcher.process_chunk(b"this is a test string")

        self.assertEqual(len(matches), 1)
        self.assertEqual(matches[0].pattern_id, pattern_id)

    def test_security_patterns(self):
        matches = []

        def on_match(result):
            matches.append(result)

        # Add OWASP patterns
        for pattern in SecurityPatterns.OWASP_TOP_10:
            self.matcher.add_pattern(pattern)

        self.matcher.add_callback(on_match)

        # Test SQL injection pattern
        self.matcher.process_chunk(b"SELECT * FROM users WHERE id = 1 UNION SELECT password FROM users")
        self.assertTrue(any("sql_injection" in m.pattern_id for m in matches))

        # Test XSS pattern
        self.matcher.process_chunk(b'<script>alert("xss")</script>')
        self.assertTrue(any("xss" in m.pattern_id for m in matches))

    def test_stream_processing(self):
        matches = []

        def on_match(result):
            matches.append(result)

        pattern_id = self.matcher.add_pattern("stream")
        self.matcher.add_callback(on_match)

        # Create an in-memory stream
        stream = io.BytesIO(b"testing stream processing")
        self.matcher.process_stream(stream)

        self.assertEqual(len(matches), 1)
        self.assertEqual(matches[0].pattern_id, pattern_id)

    async def test_async_stream_processing(self):
        matches = []

        def on_match(result):
            matches.append(result)

        pattern_id = self.matcher.add_pattern("async")
        self.matcher.add_callback(on_match)

        # Simulate async stream
        class AsyncStream:
            def __init__(self, data):
                self.data = data
                self.position = 0

            async def read(self, size):
                if self.position >= len(self.data):
                    return b""
                chunk = self.data[self.position:self.position + size]
                self.position += size
                return chunk

        stream = AsyncStream(b"testing async stream processing")
        await self.matcher.process_stream_async(stream)

        self.assertEqual(len(matches), 1)
        self.assertEqual(matches[0].pattern_id, pattern_id)

    def test_concurrent_processing(self):
        pattern_id = self.matcher.add_pattern("concurrent")
        matches_lock = threading.Lock()
        matches = []

        def on_match(result):
            with matches_lock:
                matches.append(result)

        self.matcher.add_callback(on_match)

        # Process in multiple threads
        with ThreadPoolExecutor(max_workers=4) as executor:
            futures = []
            for i in range(4):
                data = f"thread{i} concurrent test".encode()
                futures.append(executor.submit(self.matcher.process_chunk, data))

        # Wait for all threads to complete
        for future in futures:
            future.result()

        self.assertEqual(len(matches), 4)

    def test_memory_usage(self):
        # Add some patterns
        for pattern in SecurityPatterns.OWASP_TOP_10[:5]:
            self.matcher.add_pattern(pattern)

        initial_usage = self.matcher.memory_usage()

        # Process increasing amounts of data
        for size in [1024, 10240, 102400]:
            data = b'x' * size
            self.matcher.process_chunk(data)

            # Memory usage should remain relatively constant
            current_usage = self.matcher.memory_usage()
            # Allow for some small variance
            self.assertLess(abs(current_usage - initial_usage), 1024 * 10)

    def test_error_handling(self):
        # Invalid pattern type
        with self.assertRaises(TypeError):
            self.matcher.process_chunk(12345)

        # Invalid callback
        with self.assertRaises(TypeError):
            self.matcher.add_callback("not a function")


if __name__ == '__main__':
    unittest.main()