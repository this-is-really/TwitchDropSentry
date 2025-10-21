use std::{collections::{BTreeMap, HashMap}, error::Error, path::Path, time::Duration};

use tokio::time::{sleep, Instant};
use twitch_gql_rs::{client_type::ClientType, structs::DropCampaigns, TwitchClient};

#[tokio::main]
async fn main () -> Result<(), Box<dyn Error>> {
    let path = Path::new("save.json");
    if !path.exists() {
        let client_type = ClientType::android_app();
        let mut client = TwitchClient::new(&client_type).await?;
        client.auth().await?;
        client.save_file(&path).await?;
    }
    let client = TwitchClient::load_from_file(&path).await?;
    let campaign = client.get_campaign().await?;
    let campaign = campaign.dropCampaigns;

    let mut id_to_index = HashMap::new();
    let mut grouped: BTreeMap<usize, Vec<DropCampaigns>> = BTreeMap::new();
    let mut next_index: usize = 0;
    for obj in campaign {
        let idx = *id_to_index.entry(obj.game.id.clone()).or_insert_with(|| {
            let i = next_index;
            next_index += 1;
            i
        });

        grouped.entry(idx).or_default().push(obj);
    }

    for (id, obj) in &grouped {
        for i in obj {
            println!("{} | {}", id, i.game.displayName);
        }
    }

    let input: usize = dialoguer::Input::new().with_prompt("Выбери компанию").interact_text()?;
    if let Some(campaigns) = grouped.get(&input) {
        for campaign in campaigns {
            let campaign_details = client.get_campaign_details(&campaign.id).await?;
            'time_based_loop: for time_based in campaign_details.timeBasedDrops {
                let need_watch = time_based.requiredMinutesWatched;
                let start_at = Instant::now();
                let mut end_time = start_at + Duration::from_secs(need_watch * 60);
                if let Some(channels) = &campaign_details.allow.channels {
                    for channel in channels {
                        let stream_info = client.get_stream_info(&channel.name).await?;
                        if let Some(stream) = stream_info.stream {
                            loop {
                                let is_online = client.get_stream_info(&stream_info.login).await?;
                                if let None = is_online.stream {
                                    continue;
                                };
                                let get_progress = client.get_current_drop_progress_on_channel(&stream_info.login, &stream_info.id).await?;
                                let elapsed = start_at.elapsed();
                                if get_progress.currentMinutesWatched * 60 != end_time.saturating_duration_since(start_at + elapsed).as_secs() {
                                    end_time -= Duration::from_secs(get_progress.currentMinutesWatched * 60);
                                }
                                if Instant::now() >= end_time {
                                    let inventory = client.get_inventory().await?;
                                    let current_campaign = inventory.inventory.dropCampaignsInProgress.iter().find(|s| s.id == campaign_details.id).unwrap();
                                    let current_time_based = current_campaign.timeBasedDrops.iter().find(|s| s.id == time_based.id).unwrap();
                                    if let Some(instanece_id) = &current_time_based.self_drop.dropInstanceID {
                                        client.claim_drop(&instanece_id).await?;
                                        continue 'time_based_loop;
                                    } else {
                                        sleep(Duration::from_secs(5)).await
                                    }
                                }
                                match client.send_watch(&stream_info.login, &stream.id, &stream_info.id).await {
                                    Ok(_) => sleep(Duration::from_secs(20)).await,
                                    Err(e) => return Err(e)?
                                }
                            }
                        } else {
                            continue;
                        }
                    }
                } else {
                    let slug = client.get_slug(&campaign.game.displayName).await?;
                    let streams = client.get_game_directory(&slug, true).await?;
                    for stream in streams {
                        loop {
                            let is_online = client.get_stream_info(&stream.broadcaster.login).await?;
                            if let None = is_online.stream {
                                continue;
                            };
                            let get_progress = client.get_current_drop_progress_on_channel(&stream.broadcaster.login, &stream.broadcaster.id).await?;
                            let elapsed = start_at.elapsed();
                            if get_progress.currentMinutesWatched * 60 != end_time.saturating_duration_since(start_at + elapsed).as_secs() {
                                end_time -= Duration::from_secs(get_progress.currentMinutesWatched * 60);
                            }
                            if Instant::now() >= end_time {
                                let inventory = client.get_inventory().await?;
                                let current_campaign = inventory.inventory.dropCampaignsInProgress.iter().find(|s| s.id == campaign_details.id).unwrap();
                                let current_time_based = current_campaign.timeBasedDrops.iter().find(|s| s.id == time_based.id).unwrap();
                                if let Some(instanece_id) = &current_time_based.self_drop.dropInstanceID {
                                    client.claim_drop(&instanece_id).await?;
                                    continue 'time_based_loop;
                                } else {
                                    sleep(Duration::from_secs(5)).await
                                }
                            }
                            match client.send_watch(&stream.broadcaster.id, &stream.id, &stream.broadcaster.id).await {
                                Ok(_) => sleep(Duration::from_secs(20)).await,
                                Err(e) => return Err(e)?
                            }
                        }
                    }
                }
            }
            
        }
    }
    Ok(())
}
