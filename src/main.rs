#[macro_use]
extern crate rocket;
use parking_lot::RwLock;
use rocket::{serde::json::Json, State};
use serde_json::{json, Value};
use std::env::var;
use std::sync::Arc;
mod defs;
use defs::{Team, TeamMember};

impl Team {
    fn from_raw_airtable(input: Value) -> Self {
        let current: Vec<TeamMember> = input
            .get("current")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| TeamMember::from_json(r))
            .collect();

        Self {
            current,
            alumni: vec![],
        }
    }

    fn fetch() -> Self {
        let app_id = var("AT_BASE_ID").expect("an airtable base ID");
        let token = var("AT_TOKEN").expect("an airtable token");
        let slack_token = var("SLACK_TOKEN").expect("a slack token");

        let client = reqwest::blocking::Client::new();

        let res = client
            .get(format!(
                "https://api.airtable.com/v0/{app_id}/Current?view=Grid%20view"
            ))
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .unwrap()
            .text()
            .unwrap();

        let mut res = serde_json::from_str::<Value>(&res).unwrap();
        let res = res
            .get_mut("records")
            .expect("records")
            .as_array_mut()
            .expect("an array of records")
            .iter_mut()
            .map(|r| r.get_mut("fields").expect("fields"))
            .map(|r| {
                if let Some(slack_id) = r.get("Slack ID") {
                    let slack_id = slack_id.as_str().unwrap();

                    let slack_user_response = client
                        .get(format!("https://slack.com/api/users.info?user={slack_id}"))
                        .header("Authorization", format!("Bearer {}", slack_token))
                        .send()
                        .unwrap()
                        .json::<Value>()
                        .unwrap();

                    if let Some(err) = slack_user_response.get("error") {
                        log::error!("slack web api error for slack id {slack_id}: {err}");
                        return r;
                    }

                    let profile = slack_user_response
                        .get("user")
                        .expect("a user")
                        .get("profile")
                        .expect("a profile");

                    let pronouns = profile.get("pronouns").unwrap_or(&Value::Null);

                    let avatar = profile
                        .get("image_72")
                        .expect("an avatar")
                        .as_str()
                        .expect("a str");

                    let slack_display_name = profile
                        .get("display_name")
                        .expect("a display name")
                        .as_str()
                        .expect("a str");

                    log::debug!("pulled Slack data for {slack_display_name}");

                    let r_obj = r.as_object_mut().unwrap();
                    r_obj.insert("_pronouns".into(), pronouns.to_owned());
                    r_obj.insert("_avatar".into(), avatar.into());
                    r_obj.insert("_slack_display_name".into(), slack_display_name.into());
                }

                r
            })
            .collect::<Vec<_>>();

        let res = serde_json::to_value(res).unwrap();

        Self::from_raw_airtable(json!({ "current": res, "alumni": [] }))
    }
}

#[get("/")]
fn get_team(team: &State<RwLock<Team>>) -> Json<Team> {
    Json(team.read().clone())
}

#[post("/?<token..>")]
fn notify_team_change(team: &State<RwLock<Team>>, token: String) -> Json<String> {
    if token != var("TEAM_SERVER_SECRET").expect("a token") {
        return Json(String::from("invalid token"));
    }

    *team.write() = Team::fetch();
    Json(String::from("success!"))
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut config = rocket::config::Config::release_default();
    if !cfg!(debug_assertions) {
        config.address = std::net::IpAddr::from([0, 0, 0, 0]);
    }

    let team = Arc::new(RwLock::new(Team::fetch()));

    let team_thread = team.clone();
    std::thread::spawn(move || {
        let sleep_duration = std::time::Duration::from_secs(10 * 60);
        loop {
            std::thread::sleep(sleep_duration);
            log::debug!("Refetching team data");
            *team_thread.write() = Team::fetch();
        }
    });

    rocket::custom(config)
        .mount("/", routes![get_team, notify_team_change])
        .manage(team)
}
