#[macro_use]
extern crate log;
use asnrun_scrapper::Scrapper;
use clap::Parser;
use futures::future;
use tokio::task;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    limit: Option<i64>,

    #[arg(short, long)]
    offset: Option<i64>,

    #[arg(short, long)]
    jobs: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();
    let args = Args::parse();

    let offset: i64 = args.offset.unwrap_or(0);
    let limit = args.limit;

    info!(
        "Starting scrapper with {jobs} jobs",
        jobs = args.jobs.unwrap_or(1)
    );

    let scrapper = Scrapper::default();

    if let Some(jobs) = args.jobs {
        let tasks: Vec<_> = (0..jobs)
            .map(|id| {
                let scrapper = scrapper.clone();
                task::spawn(async move {
                    if let Err(e) = scrapper.run_worker(0, None, Some(jobs), id).await {
                        eprintln!("Worker {} failed: {:?}", id, e);
                    }
                })
            })
            .collect();

        future::join_all(tasks).await;
    } else {
        scrapper.run_worker(offset, limit, None, 0).await?;
    }

    Ok(())
}
