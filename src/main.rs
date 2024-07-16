use anyhow::{bail, Result};
use arl::Data;
use clap::Parser;

mod arl;

/// Deezer ARL manager
#[derive(Parser, Debug)]
#[command(author = "Damian Bednarczyk <damian@bednarczyk.xyz>")]
#[command(version = "0.1.0")]
struct Args {
    #[arg(short, long)]
    region: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let data = Data::load()?;

    let arl = if args.region.is_some() {
        let region = args.region.unwrap();

        let found = data.arls.iter().find(|p| p.region == region);

        if found.is_none() {
            bail!(
                "could not find valid ARL for {}\nValid regions: {}",
                region,
                data.regions().join(", ")
            );
        }

        &found.unwrap().value
    } else {
        &data.arls.first().unwrap().value
    };

    println!("{arl}");

    data.cache()?;

    Ok(())
}
