use std::{collections::{BinaryHeap, HashSet}, error::Error, sync::Arc, time::Duration};

use tokio::sync::watch::Receiver;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::debug;
use twitch_gql_rs::{TwitchClient, structs::{Channels, DropCampaigns, GameDirectory}};

use crate::{retry, r#static::{ALLOW_CHANNELS, CHANNEL_IDS, Channel, DEFAULT_CHANNELS, NOW_WATCHED, retry_backup}};

const UPDATE_TIME: u64 = 15;
const MAX_TOPICS: usize = 50;
const WS_URL: &'static str = "wss://pubsub-edge.twitch.tv/v1";

pub async fn filter_streams (client: Arc<TwitchClient>, campaigns: Arc<Vec<DropCampaigns>>) {
    let mut count = 0;
    let mut video_vec = HashSet::new();
    for campaign in campaigns.iter() {
        let campaign_details = retry!(client.get_campaign_details(&campaign.id));
        if let Some(allow) = campaign_details.allow.channels {
            let mut allow_channels = ALLOW_CHANNELS.lock().await;
            let allow: HashSet<Channels> = allow.into_iter().collect();
            allow_channels.insert(campaign.id.to_string(), allow.clone());
            drop(allow_channels);
            for channel in allow {
                if count >= MAX_TOPICS {
                    break;
                }
                video_vec.insert(Channel { channel_id: channel.id, channel_login: channel.name });
                count += 1
            }
        } else {
            let game_directory = retry!(client.get_game_directory(&campaign_details.game.slug, true));
            let mut all_default = DEFAULT_CHANNELS.lock().await;
            let game_directory: HashSet<GameDirectory> = game_directory.into_iter().collect();
            all_default.insert(campaign.id.to_string(), game_directory.clone());
            drop(all_default);
            for channel in game_directory {
                if count >= MAX_TOPICS {
                    break;
                }
                video_vec.insert(Channel { channel_id: channel.broadcaster.id, channel_login: channel.broadcaster.login });
                count += 1
            }
        }
    }
    let mut lock = CHANNEL_IDS.lock().await;
    *lock = video_vec;
    drop(lock);
    debug!("Drop lock video");
    spawn_ws(client.access_token.clone().unwrap()).await;

    tokio::spawn(async move {
        loop {
            let mut lock = CHANNEL_IDS.lock().await;
            let mut count = lock.len();
            if count < MAX_TOPICS {
                let mut default_channels = DEFAULT_CHANNELS.lock().await;
                for campaign in campaigns.iter() {
                    let slug = retry!(client.get_slug(&campaign.game.displayName));
                    let game_directory = retry!(client.get_game_directory(&slug, true));
                    let game_directory: HashSet<GameDirectory> = game_directory.into_iter().collect();
                    default_channels.insert(campaign.id.clone(), game_directory.clone());

                    for channel in game_directory {
                        if count == MAX_TOPICS {
                            break;
                        }
                        lock.insert(Channel { channel_id: channel.broadcaster.id.clone(), channel_login: channel.broadcaster.login.clone() });
                        count += 1
                    }

                    if count >= MAX_TOPICS {
                        break;
                    }
                }
                drop(default_channels);
            }
            drop(lock);
            debug!("Drop ids");
            sleep(Duration::from_secs(UPDATE_TIME)).await
        }
    });
}

//ws_logick
async fn spawn_ws (auth_token: String) {
    tokio::spawn(async move {
        loop {
            let mut send_channels: HashSet<Channel> = HashSet::new();
            let (ws_stream, _) = retry!(connect_async(WS_URL));
            let (mut write, mut read) = ws_stream.split();
            loop {
                let mut channel_ids = CHANNEL_IDS.lock().await;
                let new_channels: Vec<Channel> = channel_ids.iter().filter(|id| !send_channels.contains(*id)).cloned().collect();
                let delete_channels: Vec<Channel> = send_channels.iter().filter(|id| !channel_ids.contains(&id)).cloned().collect();
                let mut topics = Vec::new();
                let mut delete_topics = Vec::new();

                if !new_channels.is_empty() {
                    for new in new_channels {
                        topics.push(format!("video-playback-by-id.{}", new.channel_id));
                        send_channels.insert(new);
                    }
                    let payload = json!({
                        "type": "LISTEN",
                        "data": {
                            "topics": topics,
                            "auth_token": auth_token
                        }
                    });
                    let payload = serde_json::to_string(&payload).unwrap();
                    let payload = tokio_tungstenite::tungstenite::Message::Text(payload.into());
                    write.send(payload).await.unwrap_or_else(|e| tracing::error!("Failed to send payload to WebSocket: {e}"));
                }

                if !delete_channels.is_empty() {
                    for delete in delete_channels {
                        delete_topics.push(format!("video-playback-by-id.{}", delete.channel_id));
                        send_channels.remove(&delete);
                    }
                    let payload = json!({
                        "type": "UNLISTEN",
                        "data": {
                            "topics": delete_topics,
                            "auth_token": auth_token
                        }
                    });
                    let payload = serde_json::to_string(&payload).unwrap();
                    let payload = tokio_tungstenite::tungstenite::Message::Text(payload.into());
                    write.send(payload).await.unwrap_or_else(|e| tracing::error!("Failed to send payload to WebSocket: {e}"));
                };

                if let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if text.contains("\"type\":\"PING\"") {
                                let pong = Message::Text("{\"type\":\"PONG\"}".into());
                                write.send(pong).await.unwrap();
                            }
                            let json: Value = serde_json::from_str(&text).unwrap();
                            if let Some(err) = json.get("error") {
                                if err != "" {
                                    tracing::error!("{err}")
                                }
                            } else {
                                let data = check_json(&json, "data").unwrap_or_else(|e| {tracing::error!("{e}"); &Value::Null});
                                let message = check_json(&data, "message").unwrap_or_else(|e| {tracing::error!("{e}"); &Value::Null}).as_str().unwrap_or_default();
                                let topic = check_json(data, "topic").unwrap_or_else(|e| { tracing::error!("{e}"); &Value::Null }).as_str().unwrap_or_default();
                                let message_json: Value = serde_json::from_str(&message).unwrap();
                                if let None = message_json.get("viewers").and_then(|s| s.as_u64()) {
                                    if let Some(id_str) = topic.split('.').last() {
                                        let channel_id_to_remove = channel_ids.iter().find(|channel| channel.channel_id == id_str).cloned();
                                        if let Some(to_remove) = channel_id_to_remove {
                                            channel_ids.remove(&to_remove);
                                        }
                                        send_channels.retain(|channel| channel.channel_id != id_str );
                                    }
                                }
                            }

                        },
                        Ok(Message::Ping(ping)) => write.send(Message::Pong(ping)).await.unwrap_or_else(|e| tracing::error!("Failed to send PONG to WebSocket: {e}")),
                        Ok(_) => {},
                        Err(_) => {
                            sleep(Duration::from_secs(UPDATE_TIME)).await;
                            break ;
                        } 
                    }
                }
            }
        }
        
        
    });
}

fn check_json<'a>(v: &'a Value, data: &str) -> Result<&'a Value, Box<dyn Error>> {
    if let Some(key) = v.get(&data) {
        return Ok(key);
    } else {
        return Err(format!("Failed to find '{}' in JSON", data))?;
    }
}

#[derive(PartialEq, Eq, Clone)]
struct Priority {
    priority: u32,
    name: Channel
}

impl Ord for Priority {
    fn cmp (&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

async fn send_now_watched (mut rx: Receiver<BinaryHeap<Priority>>) {
    tokio::spawn(async move {
        loop {
            rx.changed().await.unwrap();
            let watch = rx.borrow().clone();
            let mut now_watch = NOW_WATCHED.lock().await;
            if let Some(max) = watch.peek() {
                debug!("Send: {}", max.name.channel_login);
                *now_watch = Channel { channel_id: max.name.channel_id.to_string(), channel_login: max.name.channel_login.to_string() };
                drop(now_watch);
                debug!("Drop now watch")
            } else {
                *now_watch = Channel::default();
            }

        }

    });
}

pub async fn update_stream (campaigns: Arc<Vec<DropCampaigns>>) {
    tokio::spawn(async move {
        let mut heap: BinaryHeap<Priority> = BinaryHeap::new();
        let mut old_channel_ids: HashSet<Channel> = HashSet::new();
        let (tx, rx) = tokio::sync::watch::channel(BinaryHeap::new());

        send_now_watched(rx).await;

        loop {
            let channel_ids = CHANNEL_IDS.lock().await.clone();
            let allow_channels = ALLOW_CHANNELS.lock().await.clone();
            let default_channels = DEFAULT_CHANNELS.lock().await.clone();

            if channel_ids.is_empty() || (allow_channels.is_empty() && default_channels.is_empty()) {
                sleep(Duration::from_secs(UPDATE_TIME)).await;
                continue;
            }

            let added_channels: Vec<&Channel> = channel_ids.iter().filter(|id| !old_channel_ids.contains(id)).collect();
            debug!("{:?}", added_channels);
            let delete_channels: Vec<&Channel> = old_channel_ids.iter().filter(|id| !channel_ids.contains(id)).collect();

            if !delete_channels.is_empty() {
                let delete_set: HashSet<&Channel> = delete_channels.into_iter().collect();
                let mut new_heap = BinaryHeap::new();
                while let Some(item) = heap.pop() {
                    if !delete_set.iter().any(|channel| channel.channel_id == item.name.channel_id) {
                        new_heap.push(item);
                    }
                }
                heap = new_heap
            }
            if !added_channels.is_empty() {
                for drop_id in campaigns.iter() {
                    for channel in &channel_ids {
                        debug!("{}", channel.channel_id);
                        debug!("{}", drop_id.id);
                        if let Some(allow) = allow_channels.get(&drop_id.id) {
                                if let Some(channel_allow) = allow.iter().find(|s| s.id == *channel.channel_id) {
                                    debug!("Allow {}", channel_allow.name);
                                    heap.push(Priority { priority: 2, name: Channel { channel_id: channel_allow.id.clone(), channel_login: channel_allow.name.clone() } });
                                }
                        }

                        if let Some(default) = default_channels.get(&drop_id.id) {
                            if let Some(channel_default) = default.iter().find(|s| s.broadcaster.id == *channel.channel_id) {
                                debug!("Default {}", channel_default.broadcaster.login);
                                heap.push(Priority { priority: 1, name: Channel { channel_id: channel_default.broadcaster.id.clone(), channel_login: channel_default.broadcaster.login.clone() } });
                            }
                        }

                    } 
                }
                old_channel_ids = channel_ids
            }
            tx.send(heap.clone()).unwrap();
            sleep(Duration::from_secs(UPDATE_TIME)).await;
        }
    });
}