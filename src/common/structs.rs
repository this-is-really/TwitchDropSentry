use serde::{Deserialize, Serialize};
use serde_json::Value;

// common/constants
#[derive(Debug)]
pub struct ClientInfo {
    pub url: String,
    pub id: String,
    pub user_agent: String,
}

// common/constants
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GQLoperation {
    pub operationName: String,
    pub extensions: Extensions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Extensions {
    pub persistedQuery: PersistedQuery,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct PersistedQuery {
    pub version: i32,
    pub sha256Hash: String,
}

// token/load_or_create_token
#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub userid: String,
    pub oauth: String,
}
// list_active_games
pub struct Campaigns {
    pub game_display_name: String,
    pub game_id: String,
    pub campaign_id: String,
}

// api/get_campaign_details
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub data: UserData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub user: User,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub dropCampaign: DropCampaign,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct DropCampaign {
    pub id: String,
    pub name: String,
    pub status: String,
    pub startAt: String,
    pub endAt: String,
    pub game: Game,
    pub timeBasedDrops: Vec<TimeBasedDrop>,
    pub allow: AllowList,
}

// DropCampaign
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub slug: String,
    pub displayName: String,
}


// DropCampaign
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeBasedDrop {
    pub requiredMinutesWatched: u32,
    pub benefitEdges: Vec<BenefitEdge>,
}

// TimeBasedDrop
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BenefitEdge {
    pub benefit: Benefit,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Benefit {
    pub id: String,
    pub game: BenefitGame,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenefitGame {
    id: String,
    name: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AllowList {
    pub isEnabled: bool,
    #[serde(default)]
    pub channels: Option<Vec<Channel>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub displayName: String,
    pub name: String,
}