#![allow(dead_code)]
use poise::command;
use crate::{Error, Context};
use reqwest::Client;
use serde::Deserialize;
use std::time::Instant;

#[derive(Deserialize)]
struct DiscordStatus {
    page: Page,
    status: Status
}

#[derive(Deserialize)]
struct Page {
    id: String,
    name: String,
    url: String,
    time_zone: String,
    updated_at: String,
}

#[derive(Deserialize)]
struct Status {
    indicator: Option<String>,
    description: String
}

#[command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    ctx.http().get_gateway().await?;
    let elapsed = start.elapsed();
    ctx.reply(format!("Pong! Time taken to get gateway: {:?}ms", elapsed.as_millis())).await?;
    Ok(())
}

#[command(slash_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let client = Client::new();
    let request = client.get("https://discordstatus.com/api/v2/status.json").send().await?;
    let body = request.json::<DiscordStatus>().await?;
    let reply = format!("[Discord Status](https://discordstatus.com) returns {}", body.status.description);
    ctx.reply(reply).await?;
    Ok(())
}
