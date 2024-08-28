use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
};

use anyhow::Result;
use chrono::{prelude::*, Duration};
use comrak::{nodes::NodeValue, Arena, Options};
use directories::ProjectDirs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const REMOTE_URL: &str = "https://rentry.co/firehawk52/raw";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Data {
    pub expiry: DateTime<Utc>,
    pub sha256: String,
    pub arls: Vec<ARL>,

    #[serde(skip)]
    path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ARL {
    pub region: String,
    pub value: String,
    pub expiry: NaiveDate,
}

impl Data {
    pub fn load() -> Result<Self> {
        let cache_path = ProjectDirs::from("xyz", "bednarczyk", "marl").unwrap();
        let file_path = cache_path.cache_dir().join("marl.json");

        let mut data: Data = match File::open(&file_path) {
            Ok(f) => serde_json::from_reader(f)?,
            Err(_) => Data::default(),
        };

        data.path = file_path;

        let now = Utc::now();
        if data.expiry < now {
            data.load_remote(now)?;
            data.expiry = now + Duration::days(1);
            data.arls.retain(|p| p.expiry > now.date_naive());
        }

        Ok(data)
    }

    pub fn cache(&self) -> Result<()> {
        fs::create_dir_all(self.path.parent().unwrap())?;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        serde_json::to_writer(file, self)?;

        Ok(())
    }

    pub fn regions(&self) -> Vec<String> {
        self.arls
            .iter()
            .map(|p| p.region.clone())
            .unique()
            .collect_vec()
    }

    fn load_remote(&mut self, now: DateTime<Utc>) -> Result<()> {
        let document = ureq::get(REMOTE_URL).call()?.into_string()?;

        let sum = format!("{:x}", Sha256::digest(&document));
        if sum == self.sha256 {
            return Ok(());
        }

        self.sha256 = sum;

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &document, &Options::default());

        let mut region: Option<String> = None;
        let mut expiry: Option<NaiveDate> = None;

        let mut braille_counter = 0;

        for node in root.descendants() {
            if braille_counter > 3 {
                break;
            }

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
                    // Start of token tables (Deezer and Qobuz) as of 8/20/24
                    // begin with 3 U+2800: BRAILLE PATTERN BLANK
                    // If we hit the second table, activate killswitch
                    if txt.contains('\u{2800}') {
                        braille_counter += 1;
                        continue;
                    }

                    let date: Vec<_> = txt
                        .split_whitespace()
                        .filter_map(|p| NaiveDate::parse_from_str(p, "%Y-%m-%d").ok())
                        .collect();

                    if date.is_empty() {
                        continue;
                    }

                    if let Some(d) = date.first() {
                        if &now.date_naive() < d {
                            expiry = Some(d.clone());
                        }
                    }
                }
                NodeValue::Code(ref c) => {
                    if !c.literal.chars().all(char::is_alphanumeric) {
                        continue;
                    }

                    if c.literal.len() < 128 || region.is_none() || expiry.is_none() {
                        continue;
                    }

                    self.arls.push(ARL {
                        region: region.unwrap(),
                        value: c.literal.clone(),
                        expiry: expiry.unwrap(),
                    });

                    region = None;
                    expiry = None;
                }
                _ => (),
            }
        }

        Ok(())
    }
}
