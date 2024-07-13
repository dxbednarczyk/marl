use anyhow::{anyhow, Result};
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
    let mut data = Data::default();

    if data.load_cache().is_err() {
        data.load_remote()?;
    }

    data.filter_cache();
    data.cache()?;

    let arl = if args.region.is_some() {
        let found = data
            .arls
            .iter()
            .find(|p| &p.region == args.region.as_ref().unwrap());

        if found.is_none() {
            return Err(anyhow!(
                "could not find valid ARL for {}\nValid regions: {}",
                args.region.unwrap(),
                data.regions().join(", ")
            ));
        }

        found.unwrap().value.clone()
    } else {
        data.arls.first().unwrap().value.clone()
    };

    println!("{arl}");

    Ok(())
}
