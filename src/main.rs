use std::{collections::{BTreeMap, HashMap}, error::Error, path::Path, sync::Arc, time::Duration};

use tokio::{sync::Notify, time::{sleep, Instant}};
use twitch_gql_rs::{client_type::ClientType, structs::{DropCampaigns}, TwitchClient};

use crate::{r#static::{DROP_CASH, NOW_WATCHED}, stream::{check_with_allow, default_check_streams}};
mod r#static;
mod stream;

const STREAM_SLEEP: u64 = 20;

async fn create_client () -> Result<TwitchClient, Box<dyn Error>> {
    let path = Path::new("save.json");
    if !path.exists() {
        let client_type = ClientType::android_app();
        let mut client = TwitchClient::new(&client_type).await?;
        let get_auth = client.request_device_auth().await?;
        println!("{}", get_auth.verification_uri);
        client.auth(get_auth).await?;
        client.save_file(&path).await?;
    }
    let client = TwitchClient::load_from_file(&path).await?;
    Ok(client)
}

#[tokio::main]
async fn main () -> Result<(), Box<dyn Error>> {
    let client = create_client().await?;

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

    main_logic(Arc::new(client), grouped).await?;
    Ok(())
}

async fn main_logic (client: Arc<TwitchClient>, grouped: BTreeMap<usize, Vec<DropCampaigns>>) -> Result<(), Box<dyn Error>> {
    let input: usize = dialoguer::Input::new().with_prompt("Select game").interact_text()?;
    if let Some(current_campaigns) = grouped.get(&input) {

        let notify = Arc::new(Notify::new());

        watch_sync(client.clone()).await;
        drop_sync(client.clone(), notify.clone()).await;

        for campaign in current_campaigns {
            let campaign_details = client.get_campaign_details(&campaign.id).await?;
            for _ in campaign_details.timeBasedDrops {
                if let Some(allow_channels) = &campaign_details.allow.channels {
                    check_with_allow(client.clone(), allow_channels.clone()).await;
                } else {
                    default_check_streams(client.clone(), campaign.game.displayName.clone()).await;
                };
                notify.notified().await;
            }
        }
    }
    Ok(())
}

async fn watch_sync (client: Arc<TwitchClient>) {
    tokio::spawn(async move {
        loop {
            let watching = NOW_WATCHED.lock().await.clone();
            if watching.channel_id.is_empty() {
                sleep(Duration::from_secs(STREAM_SLEEP)).await;
                continue;
            }
            match client.send_watch(&watching.channel_login, &watching.stream_id, &watching.channel_id).await {
                Ok(_) => {
                    println!("{}", watching.channel_login);
                    sleep(Duration::from_secs(STREAM_SLEEP)).await
                },
                Err(e) => println!("{e}")
            }
        }
    });
}

async fn drop_sync (client: Arc<TwitchClient>, notify: Arc<Notify>) {
    tokio::spawn(async move {
        let mut end_time = Instant::now() + Duration::from_secs(60*60);
        loop {
            let watching = NOW_WATCHED.lock().await.clone();

            if watching.stream_id.is_empty() {
                sleep(Duration::from_secs(30)).await;
                continue;
            }

            let mut cash = DROP_CASH.lock().await;
            let drop_progress = client.get_current_drop_progress_on_channel(&watching.channel_login, &watching.channel_id).await.unwrap();
            println!("{}", drop_progress.dropID);
            if !cash.contains(&drop_progress.dropID) || end_time <= Instant::now() {
                claim_drop(&client, &drop_progress.dropID).await.unwrap();
                notify.notify_one();
                cash.push(drop_progress.dropID.to_string());
            }
            drop(cash);
            let reaming = drop_progress.requiredMinutesWatched.saturating_sub(drop_progress.currentMinutesWatched);
            end_time = Instant::now() + Duration::from_secs(reaming * 60);
            sleep(Duration::from_secs(30)).await;
        }
       
    });
}

async fn claim_drop (client: &Arc<TwitchClient>, drop_progress_id: &str) -> Result<(), Box<dyn Error>> {
    let inv = client.get_inventory().await.unwrap();
        for in_progress in inv.inventory.dropCampaignsInProgress {
            for time_based in in_progress.timeBasedDrops {
                if time_based.id == drop_progress_id {
                    if let Some(id) = time_based.self_drop.dropInstanceID {
                        client.claim_drop(&id).await.unwrap();
                    }
                }
            }
        }
    Ok(())
}
