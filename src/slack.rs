pub fn send_slack_message(slack_id: &str, message: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();

    let res = client
        .post("https://slack.com/api/chat.postMessage")
        .header(
            "Authorization",
            format!("Bearer {}", std::env::var("SLACK_TOKEN").unwrap()),
        )
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "channel": slack_id,
            "text": message,
        }))
        .send();

    if let Err(err) = res {
        return Err(err);
    }

    let res = res.unwrap();

    let text = res.text();
    println!("{:?}", text);

    Ok(())
}
