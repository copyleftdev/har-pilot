# har-pilot

![har-pilot Logo](./logo.png)

**har-pilot** is a powerful tool designed to run HAR (HTTP Archive) files and perform load testing using these files.
## Features

- **Run HAR Files:** Execute HTTP requests captured in HAR files.
- **Load Testing:** Perform load testing by running HAR files multiple times.
- **Detailed Metrics:** Collect and store metrics such as response time, status codes, and response bodies in a SQLite database.
- **Concurrency:** Handle multiple requests concurrently for efficient load testing.
- **Progress Tracking:** Track progress with a detailed progress bar.

## Installation

To get started with `har-pilot`, clone the repository and install the dependencies:

```sh
git clone https://github.com/yourusername/har-pilot.git
cd har-pilot
cargo build
```

## Usage

### Running HAR Files

To run a HAR file and execute the HTTP requests contained within it, use the following command:

```sh
cargo run -- <path-to-har-file> --itercount <number-of-iterations>
```

- `<path-to-har-file>`: The path to the HAR file you want to run.
- `<number-of-iterations>`: The number of times to run the HAR file for load testing.

For example:

```sh
cargo run -- example.har --itercount 5
```

### Storing Metrics

`har-pilot` stores detailed metrics of each request in a SQLite database. The database file is named with a unique identifier to ensure no conflicts:

- **URL**: The URL of the request.
- **Method**: The HTTP method used (e.g., GET, POST).
- **Response**: The response body.
- **Status**: The HTTP status code.
- **Timestamp**: The timestamp when the request was made.
- **Duration**: The response time in milliseconds.

### Querying Metrics

You can query the SQLite database to analyze the metrics collected during the load testing. Here are some useful queries:

#### Average Response Time Per Second

```sql
SELECT 
    strftime('%Y-%m-%d %H:%M:%S', timestamp) as second,
    AVG(duration_ms) as avg_response_time_ms
FROM 
    metrics
GROUP BY 
    strftime('%Y-%m-%d %H:%M:%S', timestamp)
ORDER BY 
    second;
```

#### Total Requests Per Second

```sql
SELECT 
    strftime('%Y-%m-%d %H:%M:%S', timestamp) as second,
    COUNT(*) as total_requests
FROM 
    metrics
GROUP BY 
    strftime('%Y-%m-%d %H:%M:%S', timestamp)
ORDER BY 
    second;
```

#### Combined Query for Both Average Response Time and Total Requests

```sql
SELECT 
    strftime('%Y-%m-%d %H:%M:%S', timestamp) as second,
    COUNT(*) as total_requests,
    AVG(duration_ms) as avg_response_time_ms
FROM 
    metrics
GROUP BY 
    strftime('%Y-%m-%d %H:%M:%S', timestamp)
ORDER BY 
    second;
```

