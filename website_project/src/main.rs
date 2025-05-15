use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime},
};

use reqwest::blocking::Client;

#[derive(Debug, Clone)]
struct WebsiteStatus {
    url: String,
    action_status: Result<u16, String>,
    response_time: Duration,
    timestamp: SystemTime,
}

impl WebsiteStatus {
    fn to_json_string(&self) -> String {
        let status = match &self.action_status {
            Ok(code) => code.to_string(),
            Err(e) => format!("\"{}\"", e.replace('"', "\\\"")),
        };
        
        let timestamp = self.timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
            
        format!(
            r#"{{
    "url": "{}",
    "status": {},
    "response_time_ms": {},
    "timestamp": {}
}}"#,
            self.url.replace('"', "\\\""),
            status,
            self.response_time.as_millis(),
            timestamp
        )
    }
}

/// Print usage instructions and exit
fn print_usage() -> ! {
    eprintln!("Usage: website_checker [--file sites.txt] [URL ...]");
    eprintln!("       [--workers N] [--timeout S] [--retries N]");
    std::process::exit(2);
}

fn main() {
    // Parse command line arguments
    let mut args = env::args().skip(1);
    let mut file_path = None;
    let mut urls = Vec::new();
    let mut workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let mut timeout = 5;
    let mut retries = 0;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => {
                file_path = args.next().map(PathBuf::from);
            }
            "--workers" => {
                workers = args.next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("Invalid worker count, using default");
                        std::thread::available_parallelism()
                            .map(|n| n.get())
                            .unwrap_or(1)
                    });
            }
            "--timeout" => {
                timeout = args.next()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(5);
            }
            "--retries" => {
                retries = args.next()
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(0);
            }
            arg if arg.starts_with("--") => {
                eprintln!("Unknown option: {}", arg);
                print_usage();
            }
            url => {
                urls.push(url.to_string());
            }
        }
    }

    // Read URLs from file if specified
    if let Some(file_path) = file_path {
        match File::open(&file_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                urls.extend(
                    reader.lines()
                        .filter_map(Result::ok)
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty() && !line.starts_with('#'))
                );
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Check if we have any URLs to process
    if urls.is_empty() {
        print_usage();
    }

    // Create HTTP client with timeout
    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .unwrap_or_else(|e| {
                eprintln!("Failed to create HTTP client: {}", e);
                std::process::exit(1);
            })
    );

    // Create channel for communication between main thread and workers
    let (sender, receiver) = mpsc::channel::<String>();
    let receiver = Arc::new(Mutex::new(receiver));
    let (result_sender, result_receiver) = mpsc::channel::<WebsiteStatus>();

    // Create worker threads
    let mut handles = Vec::with_capacity(workers);
    for _ in 0..workers {
        let client = Arc::clone(&client);
        let receiver = Arc::clone(&receiver);
        let result_sender = result_sender.clone();
        let retries = retries;

        let handle = thread::spawn(move || {
            while let Ok(url) = {
                let receiver = receiver.lock().unwrap();
                receiver.recv()
            } {
                let mut last_error = None;
                let mut response_time = Duration::default();
                let mut status_code = None;

                for attempt in 0..=retries {
                    let start = Instant::now();
                    let result = client.get(&url).send();
                    let elapsed = start.elapsed();

                    match result {
                        Ok(response) => {
                            status_code = Some(response.status().as_u16());
                            response_time = elapsed;
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e);
                            if attempt < retries {
                                thread::sleep(Duration::from_millis(100));
                            }
                        }
                    }
                }

                let status = WebsiteStatus {
                    url: url.clone(),
                    action_status: match status_code {
                        Some(code) => Ok(code),
                        None => Err(last_error.unwrap().to_string()),
                    },
                    response_time,
                    timestamp: SystemTime::now(),
                };

                // Print human-readable output immediately
                println!(
                    "{} - {} in {}ms",
                    status.url,
                    match status.action_status {
                        Ok(code) => format!("HTTP {}", code),
                        Err(ref e) => format!("ERROR: {}", e),
                    },
                    status.response_time.as_millis()
                );

                // Send result to main thread
                result_sender.send(status).unwrap();
            }
        });
        handles.push(handle);
    }

    // Send URLs to workers
    for url in urls {
        sender.send(url).unwrap_or_else(|e| {
            eprintln!("Failed to send URL to worker: {}", e);
        });
    }

    // Close sender to signal workers to finish
    drop(sender);

    // Wait for all worker threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Collect all results
    let mut all_results = Vec::new();
    while let Ok(status) = result_receiver.recv() {
        all_results.push(status);
    }

    // Write JSON output
    let json_string = format!(
        "[\n{}\n]",
        all_results.iter()
            .map(|result| result.to_json_string())
            .collect::<Vec<_>>()
            .join(",\n")
    );

    match File::create("status.json") {
        Ok(mut file) => {
            file.write_all(json_string.as_bytes()).unwrap_or_else(|e| {
                eprintln!("Failed to write JSON file: {}", e);
            });
            println!("Results written to status.json");
        }
        Err(e) => {
            eprintln!("Failed to create status.json: {}", e);
        }
    }
}
