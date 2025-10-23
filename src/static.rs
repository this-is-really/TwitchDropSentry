use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

#[derive(Default, Clone)]
pub struct NowWatched {
    pub channel_login: String,
    pub channel_id: String,
    pub stream_id: String,
}

pub static NOW_WATCHED: Lazy<Arc<Mutex<NowWatched>>> = Lazy::new(|| Arc::new(Mutex::new(NowWatched::default())));

pub static DROP_CASH: Lazy<Arc<Mutex<Vec<String>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));