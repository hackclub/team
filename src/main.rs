#[macro_use]
extern crate rocket;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TeamMember {
    name: String,
    department: String,
    role: String,
    bio: Option<String>,
    bio_hf: Option<String>,
    pronouns: String,
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
            current: vec![],
            alumni: vec![],
        }
    }
}

#[get("/")]
fn get_team(team: &State<Team>) -> Json<&Team> {
    Json(team.inner())
}

#[post("/update", format = "json", data = "<user_input>")]
fn update_team(user_input: Json<Value>) -> String {
    println!("{:?}", user_input);
    format!("recv!")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_team, update_team])
        .manage(Team::fetch())
}
