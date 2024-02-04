use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamMember {
    name: String,
    bio: Option<String>,
    department: String,
    role: String,
    bio_hackfoundation: Option<String>,
    pronouns: Option<String>,
    slack_display_name: Option<String>,
    avatar: Option<String>, // 72^2 px if from Slack; ..=512^2 if overriden in Airtable
}
impl TeamMember {
    pub fn from_json(json: &Value) -> Self {
        // Title Case names are from Airtable, _snake_case names are from Slack.
        Self {
            name: json["Name"].as_str().unwrap().into(),
            bio: json["Bio"].as_str().map(|s| s.into()),
            department: json["Department"].as_str().unwrap().into(),
            role: json["Role"].as_str().unwrap().into(),
            bio_hackfoundation: json["Bio (Hack Foundation)"].as_str().map(|s| s.into()),
            pronouns: json
                .get("_pronouns")
                .and_then(|s| s.as_str().map(|s| s.to_string())),
            slack_display_name: json
                .get("_slack_display_name")
                .map(|s| s.as_str().unwrap().to_string()),
            avatar: json
                .get("Override Avatar")
                .and_then(|s| s.as_array())
                .and_then(|s| s.get(0))
                .and_then(|s| s.get("thumbnails"))
                .and_then(|s| s.get("large"))
                .and_then(|s| s.get("url"))
                .map(|s| s.as_str().unwrap().to_string())
                .or(json.get("_avatar").map(|s| s.as_str().unwrap().to_string())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team {
    pub version: &'static str,
    pub refreshed_at: u64,
    pub current: Vec<TeamMember>,
    pub alumni: Vec<TeamMember>,
}

#[derive(Debug)]
pub struct TeamFetchError {
    message: String,
}
impl TeamFetchError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
    pub fn log(&self) {
        log::error!("{}", self.message);
    }
}
impl std::fmt::Display for TeamFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TeamFetchError: {}", self.message)
    }
}
impl std::error::Error for TeamFetchError {}
