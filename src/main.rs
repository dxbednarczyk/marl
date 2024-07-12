use std::path::PathBuf;

use anyhow::Result;
use chrono::prelude::*;
use comrak::{arena_tree::Node, nodes::NodeValue, Arena, Options};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DeezerARL {
    value: String,
    expiry: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let dir = ProjectDirs::from("xyz", "bednarczyk", "marl").unwrap();
    fs::create_dir_all(dir.data_dir()).await?;

    let file_path = dir.data_dir().join("arls.json");
    let arl_file = File::open(&file_path).await;

    let mut arl: Option<DeezerARL> = None;

    if arl_file.is_ok() {
        let mut content = String::new();
        arl_file?.read_to_string(&mut content).await?;

        let arls: Vec<DeezerARL> = serde_json::from_str(&content)?;
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

async fn get_new_arls() -> Result<Vec<DeezerARL>> {
    let document = reqwest::get("https://rentry.co/firehawk52/raw")
        .await?
        .text()
        .await?;

    let arena = Arena::new();
    let root = comrak::parse_document(&arena, &document, &Options::default());

    let mut arls = vec![];
    let mut previous: &Node<_> = root;

    for node in root.descendants() {
        if let NodeValue::Code(ref code) = node.data.borrow().value {
            let text_valid = code.literal.chars().all(char::is_alphanumeric);
            let len_valid = code.literal.len() >= 128;

            if text_valid && len_valid {
                if let NodeValue::Text(ref text) = previous.data.borrow().value {
                    let split: Vec<_> = text.trim_end().rsplit(" ").collect();
                    let time = split.iter().skip(1).next().unwrap();

                    if time_is_valid(time) {
                        arls.push(DeezerARL {
                            value: code.literal.clone(),
                            expiry: time.to_string(),
                        })
                    }
                }
            }
        }

        previous = node;
    }

    return Ok(arls);
}

async fn save_arls(arls: &[DeezerARL], file_path: &PathBuf) -> Result<()> {
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
