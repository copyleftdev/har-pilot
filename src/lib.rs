use structopt::StructOpt;
use futures::stream::{self, StreamExt};
use tokio;
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{params, Connection};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use tokio::process::Command;

pub mod har;
use har::{Har, Entry};
use reqwest::Client;
use std::fs::File;
use std::io::BufReader;

#[derive(StructOpt)]
pub struct Cli {
    /// The path to the HAR file
    pub har_file: String,

    /// The number of times to run the HAR file
    #[structopt(short, long, default_value = "1")]
    pub itercount: usize,

    /// The S3 bucket to upload the results to
    #[structopt(long)]
    pub s3_bucket: Option<String>,
}

pub async fn run(args: Cli, mock: bool) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(&args.har_file)?;
    let reader = BufReader::new(file);
    let har: Har = serde_json::from_reader(reader)?;

    let client = Client::new();
    let total_requests = har.log.entries.len() * args.itercount;
    let pb = ProgressBar::new(total_requests as u64);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .progress_chars("#>-"),
    );

    let db_path = format!("{}_results.db", Uuid::new_v4().to_string());
    let conn = Arc::new(Mutex::new(Connection::open(&db_path)?));

    {
        let conn = conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE metrics (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                method TEXT NOT NULL,
                response TEXT,
                status INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                duration_ms INTEGER NOT NULL
            )",
            [],
        )?;
    }

    for _ in 0..args.itercount {
        let entries = har.log.entries.clone();
        // Use a stream to process the requests concurrently
        let requests = stream::iter(entries)
            .map(|entry| {
                let client = &client;
                let conn = Arc::clone(&conn);
                let pb = pb.clone();
                async move {
                    let result = replay_request(client, conn, entry).await;
                    pb.inc(1);
                    result
                }
            })
            .buffer_unordered(10) // Adjust the number of concurrent requests
            .collect::<Vec<_>>()
            .await;

        // Check for errors in the requests
        for result in requests {
            if let Err(e) = result {
                println!("Request failed: {}", e);
            }
        }
    }

    pb.finish_with_message("All requests completed");

    // Upload the SQLite database to S3 if the bucket is specified
    if let Some(bucket) = &args.s3_bucket {
        upload_to_s3(&db_path, &bucket, mock).await?;
    }

    Ok(())
}

pub async fn replay_request(
    client: &Client,
    conn: Arc<Mutex<Connection>>,
    entry: Entry,
) -> Result<(), reqwest::Error> {
    let request = &entry.request;
    let start = std::time::Instant::now();
    let mut req = client.request(request.method.parse().unwrap(), &request.url);

    let pseudo_headers = [":path", ":scheme", ":authority", ":method"];
    for header in &request.headers {
        if !pseudo_headers.contains(&header.name.as_str()) {
            if let Ok(header_name) = header.name.parse::<reqwest::header::HeaderName>() {
                req = req.header(header_name, &header.value);
            }
        }
    }

    if let Some(post_data) = &request.post_data {
        req = req.body(post_data.text.clone());
    }

    let response = req.send().await?;
    let status = response.status();
    let duration = start.elapsed().as_millis() as i64;
    let timestamp = Utc::now().to_rfc3339();
    let response_text = response.text().await.unwrap_or_else(|_| "Failed to read response".to_string());

    let conn = conn.lock().unwrap();
    if let Err(e) = conn.execute(
        "INSERT INTO metrics (url, method, response, status, timestamp, duration_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![request.url, request.method, response_text, status.as_u16(), timestamp, duration],
    ) {
        println!("Failed to insert into database: {}", e);
    }

    Ok(())
}

pub async fn upload_to_s3(db_path: &str, bucket: &str, mock: bool) -> Result<(), Box<dyn std::error::Error>> {
    if mock {
        println!("Mock upload to S3: {} -> s3://{}", db_path, bucket);
        return Ok(());
    }

    let output = Command::new("aws")
        .arg("s3")
        .arg("cp")
        .arg(db_path)
        .arg(format!("s3://{}", bucket))
        .output()
        .await?;

    if !output.status.success() {
        Err(format!(
            "Failed to upload to S3: {}",
            String::from_utf8_lossy(&output.stderr)
        ))?;
    }

    Ok(())
}
