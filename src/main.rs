use std::{collections::HashMap, error::Error, io::{self, Write}};

use dialoguer::Select;
use serde_json::Value;


use crate::{api::{device_flow_auth, get_campaign_details, get_playback_token, get_slug, list_active_games, watch_stream, websockets_connections, GQL_ENDPOINT}, client::client_new, common::structs::{Channel, GQLoperation}, token::{check_token, load_or_create_token}};
mod common;
mod client;
mod token;
mod api;

struct CompanyInfo {
    game_display_name: String,
    game_id: String,
    campaign_id: String,
}


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
    let mut company_info = CompanyInfo { 
        game_display_name: "".to_string(), 
        game_id: "".to_string(), 
        campaign_id: "".to_string() 
    };
    loop {
        let mut select = String::new();
        print!("Select a campaign: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut select)?;
        match select.trim().parse::<usize>() {
            Ok(num) => {
                if let Some(company) = hash.get(&num) {
                    println!("You chose: {}", company.game_display_name);
                    company_info = CompanyInfo {
                        game_display_name: company.game_display_name.to_string(),
                        game_id: company.game_id.to_string(),
                        campaign_id: company.campaign_id.to_string(),
                    };
                    break;
                } else {
                    println!("Invalid number. Try again.");
                }
            }
            Err(_) => println!("Please enter a valid number."),
        } 
        
    }

    let campaign_details = get_campaign_details(&client, &load_token.userid, &company_info.campaign_id).await?;
    let mut active_streams = Vec::new();
    let allow = campaign_details.data.user.dropCampaign.allow;
    match allow.channels {
        Some(channels) => {
            for channel in channels {
                active_streams.push(channel);
            }
        },
        None => {
            let game_slug = get_slug(&client, &company_info.game_display_name).await?;
            let game_directory = GQLoperation::gamedirectory(&game_slug).await?;
            let response = client.post(GQL_ENDPOINT).json(&game_directory).send().await?;
            let json: Value = response.json().await?;
            if let Some(edges) = json.get("data").and_then(|d| d.get("game")).and_then(|g| g.get("streams")).and_then(|s| s.get("edges")).and_then(|e| e.as_array()) {
                for edge in edges {
                    if let Some(broadcaster) = edge.get("node").and_then(|n| n.get("broadcaster")) {
                        let id = broadcaster.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
                        let display_name = broadcaster.get("displayName").and_then(Value::as_str).unwrap_or_default().to_string();
                        let login = broadcaster.get("login").and_then(Value::as_str).unwrap_or_default().to_string();
                        let chan = Channel {
                            id: id,
                            displayName: display_name,
                            name: login,
                        };
                        active_streams.push(chan);
                    }
                }
        }   else {
                return Err("There is no field data.game.streams.edges in the response or it is not an array")?;
            }
        },
    }
    let mut drop_times = campaign_details.data.user.dropCampaign.timeBasedDrops;
    drop_times.sort_by_key(|s| s.requiredMinutesWatched);
    let mut filter_drops_map = HashMap::new();
    for (i, drop) in drop_times.iter().enumerate() {
        filter_drops_map.insert(i + 1, drop);
    };
    let playback_token = get_playback_token(&client, &active_streams.first().map(|s| s.name.clone()).unwrap()).await?;
    let _stream = tokio::spawn({
        let first_stream_name = active_streams.first().map(|s| s.name.clone()).unwrap().clone();
        async move {
            let client = client.clone();
            if let Err(e) = watch_stream(&client, &first_stream_name, &playback_token.0, &playback_token.1).await {
                println!("Error watching stream: {}", e)
            }
        }
    });
    let _websocket = tokio::spawn({
        async move {
            if let Err(e) = websockets_connections(&load_token.oauth, &load_token.userid, &active_streams.first().map(|s| s.id.clone()).unwrap()).await {
                println!("Websocket error: {}", e)
            }
        }
    });
    let first_drop = filter_drops_map.get(&1).ok_or("Didn't find the first drop")?;
    Ok(())
}

