#[macro_use]
extern crate rocket;
use parking_lot::RwLock;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env::var;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TeamMember {
    name: String,
    bio: Option<String>,
    department: String,
    role: String,
    bio_hackfoundation: Option<String>,
    pronouns: String,
    slack_id: Option<String>,
}
impl TeamMember {
    fn from_json(json: &Value) -> Self {
        Self {
            name: json["Name"].as_str().unwrap().into(),
            bio: json["Bio"].as_str().map(|s| s.into()),
            department: json["Department"].as_str().unwrap().into(),
            role: json["Role"].as_str().unwrap().into(),
            bio_hackfoundation: json["Bio (Hack Foundation)"].as_str().map(|s| s.into()),
            pronouns: json["Pronouns"].as_str().unwrap().into(),
            slack_id: json["Slack ID"].as_str().map(|s| s.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Team {
    current: Vec<TeamMember>,
    alumni: Vec<TeamMember>,
}
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

        // let alumni: Vec<TeamMember> = input
        //     .get("alumni")
        //     .unwrap()
        //     .as_array()
        //     .unwrap()
        //     .iter()
        //     .map(|r| TeamMember::from_json(r))
        //     .collect();

        Self {
            current,
            alumni: vec![],
        }
    }

    fn fetch() -> Self {
        let app_id = var("AT_BASE_ID").expect("an airtable base ID");
        let token = var("AT_TOKEN").expect("an airtable token");

        let res = reqwest::blocking::Client::new()
            .get(format!("https://api.airtable.com/v0/{app_id}/Current"))
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .unwrap()
            .text()
            .unwrap();

        let res = serde_json::from_str::<Value>(&res).unwrap();
        let res = res
            .get("records")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.get("fields").unwrap())
            .collect::<Vec<_>>();

        // Convert to Value
        let res = serde_json::to_value(res).unwrap();

        Self::from_raw_airtable(json!({ "current": res, "alumni": [] }))
    }
}

#[get("/")]
fn get_team(team: &State<RwLock<Team>>) -> Json<Team> {
    Json(team.read().clone())
}

#[post("/?<token..>", format = "json", data = "<input>")]
fn update_team(team: &State<RwLock<Team>>, input: Json<Value>, token: String) -> Json<String> {
    if token != var("TEAM_SERVER_SECRET").expect("a token") {
        return Json(String::from("invalid token"));
    }

    *team.write() = Team::from_raw_airtable(input.into_inner());
    Json(String::from("success!"))
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build()
        .mount("/", routes![get_team, update_team])
        .manage(RwLock::new(Team::fetch()))
}
