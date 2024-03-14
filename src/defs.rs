use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamMember {
    pub name: String,
    bio: Option<String>,
    department: String,
    role: String,
    bio_hackfoundation: Option<String>,
    pronouns: Option<String>,
    pub slack_id: Option<String>,
    slack_display_name: Option<String>,
    avatar: Option<String>, // 72^2 px if from Slack; ..=512^2 if overriden in Airtable
    avatar_id: Option<String>,
    email: Option<String>,
}
impl TeamMember {
    pub fn from_json(json: &Value) -> Self {
        // Title Case names are from Airtable, _snake_case names are from Slack.

        let avatar_object = json
            .get("Override Avatar")
            .and_then(|s| s.as_array())
            .and_then(|s| s.get(0))
            .and_then(|s| s.get("thumbnails"))
            .and_then(|s| s.get("large"));

        Self {
            name: json["Name"].as_str().unwrap().into(),
            bio: json["Bio"].as_str().map(|s| s.into()),
            department: json["Department"].as_str().unwrap().into(),
            role: json["Role"].as_str().unwrap().into(),
            bio_hackfoundation: json["Bio (Hack Foundation)"].as_str().map(|s| s.into()),
            pronouns: json
                .get("_pronouns")
                .and_then(|s| s.as_str().map(|s| s.to_string())),
            slack_id: json
                .get("Slack ID")
                .and_then(|s| s.as_str().map(|s| s.to_string())),
            slack_display_name: json
                .get("_slack_display_name")
                .map(|s| s.as_str().unwrap().to_string()),
            avatar: avatar_object
                .and_then(|s| s.get("url"))
                .map(|s| s.as_str().unwrap().to_string())
                .or(json.get("_avatar").map(|s| s.as_str().unwrap().to_string())),
            avatar_id: avatar_object
                .and_then(|s| s.get("id"))
                .map(|s| s.as_str().unwrap().to_string())
                .or(Some(String::new())),
            email: json["Email"].as_str().map(|s| s.into()),
        }
    }

    pub fn differences(&self, other: &Self) -> Vec<&str> {
        let mut diffs = vec![];

        if self.name != other.name {
            diffs.push("name")
        }
        if self.bio != other.bio {
            diffs.push("bio")
        }
        if self.department != other.department {
            diffs.push("department")
        }
        if self.role != other.role {
            diffs.push("role")
        }
        if self.bio_hackfoundation != other.bio_hackfoundation {
            diffs.push("alternate bio")
        }
        if self.avatar_id != other.avatar_id {
            diffs.push("avatar")
        }
        if self.email != other.email {
            diffs.push("email")
        }

        diffs
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
