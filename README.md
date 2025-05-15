# Website-Status-Checker
A concurrent website monitoring tool written in Rust that checks the availability of multiple websites in parallel.

# Build the project:
bashcargo build --release
Usage
Run the program with various options:
Check specific URLs
bashcargo run -- https://example.com https://google.com
Check URLs from a file
bashcargo run -- --file sites.txt
Advanced Options
bash# Specify number of worker threads
cargo run -- --workers 4

# Set request timeout
cargo run -- --timeout 10

# Configure retries
cargo run -- --retries 2
Full Example
bashcargo run -- --file sites.txt --workers 8 --timeout 5 --retries 1
Output

Live output to console showing each URL's status
Generates status.json with detailed results
```json
JSON Output Format
json[
  {
    "url": "https://example.com",
    "status": 200,
    "response_time_ms": 250,
    "timestamp": 1672531200
  }
  
]
```
# Commandline Options:

--file <path>: Text file with URLs (one per line)
--workers N: Number of concurrent worker threads (default: CPU cores)
--timeout S: Timeout for each request in seconds (default: 5)
--retries N: Number of retry attempts (default: 0)

