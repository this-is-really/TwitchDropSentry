use std::{collections::{BTreeMap, HashMap}, error::Error, path::{Path, PathBuf}, sync::Arc, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::{fs, sync::watch::Sender, time::{Instant, sleep}};
use tracing::{debug, info};
use twitch_gql_rs::{TwitchClient, client_type::ClientType, structs::{DropCampaigns}};

use crate::{r#static::{Channel, DROP_CASH, NOW_WATCHED, retry_backup}, stream::{filter_streams, update_stream}};
mod r#static;
mod stream;

const STREAM_SLEEP: u64 = 20;

async fn create_client (home_dir: &Path) -> Result<TwitchClient, Box<dyn Error>> {
    let path = home_dir.join("save.json");
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
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    let home_dir = Path::new("data");
    if !home_dir.exists() {
        fs::create_dir_all(&home_dir).await?;
    }

    let client = create_client(home_dir).await?;

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

    main_logic(Arc::new(client), grouped, home_dir).await?;
    Ok(())
}

async fn main_logic (client: Arc<TwitchClient>, grouped: BTreeMap<usize, Vec<DropCampaigns>>, home_dir: &Path) -> Result<(), Box<dyn Error>> {
    let input: usize = dialoguer::Input::new().with_prompt("Select game").interact_text()?;
    if let Some(current_campaigns) = grouped.get(&input) {

        let (tx, mut rx) = tokio::sync::watch::channel(String::new());
        let drop_campaigns = Arc::new(current_campaigns.clone());
        
        let drop_cash_dir = home_dir.join("cash.json");

        watch_sync(client.clone()).await;
        info!("Watch synchronization task has been successfully initiated");
        drop_sync(client.clone(), tx, drop_cash_dir).await;
        info!("Drop progress tracker is active");
        filter_streams(client.clone(), drop_campaigns.clone()).await;
        info!("Stream filtering has begun");
        update_stream(drop_campaigns).await;
        info!("Stream priority updated");

        for campaign in current_campaigns {
            if campaign.status == "EXPIRED" {
                tracing::error!("Drop is EXPIRED");
                break;
            }

            let mut campaign_details = client.get_campaign_details(&campaign.id).await?;

            let drop_ids_cache = DROP_CASH.lock().await.clone();    
            for drop_id_cache in drop_ids_cache {
                let deleted_time_based = campaign_details.timeBasedDrops.iter().filter(|time_based| time_based.id == drop_id_cache).map(|time_based| time_based.id.clone()).collect::<Vec<String>>();
                for delete in deleted_time_based {
                    if let Some(pos) = campaign_details.timeBasedDrops.iter().position(|time_based| time_based.id == delete) {
                        campaign_details.timeBasedDrops.remove(pos);
                    }
                }
            }

            loop {
                rx.changed().await.unwrap();
                let drop_id = rx.borrow();
                if drop_id.is_empty() {
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
                if campaign_details.timeBasedDrops.is_empty() {
                    break;
                }
                if let Some(pos) = campaign_details.timeBasedDrops.iter().position(|time_based| time_based.id == *drop_id) {
                    campaign_details.timeBasedDrops.remove(pos);
                }
            }
            
        }
    }
    Ok(())
}

async fn watch_sync (client: Arc<TwitchClient>) {
    tokio::spawn(async move {
        let mut old_stream_name = String::new();
        let mut stream_id = String::new();
        loop {
            let watching = NOW_WATCHED.lock().await.clone();

            if watching == Channel::default() {
                drop(watching);
                sleep(Duration::from_secs(STREAM_SLEEP)).await;
                continue;
            }

            if old_stream_name.is_empty() || old_stream_name != watching.channel_login {
                info!("Now actively watching channel {}", watching.channel_login);
                old_stream_name = watching.channel_login.clone();
                stream_id.clear();
            }

            if stream_id.is_empty() {
                let stream = retry!(client.get_stream_info(&watching.channel_login));
                if let Some(id) = stream.stream {
                    stream_id = id.id
                } else {
                    sleep(Duration::from_secs(STREAM_SLEEP)).await;
                    continue;
                }
            }

            match client.send_watch(&watching.channel_login, &stream_id, &watching.channel_id).await {
                Ok(_) => {
                    drop(watching);
                    sleep(Duration::from_secs(STREAM_SLEEP)).await
                },
                Err(e) => {
                    drop(watching);
                    tracing::error!("{e}");
                    sleep(Duration::from_secs(STREAM_SLEEP)).await;
                }
            }
        }
    });
}

async fn drop_sync (client: Arc<TwitchClient>, tx: Sender<String>, cash_path: PathBuf) {
    tokio::spawn(async move {
        let mut end_time = Instant::now() + Duration::from_secs(60*60);
        let mut old_drop = String::new();

        //bar
        let bar = ProgressBar::new(1);
        bar.set_style(ProgressStyle::with_template("[{bar:40.cyan/blue}] {percent:.1}% ({pos}/{len} min) {msg}").unwrap());
        bar.set_message("Initialization...");

        if !cash_path.exists() {
            retry!(fs::write(&cash_path, "[]"));
        } else {
            let mut cash = DROP_CASH.lock().await;
            let cash_str = retry!(fs::read_to_string(&cash_path));
            let cash_vec: Vec<String> = serde_json::from_str(&cash_str).unwrap();  
            *cash = cash_vec;
            drop(cash);
        }

        let tolerance = Duration::from_secs(5 * 60);
        loop {
            let watching = NOW_WATCHED.lock().await.clone();
            debug!("Lock watching in drop");

            if watching == Channel::default() {
                sleep(Duration::from_secs(30)).await;
                continue;
            }

            let mut cash = DROP_CASH.lock().await;

            let drop_progress = retry!(client.get_current_drop_progress_on_channel(&watching.channel_login, &watching.channel_id));
            drop(watching);
            if drop_progress.currentMinutesWatched == 0 {
                drop(cash);
                sleep(Duration::from_secs(30)).await;
                continue;
            }

            if old_drop.is_empty() {
                old_drop = drop_progress.dropID.to_string()
            }

            let mut need_update = false;

            if end_time <= Instant::now() || old_drop != drop_progress.dropID && !cash.contains(&drop_progress.dropID) {
                retry!(claim_drop(&client, &old_drop));
                info!("Drop claimed: {}", old_drop);
                tx.send(old_drop.to_string()).unwrap();
                cash.push(old_drop.to_string());
                old_drop = drop_progress.dropID.to_string();
                need_update = true;

                let cash_string_writer = serde_json::to_string_pretty(&cash.clone()).unwrap();
                retry!(fs::write(&cash_path, cash_string_writer.clone()))
            }
            drop(cash);


            if bar.length().unwrap_or(0) == 1 {
                bar.set_length(drop_progress.requiredMinutesWatched);
                bar.set_message(format!("DropID: {}", drop_progress.dropID));
                need_update = true
            }

            if need_update || drop_progress.currentMinutesWatched != bar.position() {
                bar.set_length(drop_progress.requiredMinutesWatched);
                bar.set_position(drop_progress.currentMinutesWatched);
                bar.tick();
            }

            if end_time <= Instant::now() + tolerance || Instant::now() <= end_time + tolerance && need_update  {
                let reaming = drop_progress.requiredMinutesWatched.saturating_sub(drop_progress.currentMinutesWatched);
                end_time = Instant::now() + Duration::from_secs(reaming * 60);
            }

            sleep(Duration::from_secs(30)).await;
        }
       
    });
}

async fn claim_drop (client: &Arc<TwitchClient>, drop_progress_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        let inv = retry!(client.get_inventory());
        for in_progress in inv.inventory.dropCampaignsInProgress {
            for time_based in in_progress.timeBasedDrops {
                if time_based.id == drop_progress_id {
                    if let Some(id) = time_based.self_drop.dropInstanceID {
                        retry!(client.claim_drop(&id));
                        return Ok(());
                    }
                }
            }
        }
    }
}
