use std::{collections::HashMap, error::Error, io};

use dialoguer::Select;


use crate::{api::{device_flow_auth, list_active_games}, client::client_new, token::{check_token, load_or_create_token}};
mod common;
mod client;
mod token;
mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let dialogs = vec!["Add a twitch account", "Start farming", "Exit"];
    loop {
        let dialog = Select::new().with_prompt("== Main Menu ==").items(&dialogs).default(0).interact()?;
        match dialog {
            0 => {
                if let Err(e) = auth().await {
                    eprintln!("[Auth Error] {}", e)
                }
            },
            1 => {
                if let Err(e) = farm().await {
                    eprintln!("[Farm Error] {}", e)
                }
            },
            2 => break,
            _ => return Err("Unexpected error")?
        }  
    }
    
    Ok(())
}

async fn auth () -> Result<(), Box<dyn Error>> {
    let client = client_new(None).await?;
    let auth = device_flow_auth(&client).await?;
    load_or_create_token(&auth.0, &auth.1).await?;
    Ok(())
}

async fn farm () -> Result<(), Box<dyn Error>> {
    if check_token().await? == false {
        return Err("Register first")?;
    }
    let load_token = load_or_create_token(&"".to_string(), &"".to_string()).await?;
    let client = client_new(Some(&load_token.oauth)).await?;
    let campaigns = list_active_games(&client).await?;
    let mut hash = HashMap::new();
    for (i, campaign) in campaigns.iter().enumerate() {
        println!("{}: {}", i+1, campaign.game_display_name);
        hash.insert(i + 1, campaign);
    }
    let mut select = String::new();
    println!("Select a campaign");
    io::stdin().read_line(&mut select)?;
    let select = select.trim().parse::<usize>()?;
    if let Some(company) = hash.get(&select) {
        println!("You chose: {}", company.game_display_name)
    }
    Ok(())
}

