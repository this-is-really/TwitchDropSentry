use std::error::Error;

use serde_json::{json, Value};

use crate::common::structs::{ClientInfo, Extensions, GQLoperation, PersistedQuery};

impl ClientInfo {
    pub async fn android() -> Result<Self, Box<dyn Error>> {
        let user_agent = String::from("Dalvik/2.1.0 (Linux; U; Android 7.1.2; SM-G977N Build/LMY48Z) tv.twitch.android.app/16.8.1/1608010");
        let client = ClientInfo {
            url: "https://www.twitch.tv".to_string(),
            id: "kd1unb4b3q4t58fwlpcbzcbnm76a8fp".to_string(),
            user_agent: user_agent,
        };
        Ok(client)
    }

    pub async fn web() -> Result<Self, Box<dyn Error>> {
        let user_agent = String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36");
        let client = ClientInfo {
            url: "https://www.twitch.tv".to_string(),
            id: "kimne78kx3ncx6brgo4mv6wki5h1ko".to_string(),
            user_agent: user_agent,
        };
        Ok(client)
    }
}

impl GQLoperation {
    pub async fn get_stream_info (channel_login: &String) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "channel": channel_login
        });
        let info = GQLoperation {
            operationName: "VideoPlayerStreamInfoOverlayChannel".to_string(),
            extensions: Extensions {
                persistedQuery: PersistedQuery {
                    version: 1,
                    sha256Hash: "198492e0857f6aedead9665c81c5a06d67b25b58034649687124083ff288597d".to_string()
                }
            },
            variables: Some(variables)
        };
        Ok(info)
    }
    pub async fn inventory (fetchrewardcampaigns: bool) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "fetchRewardCampaigns": fetchrewardcampaigns,
        });
        let info = GQLoperation {
            operationName: "Inventory".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "09acb7d3d7e605a92bdfdcc465f6aa481b71c234d8686a9ba38ea5ed51507592".to_string() }
            },
            variables: Some(variables)
        };
        Ok(info)
    }

    pub async fn campaigns (fetchrewardcampaigns: bool) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "fetchRewardCampaigns": fetchrewardcampaigns,
        });
        let info = GQLoperation {
            operationName: "ViewerDropsDashboard".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "5a4da2ab3d5b47c9f9ce864e727b2cb346af1e3ea8b897fe8f704a97ff017619".to_string() } 
            },
            variables: Some(variables)
        };
        Ok(info)
    }

    pub async fn campaigndetails (user_login: &String, dropid: &String) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "channelLogin": user_login,
            "dropID": dropid,
        });
        let info = GQLoperation {
            operationName: "DropCampaignDetails".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "039277bf98f3130929262cc7c6efd9c141ca3749cb6dca442fc8ead9a53f77c1".to_string() } 
            },
            variables: Some(variables)
        };
        Ok(info)
    }

    pub async fn currentdrop (channel_id: String) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "channelID": channel_id,
            "channelLogin": "",
        });
        let info = GQLoperation {
            operationName: "DropCurrentSessionContext".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "4d06b702d25d652afb9ef835d2a550031f1cf762b193523a92166f40ea3d142b".to_string() }
            },
            variables: Some(variables)
        };
        Ok(info)
    }

    pub async fn playbackaccesstoken (channel_login: &String) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "isLive": true,
            "isVod": false,
            "login": channel_login,
            "platform": "web",
            "playerType": "site",
            "vodID": "",
        });
        let info = GQLoperation {
            operationName: "PlaybackAccessToken".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "ed230aa1e33e07eebb8928504583da78a5173989fadfb1ac94be06a04f3cdbe9".to_string() } 
            },
            variables: Some(variables)
        };
        Ok(info)
    }

    pub async fn gamedirectory (game_slug: &String) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "limit": 30,
            "slug": game_slug,
            "imageWidth": 50,
            "includeIsDJ": false,
            "options": {
                "broadcasterLanguages": [],
                "freeformTags": Value::Null,
                "includeRestricted": ["SUB_ONLY_LIVE"],
                "recommendationsContext": {"platform": "web"},
                "sort": "RELEVANCE",
                "systemFilters": ["DROPS_ENABLED"],
                "tags": [],
                "requestID": "JIRA-VXP-2397",
            },
            "includeIsDJ": false,
            "sortTypeIsRecency": false,
        });
        let info = GQLoperation {
            operationName: "DirectoryPage_Game".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "c7c9d5aad09155c4161d2382092dc44610367f3536aac39019ec2582ae5065f9".to_string() } 
            },
            variables: Some(variables)
        };
        Ok(info)
    }

    pub async fn slugredirect (game_name: &String) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "name": game_name
        });
        let info = GQLoperation {
            operationName: "DirectoryGameRedirect".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "1f0300090caceec51f33c5e20647aceff9017f740f223c3c532ba6fa59f6b6cc".to_string() }
            },
            variables: Some(variables)
        };
        Ok(info)
    }

    pub async fn claimdrop (claim_id: &String) -> Result<Self, Box<dyn Error>> {
        let variables = json!({
            "input": {
                "dropInstanceID": claim_id
            }
        });
        let info = GQLoperation {
            operationName: "DropsPage_ClaimDropRewards".to_string(),
            extensions: Extensions { 
                persistedQuery: PersistedQuery { version: 1, sha256Hash: "a455deea71bdc9015b78eb49f4acfbce8baa7ccbedd28e549bb025bd0f751930".to_string() }
            },
            variables: Some(variables)
        };
        Ok(info)
    }
}

