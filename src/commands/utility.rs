#![allow(dead_code)]
use poise::{command, serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter}, CreateReply};
use tracing::info;
use crate::{Error, Context};
use reqwest::Client;
use serde::Deserialize;
use std::time::Instant;

// Discord Status

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

#[derive(Deserialize)]
struct WeekData {
    week: i32,
}

#[command(slash_command, category="Latency")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    ctx.http().get_gateway().await?;
    let elapsed = start.elapsed();
    ctx.reply(format!("Pong! Time taken to get gateway: {:?}ms", elapsed.as_millis())).await?;
    Ok(())
}

#[command(slash_command, category="Latency")]
pub async fn status(ctx: Context<'_>,) -> Result<(), Error> {
    let client = Client::new();
    let request = client.get("https://discordstatus.com/api/v2/status.json").header("Accept", "application/json").send().await?;
    let body = request.json::<DiscordStatus>().await?;

    let reply_embed = CreateEmbed::new()
        .title(format!("Response from {}", body.page.name))
        .url(body.page.url)
        .description(format!("Discord responds with {}", body.status.description))
        .author(CreateEmbedAuthor::new("fardbot by h4rl").url("https://h4rl.dev"))
        .footer(CreateEmbedFooter::new("Licensed under the BSD-3 Clause License"));

    let reply = CreateReply::default().embed(reply_embed).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

#[command(slash_command, category="Utility")]
pub async fn get_week(ctx: Context<'_>) -> Result<(), Error> {
    let client = Client::new();
    let request = client.get("https://vecka.nu/").header("Accept", "application/json").send().await?;

    let body = request.json::<WeekData>().await?;

    let reply_embed = CreateEmbed::new()
        .title(format!("The current week is {}", body.week))
        .url("https://vecka.nu")
        .author(CreateEmbedAuthor::new("fardbot by h4rl").url("https://h4rl.dev"))
        .footer(CreateEmbedFooter::new("Licensed under the BSD-3 Clause License"));


    let reply = CreateReply::default().embed(reply_embed).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())

}
