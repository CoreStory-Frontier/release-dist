use chrono::Utc;
use clap::Parser;
use latest_json::Manifest;
use std::path::PathBuf;
use url::Url;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    Manifest::from_dir(
        &args.dist,
        &args.base_url,
        &args.tag,
        args.notes,
        Utc::now(),
    )?
    .write(&args.dist)
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    tag: String,
    #[arg(long)]
    base_url: Url,
    #[arg(long, default_value = "dist")]
    dist: PathBuf,
    #[arg(long)]
    notes: Option<String>,
}
