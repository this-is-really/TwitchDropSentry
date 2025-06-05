use std::{env::current_dir, error::Error};

use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::common::structs::Token;

pub async fn load_or_create_token (userid: &String, oauth: &String) -> Result<Token, Box<dyn Error>> {
    let home_path = current_dir()?;
    let save_path = home_path.join("save.json");
    let check = check_token().await?;
    match check {
        true => {
            let content = fs::read_to_string(&save_path).await?;
            let content_str: Token = serde_json::from_str(&content)?;
            return Ok(content_str)
        }
        false => {
            let mut create_path = fs::File::create(&save_path).await?;
            let token = Token {
                userid: userid.to_string(),
                oauth: oauth.to_string(),
            };
            let token_write = serde_json::to_string_pretty(&token)?;
            create_path.write_all(&token_write.as_bytes()).await?;
            return Ok(token)
        }
    }
}

pub async fn check_token () -> Result<bool, Box<dyn Error>> {
    let home_path = current_dir()?;
    let save_path = home_path.join("save.json");
    if save_path.exists() {
        Ok(true)
    } else {
        Ok(false)
    }
}