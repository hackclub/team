#[macro_use]
extern crate rocket;
use parking_lot::RwLock;
use rocket::{serde::json::Json, State};
use serde_json::{json, Value};
use std::env::var;
use std::sync::Arc;
mod defs;
use defs::{Team, TeamFetchError, TeamMember};
mod slack;

impl Team {
    fn from_raw_airtable(refreshed_at: u64, input: Value) -> Self {
        let current: Vec<TeamMember> = input
            .get("current")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| TeamMember::from_json(r))
            .collect();

        Self {
            version: env!("CARGO_PKG_VERSION"),
            refreshed_at,

            current,
            alumni: vec![],
        }
    }

    fn fetch() -> Result<Self, TeamFetchError> {
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
            .text();

        if let Err(ref err) = res {
            return Err(TeamFetchError::new(format!(
                "could not make a request to the airtable api: {err}"
            )));
        }
        let res = res.unwrap();

        let mut res = serde_json::from_str::<Value>(&res).unwrap();
        let mut res = res
            .get_mut("records")
            .expect("records")
            .as_array_mut()
            .expect("an array of records")
            .iter_mut()
            .map(|r| r.get_mut("fields").expect("fields"))
            .collect::<Vec<_>>();

        for r in res.iter_mut() {
            if let Some(slack_id) = r.get("Slack ID") {
                let slack_id = slack_id.as_str().unwrap();

                let slack_user_response = client
                    .get(format!(
                        "https://slack.com/api/users.profile.get?user={slack_id}"
                    ))
                    .header("Authorization", format!("Bearer {}", slack_token))
                    .send();

                if let Err(ref err) = slack_user_response {
                    return Err(TeamFetchError::new(format!(
                        "could not make a request to the slack web api: {err}"
                    )));
                }

                let slack_user_response = slack_user_response.unwrap().json::<Value>().unwrap();

                if let Some(err) = slack_user_response.get("error") {
                    log::error!("slack web api error for slack id {slack_id}: {err}");
                    return Err(TeamFetchError::new(format!(
                        "there was an error in the slack api response: {err}"
                    )));
                }

                let profile = slack_user_response.get("profile").expect("a profile");

                let pronouns = profile.get("pronouns").unwrap_or(&Value::Null);

                let avatar_hash = profile
                    .get("avatar_hash")
                    .expect("an avatar hash")
                    .as_str()
                    .expect("a str");
                let avatar =
                    format!("https://ca.slack-edge.com/T0266FRGM-{slack_id}-{avatar_hash}-128");

                let slack_display_name = profile
                    .get("display_name")
                    .expect("a display name")
                    .as_str()
                    .expect("a str");

                log::trace!("{:?}", profile);
                log::debug!("pulled Slack data for {slack_display_name}");

                let r_obj = r.as_object_mut().unwrap();
                r_obj.insert("_pronouns".into(), pronouns.to_owned());
                r_obj.insert("_avatar".into(), avatar.into());
                r_obj.insert("_slack_display_name".into(), slack_display_name.into());
            }
        }

        let res = serde_json::to_value(res).unwrap();

        let refreshed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        Ok(Self::from_raw_airtable(
            refreshed_at,
            json!({ "current": res, "alumni": [] }),
        ))
    }
}

#[get("/")]
fn get_team(team: &State<Arc<RwLock<Team>>>) -> Json<Team> {
    Json(team.read().clone())
}

#[post("/?<token..>")]
fn notify_team_change(team: &State<Arc<RwLock<Team>>>, token: String) -> Json<String> {
    if token != var("TEAM_SERVER_SECRET").expect("a token") {
        return Json(String::from("invalid token"));
    }

    match Team::fetch() {
        Ok(t) => {
            let current_team = team.read();

            for current_member in (*current_team.current).iter() {
                if let Some(other_member) = t.current.iter().find(|m| m.name == current_member.name)
                {
                    let changed_fields = current_member.differences(other_member);
                    if !changed_fields.is_empty() {
                        if let Some(ref sid) = current_member.slack_id {
                            if slack::send_slack_message(
                                sid,
                                &format!(
                                    "The following fields have changed for you: {}",
                                    changed_fields.join(", ")
                                ),
                            )
                            .is_err()
                            {
                                log::error!(
                                    "failed to send row change slack notification to {}",
                                    current_member.name
                                );
                            }
                        }
                    }
                }
            }

            // for changed_member in team.read().changed_members(&t) {
            //     if let Some(ref sid) = changed_member.slack_id {
            //         let other_member = other
            //             .current
            //             .iter()
            //             .find(|m| m.name == changed_member.name)
            //             .unwrap_or_else(|| {
            //                 log::warn!("Member {} not found in other team", member.name);
            //                 member
            //             });

            //         let changed = changed_member.clone().differences(t);
            //         if slack::send_slack_message(sid, "test1").is_err() {
            //             log::error!(
            //                 "failed to send row change slack notification to {}",
            //                 changed_member.name
            //             );
            //         }
            //     }
            // }

            *team.write() = t;
            Json(String::from("success!"))
        }
        Err(err) => Json(String::from("failure")),
    }
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut config = rocket::config::Config::release_default();
    if !cfg!(debug_assertions) {
        config.address = std::net::IpAddr::from([0, 0, 0, 0]);
    }

    let team = match Team::fetch() {
        Ok(t) => {
            log::info!("finished fetching team data");
            Arc::new(RwLock::new(t))
        }
        Err(err) => {
            log::error!("failed to fetch team data on startup: {err}");
            std::process::exit(1);
        }
    };

    let team_thread = team.clone();
    std::thread::spawn(move || {
        let sleep_duration = std::time::Duration::from_secs(10 * 60);
        loop {
            std::thread::sleep(sleep_duration);
            log::debug!("Refetching team data");
            match Team::fetch() {
                Ok(t) => {
                    *team_thread.write() = t;
                }
                Err(err) => {
                    log::error!("failed to refetch team data: {err}");
                }
            }
        }
    });

    log::info!(
        "starting rocket server on {}:{}",
        config.address,
        config.port
    );

    rocket::custom(config)
        .mount("/", routes![get_team, notify_team_change])
        .manage(team)
}
