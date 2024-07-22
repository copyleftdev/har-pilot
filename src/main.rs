use structopt::StructOpt;
use futures::stream::{self, StreamExt};
use tokio;
use indicatif::{ProgressBar, ProgressStyle};

mod har;
use har::{Har, Entry};
use reqwest::Client;
use std::fs::File;
use std::io::BufReader;

#[derive(StructOpt)]
struct Cli {
    /// The path to the HAR file
    har_file: String,

    /// The number of times to run the HAR file
    #[structopt(short, long, default_value = "1")]
    itercount: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
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

    for _ in 0..args.itercount {
        let entries = har.log.entries.clone();
        // Use a stream to process the requests concurrently
        let requests = stream::iter(entries)
            .map(|entry| {
                let client = &client;
                let pb = pb.clone();
                async move {
                    let result = replay_request(client, entry).await;
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

    Ok(())
}

async fn replay_request(client: &Client, entry: Entry) -> Result<(), reqwest::Error> {
    let request = &entry.request;
    let mut req = client.request(
        request.method.parse().unwrap(),
        &request.url,
    );

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

    let _response = req.send().await?;
    // The response status is fetched but not used
    // let status = response.status();

    Ok(())
}
