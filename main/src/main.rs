#![allow(dead_code)]
use std::path::PathBuf;

use clap::Parser;
use mimalloc::MiMalloc;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long)]
    pub bvh_disabled: bool,

    #[clap(long)]
    pub debug: bool,

    #[clap(short = 'W', long, default_value = "1280")]
    pub width: u32,

    #[clap(short = 'H', long, default_value = "720")]
    pub height: u32,

    #[clap(short = 'd', long, default_value = "50")]
    pub max_depth: u32,

    #[clap(long = "spp", default_value = "100")]
    pub samples_per_pixel: u32,

    #[clap(short, long, default_value = "0")]
    pub camera: usize,

    pub input: PathBuf,

    #[clap(default_value = "image.jpeg")]
    pub output: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(if args.debug {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .init();

    Ok(())
}
