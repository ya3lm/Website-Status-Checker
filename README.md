# Website-Status-Checker
A concurrent website monitoring tool written in Rust that checks the availability of multiple websites in parallel.

# Build the project:
bashcargo build --release
Usage
Run the program with various options:
# Check specific URLsL:
```
cargo run -- https://example.com https://google.com
```
# Check URLs from a file:
```
cargo run -- --file sites.txt
```
# Advanced Options:
```
--workers N	Number of concurrent worker threads	CPU cores
--timeout S	Request timeout in seconds	5
--retries N	Retry attempts for failed requests	0
```

# Output examples:
```
https://example.com - HTTP 200 in 142ms  
https://invalid-url - ERROR: failed to resolve domain in 0ms 
```

Live output to console showing each URL's status
Generates status.json with detailed results
```json
JSON Output Format
json[
  {
    "url": "https://example.com",
    "status": 200,
    "response_time_ms": 142,
    "timestamp": 1715784321
  },
  {
    "url": "https://invalid-url",
    "status": "ERROR: failed to resolve domain",
    "response_time_ms": 0,
    "timestamp": 1715784322
  }
]
```
# Commandline Options:

--file <path>: Text file with URLs (one per line)
--workers N: Number of concurrent worker threads (default: CPU cores)
--timeout S: Timeout for each request in seconds (default: 5)
--retries N: Number of retry attempts (default: 0)