use serde_json::Value;
use tokio::time::sleep;
use std::{error::Error, time::Duration};
use reqwest::{header::{HeaderValue, AUTHORIZATION}, Client};
use crate::common::structs::{Campaigns, ClientInfo, GQLoperation};

pub const GQL_ENDPOINT: &'static str = "https://gql.twitch.tv/gql";

pub async fn device_flow_auth (client: &Client) -> Result<(String, String), Box<dyn Error>> {
    let client_info = ClientInfo::android().await?;
    println!("Requesting authorization via Device Flow...");
    let payload = [
        ("client_id", client_info.id.as_str()),
        ("scope", "user_read")
    ];
    let response = client.post("https://id.twitch.tv/oauth2/device").form(&payload).send().await?;
    let json: Value = response.json().await?;
    if let Some(message) = json.get("message") {
        return Err(format!("Authorization error: {}", message))?;
    };

    let user_code = &json["user_code"].as_str().ok_or("Missing user_code field in response")?;
    let device_code = &json["device_code"].as_str().ok_or("The device_code field is missing in the response")?;
    let verification_url = &json["verification_uri"].as_str().ok_or("Missing verification_uri field in response")?;
    println!("Go to: {} and enter the code: {}", verification_url, user_code);
    println!("We are waiting for confirmation...");
    let data = [
        ("client_id", client_info.id.as_str()),
        ("device_code", device_code),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
    ];
    loop {
        sleep(Duration::from_secs(5)).await;
        let token_response = client.post("https://id.twitch.tv/oauth2/token").form(&data).send().await?;
        let json: Value = token_response.json().await?;
        if let Some(acces_token) = json.get("access_token").and_then(|v| v.as_str()) {
            let inventory_form = GQLoperation::inventory(false).await?;
            let inventory = client.post(GQL_ENDPOINT).header(AUTHORIZATION, HeaderValue::from_str(&format!("OAuth {}", acces_token))?).json(&inventory_form).send().await?;
            let inventory_json: Value = inventory.json().await?;
            if let Some(data) = inventory_json.get("data") {
                let user_id = &data.get("currentUser").and_then(|v| v.get("id")).and_then(|v| v.as_str()).ok_or("Missing user_id field in response")?;
                return Ok((user_id.to_string(), acces_token.to_string()));
            }
        }
    }
}

pub async fn list_active_games (client: &Client) -> Result<Vec<Campaigns>, Box<dyn Error>> {
    let json_send = GQLoperation::campaigns(false).await?;
    let response = client.post(GQL_ENDPOINT).json(&json_send).send().await?;
    let json: Value = response.json().await?;
    let drops = json.get("data").and_then(|s| s.get("currentUser")).and_then(|s| s.get("dropCampaigns")).and_then(|s| s.as_array()).ok_or("There are no Drops available")?;
    let mut active_games = Vec::new();
    for drop in drops {
        let game_id = drop.get("game").and_then(|s| s.get("id")).and_then(|s| s.as_str()).ok_or("Did not find the game_id in the GQL response")?;
        let game_displayname = drop.get("game").and_then(|s| s.get("displayName")).and_then(|s| s.as_str()).ok_or("Did not find the displayName in the GQL response")?;
        let campaign_id = drop.get("id").and_then(|s| s.as_str()).ok_or("Did not find the campaign_id in the GQL response")?;
        let games = Campaigns {
            game_id: game_id.to_string(),
            game_display_name: game_displayname.to_string(),
            campaign_id: campaign_id.to_string(),
        };
        active_games.push(games);
    }
    Ok(active_games)
}