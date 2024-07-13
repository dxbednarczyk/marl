use std::path::PathBuf;

use anyhow::Result;
use chrono::prelude::*;
use comrak::{nodes::NodeValue, Arena, Options};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ARL {
    pub region: String,
    pub value: String,
    pub expiry: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let dir = ProjectDirs::from("xyz", "bednarczyk", "marl").unwrap();
    fs::create_dir_all(dir.data_dir()).await?;

    let file_path = dir.data_dir().join("arls.json");
    let arl_file = File::open(&file_path).await;

    let mut arl: Option<ARL> = None;

    if arl_file.is_ok() {
        let mut content = String::new();
        arl_file?.read_to_string(&mut content).await?;

        let arls: Vec<ARL> = serde_json::from_str(&content)?;
        let filtered: Vec<_> = arls
            .into_iter()
            .filter(|p| time_is_valid(&p.expiry))
            .collect();

        if filtered.len() > 0 {
            save_arls(&filtered, &file_path).await?;
            arl = filtered.first().cloned();
        }
    }

    if arl.is_none() {
        let new = get_new_arls().await?;
        arl = new.first().cloned();
        save_arls(&new, &file_path).await?;
    }

    if arl.is_none() {
        return Err(anyhow::anyhow!("can't get valid arl"));
    }

    println!("{}", arl.unwrap().value);

    Ok(())
}

async fn get_new_arls() -> Result<Vec<ARL>> {
    let document = reqwest::get("https://rentry.co/firehawk52/raw")
        .await?
        .text()
        .await?;

    let arena = Arena::new();
    let root = comrak::parse_document(&arena, &document, &Options::default());

    let current_date = Utc::now().date_naive();

    let mut arls = vec![];

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
                if current_date > exp {
                    continue;
                }

                expiry = Some(exp);
            }
            NodeValue::Code(ref c) => {
                // deezer ARLs roughly fit this description, longer than qobuz tokens
                if c.literal.chars().any(|c| !char::is_alphanumeric(c)) || c.literal.len() < 128 {
                    continue;
                }

                if region.is_none() || expiry.is_none() {
                    continue;
                }

                arls.push(ARL {
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

    return Ok(arls);
}

async fn save_arls(arls: &[ARL], file_path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(arls)?;

    let mut out = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .await?;

    out.write_all(json.as_bytes()).await?;

    Ok(())
}

fn time_is_valid(time: &str) -> bool {
    NaiveDate::parse_from_str(time, "%Y-%m-%d").is_ok()
}
