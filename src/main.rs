use structopt::StructOpt;
use har_pilot::{Cli, run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Cli::from_args(), false).await
}
