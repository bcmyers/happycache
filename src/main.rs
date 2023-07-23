#![cfg(unix)]

use std::fs;
use std::io;
use std::path::PathBuf;

use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;

/// Simple program to greet a person
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output
    #[arg(short, long, default_value = ".happycache.gz")]
    output: PathBuf,

    /// Input
    #[arg(default_value = ".")]
    input: PathBuf,
}

fn main() -> io::Result<()> {
    let Args { input, output } = Args::parse();
    let mut out = {
        let f = fs::File::create(output)?;
        GzEncoder::new(f, Compression::default())
    };
    happycache::spider(&mut out, &input)?;
    out.finish()?;
    Ok(())
}
