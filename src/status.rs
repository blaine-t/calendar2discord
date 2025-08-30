use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub message: String,
    pub emoji: String,
}

pub async fn set_discord_status(status: Status) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "custom_status": {
            "text": status.message,
            "emoji_name": status.emoji
        }
    })
    .to_string();

    let res = client
        .patch("https://discord.com/api/v10/users/@me/settings")
        .header(
            "authorization",
            std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable not set"),
        )
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(reqwest::Error::from(res.error_for_status().unwrap_err()))
    }
}
