from streamregex import StreamMatcher, SecurityPatterns

matcher = StreamMatcher()
for pattern in SecurityPatterns.OWASP_TOP_10:
    matcher.add_pattern(pattern)

# Process streaming data
matcher.process_chunk(data)