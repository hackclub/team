#[macro_use]
extern crate rocket;
use parking_lot::RwLock;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
        //TODO: Get from Airtable

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
    rocket::build()
        .mount("/", routes![get_team, update_team])
        .manage(RwLock::new(Team::fetch()))
}
