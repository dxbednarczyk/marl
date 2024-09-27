use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use clap::Subcommand;
use directories::ProjectDirs;
use toml_edit::{value, DocumentMut};

use crate::arl::Data;

#[derive(Debug, Subcommand)]
pub enum Config {
    Streamrip {
        /// Override the config path if necessary
        #[clap(alias = "slug")]
        path: Option<String>,
    },
}

impl Config {
    pub fn update(&self, data: &Data, region: &Option<String>) -> Result<()> {
        match self {
            Config::Streamrip { path } => streamrip(data, path, region)?,
        }

        Ok(())
    }
}

fn get_path(over: &Option<String>, project: String) -> Result<PathBuf> {
    let p = if let Some(p) = over {
        let conv = PathBuf::from(p);

        if !conv.is_file() {
            bail!("must not be a path to a file");
        }

        if !conv.try_exists()? {
            bail!("path does not exist");
        }

        ProjectDirs::from_path(PathBuf::from(p)).unwrap()
    } else {
        ProjectDirs::from("", "", &project).unwrap()
    };

    Ok(PathBuf::from(p.config_dir()))
}

fn streamrip(data: &Data, path: &Option<String>, region: &Option<String>) -> Result<()> {
    let config_path = get_path(path, String::from("streamrip"))?.join("config.toml");

    let mut cfg = File::open(&config_path)?;

    let mut content = String::new();
    cfg.read_to_string(&mut content)?;

    let mut document = content.parse::<DocumentMut>()?;

    let arl = if region.is_some() {
        &data.get_region(region.clone().unwrap())?
    } else {
        &data.arls.first().unwrap().value
    };

    if !document.contains_table("deezer") {
        bail!("config file does not contain deezer table");
    }

    let deezer = document["deezer"].as_table_mut().unwrap();
    deezer["arl"] = value(arl);

    let mut cfg = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&config_path)?;

    cfg.write_all(document.to_string().as_bytes())?;

    Ok(())
}
