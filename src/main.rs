use anyhow::{bail, Result};
use arl::Data;
use clap::{Parser, Subcommand};

mod arl;

/// Deezer ARL manager
#[derive(Parser, Debug)]
#[command(author = "Damian Bednarczyk <damian@bednarczyk.xyz>")]
#[command(version = "0.1.0")]
struct Args {
    #[command(subcommand)]
    cmd: Option<Commands>,
    #[arg(short, long)]
    region: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Invalidates the current ARL in the stack (optionally, for a specific region)
    Invalidate { region: Option<String> },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut data = Data::load()?;

    if let Some(c) = args.cmd {
        match c {
            Commands::Invalidate { region } => data.invalidate(region),
        }

        data.cache()?;
        return Ok(());
    }

    let arl = if args.region.is_some() {
        let region = args.region.unwrap();

        let found = data.arls.iter().find(|p| p.region == region);

        if found.is_none() {
            let region_list = data.regions().join(", ");

            bail!("no ARL present for {region}\nValid regions: {region_list}");
        }

        &found.unwrap().value
    } else {
        &data.arls.first().unwrap().value
    };

    println!("{arl}");

    data.cache()?;

    Ok(())
}
