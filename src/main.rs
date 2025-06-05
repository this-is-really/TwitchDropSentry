use std::{error::Error, thread, time::Duration};

use reqwest::{header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT}, Client};
use serde_json::{json, Value};
use tokio::time::sleep;
use uuid::Uuid;

use crate::common::structs::{ClientInfo, GQLoperation};
mod common;

const GQL_ENDPOINT: &'static str = "https://gql.twitch.tv/gql";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let auth = device_flow_auth(&client).await?;
    println!("{} | {}", auth.0, auth.1);
    Ok(())
}

async fn device_flow_auth (client: &Client) -> Result<(String, String), Box<dyn Error>> {
    let client_info = ClientInfo::android().await?;
    let mut headers = HeaderMap::new();
    let device_id = Uuid::new_v4().to_string();
    headers.insert(ACCEPT, HeaderValue::from_str("application/json")?);
    headers.insert("Client-Id", HeaderValue::from_str(&client_info.id)?);
    headers.insert(ORIGIN, HeaderValue::from_str(&client_info.url)?);
    headers.insert(REFERER, HeaderValue::from_str(&client_info.url)?);
    headers.insert(USER_AGENT, HeaderValue::from_str(&client_info.user_agent)?);
    headers.insert("X-Device-Id", HeaderValue::from_str(&device_id)?);
    println!("Requesting authorization via Device Flow...");
    let payload = [
        ("client_id", client_info.id.as_str()),
        ("scope", "user_read")
    ];
    let response = client.post("https://id.twitch.tv/oauth2/device").headers(headers.clone()).form(&payload).send().await?;
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
        let token_response = client.post("https://id.twitch.tv/oauth2/token").headers(headers.clone()).form(&data).send().await?;
        let json: Value = token_response.json().await?;
        if let Some(acces_token) = json.get("access_token").and_then(|v| v.as_str()) {
            let inventory_form = GQLoperation::inventory(false).await?;
            let mut invnetory_headers = HeaderMap::new();
            invnetory_headers.insert("Client-ID", HeaderValue::from_str("kimne78kx3ncx6brgo4mv6wki5h1ko")?);
            invnetory_headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
            invnetory_headers.insert(USER_AGENT, HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")?);
            invnetory_headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("OAuth {}", acces_token))?);
            let inventory = client.post(GQL_ENDPOINT).headers(invnetory_headers).json(&inventory_form).send().await?;
            let inventory_json: Value = inventory.json().await?;
            if let Some(data) = inventory_json.get("data") {
                let user_id = &data.get("currentUser").and_then(|v| v.get("id")).and_then(|v| v.as_str()).ok_or("Missing user_id field in response")?;
                return Ok((user_id.to_string(), acces_token.to_string()));
            }
        }
    }
}