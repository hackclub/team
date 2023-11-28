#[macro_use]
extern crate rocket;
use parking_lot::RwLock;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Team {
    current: Vec<TeamMember>,
    alumni: Vec<TeamMember>,
}
impl Team {
    fn fetch() -> Self {
        let app_id = var("APP_ID").expect("an airtable app ID");
        let token = var("TOKEN").expect("an airtable token");

        let res = reqwest::blocking::Client::new()
            .get(format!("https://api.airtable.com/v0/{app_id}/Current"))
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .unwrap()
            .text()
            .unwrap();

        //TODO: Traverse and parse
        println!("{}", res);

        Self {
            current: vec![TeamMember {
                name: "Aidan".to_owned(),
                bio: None,
                department: "Engineering".to_owned(),
                role: "Software Engineer".to_owned(),
                bio_hackfoundation: None,
                pronouns: "he/him".to_owned(),
                slack_id: None,
            }],
            alumni: vec![],
        }
    }
}

#[get("/")]
fn get_team(team: &State<RwLock<Team>>) -> Json<Team> {
    Json(team.read().clone())
}

#[post("/", format = "json", data = "<input>")]
fn update_team(team: &State<RwLock<Team>>, input: Json<Team>) -> Json<String> {
    *team.write() = input.into_inner();
    Json(String::from("success!"))
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build()
        .mount("/", routes![get_team, update_team])
        .manage(RwLock::new(Team::fetch()))
}
