use std::{sync::Arc, time::Duration};

use tokio::time::sleep;
use twitch_gql_rs::{structs::Channels, TwitchClient};

use crate::r#static::{NowWatched, NOW_WATCHED};

const UPDATE_TIME: u64 = 15;

pub async fn check_with_allow (client: Arc<TwitchClient>, allow: Vec<Channels>) {
    tokio::spawn(async move {
        for channel in allow {
            loop {
                let stream_info = client.get_stream_info(&channel.name).await.unwrap();
                if let Some(stream) = stream_info.stream {
                    let mut watching = NOW_WATCHED.lock().await;
                    *watching = NowWatched { channel_login: stream_info.login, stream_id: stream.id, channel_id: stream_info.id };
                    drop(watching);
                    sleep(Duration::from_secs(UPDATE_TIME)).await;
                } else {
                   break;
                }
            }
        }
    });
}

pub async fn default_check_streams (client: Arc<TwitchClient>, game_name: String) {
    tokio::spawn(async move {
        let slug = client.get_slug(&game_name).await.unwrap();
        let game_directory = client.get_game_directory(&slug, true).await.unwrap();
        for channel in game_directory {
            loop {
                let stream_info = client.get_stream_info(&channel.broadcaster.login).await.unwrap();
                if let Some(stream) = stream_info.stream {
                    let mut watching = NOW_WATCHED.lock().await;
                    *watching = NowWatched { channel_login: stream_info.login, channel_id: stream_info.id, stream_id: stream.id };
                    drop(watching);
                    sleep(Duration::from_secs(UPDATE_TIME)).await
                } else {
                    break;
                }
            }
        }
    });
}