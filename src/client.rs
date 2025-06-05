use std::error::Error;

use reqwest::{header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT}, Client};
use uuid::Uuid;

use crate::common::structs::ClientInfo;

pub async fn client_new (oauth: Option<&String>) -> Result<Client, Box<dyn Error>> {
    let client = rand::random_range(0..=1);
    let client = match client {
        0 => ClientInfo::android().await?,
        1 => ClientInfo::web().await?,
        _ => unreachable!("Unexpected random value")
    };
    let uuid = Uuid::new_v4().to_string();
    let mut headers = HeaderMap::new();
    headers.insert("Client-Integrity", HeaderValue::from_str(&uuid)?);
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_str("en-EN")?);
    headers.insert("X-Device-Id", HeaderValue::from_str(&uuid)?);
    headers.insert("Client-Version", HeaderValue::from_str(&uuid)?);
    headers.insert("Client-Session-Id", HeaderValue::from_str(&uuid)?);
    headers.insert(ACCEPT, HeaderValue::from_str("*/*")?);
    headers.insert(ORIGIN, HeaderValue::from_str(&client.url)?);
    headers.insert(REFERER, HeaderValue::from_str(&client.url)?);
    headers.insert(USER_AGENT, HeaderValue::from_str(&client.user_agent)?);
    if let Some(token) = oauth {
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("OAuth {}", token))?);
    }
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
    headers.insert("Client-Id", HeaderValue::from_str(&client.id)?);

    let twitch_client = Client::builder().default_headers(headers).build()?;
    Ok(twitch_client)
}