use anyhow::Result;
use arl::Data;
use clap::{Parser, Subcommand};

mod arl;
mod config;

/// Deezer ARL manager
#[derive(Parser, Debug)]
#[command(author = "Damian Bednarczyk <damian@bednarczyk.xyz>")]
#[command(version = "0.1.1")]
struct Args {
    #[command(subcommand)]
    cmd: Option<Commands>,
    #[arg(short, long)]
    region: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Edit the configuration file for certain downloaders
    #[command(subcommand)]
    Config(config::Config),
    /// Invalidates the current ARL in the stack (optionally, for a specific region)
    Invalidate,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut data = Data::load()?;

    if let Some(c) = args.cmd {
        match c {
            Commands::Invalidate => data.invalidate(args.region),
            Commands::Config(c) => c.update(&data, &args.region)?,
        }

        data.cache()?;
        return Ok(());
    }

    let arl = if args.region.is_some() {
        &data.get_region(args.region.unwrap())?
    } else {
        &data.arls.first().unwrap().value
    };

    println!("{arl}");

    data.cache()?;

    Ok(())
}
