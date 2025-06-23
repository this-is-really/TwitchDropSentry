use serde_json::Value;
use tokio::time::sleep;
use std::{error::Error, time::Duration};
use reqwest::{header::{HeaderValue, AUTHORIZATION}, Client};
use crate::{common::structs::{ApiResponse, Campaigns, ClientInfo, GQLoperation}};

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
            } else {
                return Err("Didn't find inventory data")?;
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
        if let Some(active_drop) = drop.get("status") {
            if active_drop == "ACTIVE" {
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
        }
        
    }
    Ok(active_games)
}

pub async fn get_campaign_details (client: &Client, user_id: &String, campaign_id: &String) -> Result<ApiResponse, Box<dyn Error>> {
    let campaign_details_form = GQLoperation::campaigndetails(&user_id, &campaign_id).await?;
    let campaign_details = client.post(GQL_ENDPOINT).json(&campaign_details_form).send().await?;
    let campaign_answer: Value = campaign_details.json().await?;
    if let Some(_) = campaign_answer.get("data") {
        let answer: ApiResponse = serde_json::from_value(campaign_answer)?;
        return Ok(answer)
    } else {
        return Err("No data received from GQL response")?;
    }
}

pub async fn get_slug (client: &Client, game_name: &String) -> Result<String, Box<dyn Error>> {
    let slug = GQLoperation::slugredirect(&game_name).await?;
    let response = client.post(GQL_ENDPOINT).json(&slug).send().await?;
    let json: Value = response.json().await?;
    let mut slug_str = String::new();
    if let Some(data) = json.get("data") {
        let slug = data.get("game").and_then(|s| s.get("slug")).and_then(|s| s.as_str()).ok_or("Didn't find slug")?;
        slug_str.push_str(slug);
    } else {
        return Err("No data received from GQL response")?;
    }
    Ok(slug_str)
}

pub async fn get_playback_token (client: &Client,channel_login: &String) -> Result<(String, String), Box<dyn Error>> {
    let playback_token = GQLoperation::playbackaccesstoken(channel_login).await?;
    let response = client.post(GQL_ENDPOINT).json(&playback_token).send().await?;
    let json: Value = response.json().await?;
    let mut value_str = String::new();
    let mut signature_str = String::new();
    if let Some(data) = json.get("data") {
        if let Some(stream_playback) = data.get("streamPlaybackAccessToken") {
            let value = stream_playback.get("value").and_then(|s| s.as_str()).ok_or("Didn't find token value")?;
            value_str.push_str(&value.to_string());
            let signature = stream_playback.get("signature").and_then(|s| s.as_str()).ok_or("Token signature not found")?;
            signature_str.push_str(&signature.to_string());
        } else {
            return Err("There is no data.streamPlaybackAccessToken field in the response or it is not an array")?;
        }
    } else {
        return Err("Didn't find data in DirectoryPage_Game")?;
    }
    Ok((value_str, signature_str))
}

pub async fn watch_stream (client: &Client, channel_login: &String, token_value: &String, token_signature: &String) -> Result<(), Box<dyn Error + Sync + Send>> {
    let url = format!("https://usher.ttvnw.net/api/channel/hls/{}.m3u8?sig={}&token={}&allow_source=true&player_backend=mediaplayer&playlist_include_framerate=true", channel_login, token_signature, token_value);
    let playlist_response = client.get(&url).timeout(std::time::Duration::from_secs(45)).send().await?;
    if ! playlist_response.status().is_success() {
        return Err(format!("Error getting playlist, status: {}", playlist_response.status()))?
    }

    let mut stream = playlist_response.bytes_stream();

    use tokio_stream::StreamExt;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk_lock) => {
                if chunk_lock.is_empty() {
                    break;
                }
                println!("Received chunk of size {} bytes", chunk_lock.len())
            }
            Err(e) => {
                println!("Error reading stream: {}", e);
                break;
            }
        }
    }
    Ok(())
}

pub async fn check_online (client: &Client, channel_login: &String) -> Result<(), Box<dyn Error + Sync + Send>> {
    let json = GQLoperation::get_stream_info(channel_login).await?;
    let response = client.post(GQL_ENDPOINT).json(&json).send().await?;
    let response_json: Value = response.json().await?;
    let stream = &response_json["data"]["user"]["stream"];
    if stream.is_null() {
        return Err("Streamer offline")?
    } else {
        return Ok(());
    }
}