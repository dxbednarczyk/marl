use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use anyhow::{bail, Result};
use chrono::{prelude::*, Duration};
use comrak::{nodes::NodeValue, Arena, Options};
use directories::ProjectDirs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

const REMOTE_URL: &str = "https://rentry.co/firehawk52/raw";

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub expiry: DateTime<Utc>,
    pub arls: Vec<ARL>,

    #[serde(skip)]
    cache_path: PathBuf,
    #[serde(skip)]
    now: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ARL {
    pub region: String,
    pub value: String,
    pub expiry: String,
}

impl Default for Data {
    fn default() -> Self {
        let dir = ProjectDirs::from("xyz", "bednarczyk", "marl").unwrap();
        let now = Utc::now();

        Self {
            expiry: now + Duration::days(1),
            arls: Vec::new(),
            cache_path: dir.data_dir().join("arls.json"),
            now,
        }
    }
}

impl Data {
    pub fn load_cache(&mut self) -> Result<()> {
        let arl_file = File::open(&self.cache_path)?;

        let data: Self = serde_json::from_reader(&arl_file)?;

        if data.expiry < data.now {
            bail!("cache expired");
        }

        self.arls = data.arls;

        Ok(())
    }

    pub fn load_remote(&mut self) -> Result<()> {
        let document = ureq::get(REMOTE_URL).call()?.into_string()?;

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &document, &Options::default());

        let mut region: Option<String> = None;
        let mut expiry: Option<NaiveDate> = None;

        for node in root.descendants() {
            match node.data.borrow().value {
                // Flags are images for some reason, and not emojis
                NodeValue::Image(_) => {
                    let alt_text = node.first_child().unwrap().data.borrow();

                    if let NodeValue::Text(ref txt) = alt_text.value {
                        // For country names like Brazil/Brasil
                        let english_name = txt.split('/').next().unwrap();
                        region = Some(english_name.to_string())
                    }
                }
                NodeValue::Text(ref txt) => {
                    // All the relevant table rows are centered using <- ->
                    if !txt.starts_with('<') {
                        continue;
                    }

                    let dates: Vec<_> = txt
                        .trim_end()
                        .split(" ")
                        .filter_map(|p| NaiveDate::parse_from_str(p, "%Y-%m-%d").ok())
                        .collect();

                    if dates.is_empty() {
                        continue;
                    }

                    let exp = dates.first().unwrap().clone();
                    if self.now.date_naive() > exp {
                        continue;
                    }

                    expiry = Some(exp);
                }
                NodeValue::Code(ref c) => {
                    if c.literal.chars().any(|c| !char::is_alphanumeric(c)) {
                        continue;
                    }

                    if c.literal.len() < 128 || region.is_none() || expiry.is_none() {
                        continue;
                    }

                    self.arls.push(ARL {
                        region: region.unwrap(),
                        value: c.literal.clone(),
                        expiry: expiry.unwrap().to_string(),
                    });

                    region = None;
                    expiry = None;
                }
                _ => (),
            }
        }

        self.expiry = self.now + Duration::days(1);

        Ok(())
    }

    pub fn cache(&mut self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.cache_path)?;

        serde_json::to_writer(&mut file, self)?;

        Ok(())
    }

    pub fn filter_cache(&mut self) {
        self.arls.retain(|p| {
            let date = NaiveDate::parse_from_str(&p.expiry, "%Y-%m-%d").unwrap();
            date >= self.now.date_naive()
        });
    }

    pub fn regions(&self) -> Vec<String> {
        self.arls
            .iter()
            .map(|p| p.region.clone())
            .unique()
            .collect_vec()
    }
}
