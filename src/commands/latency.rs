use crate::{Context, Error};
use poise::command;
use std::time::Instant;

#[command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    let _ = ctx.http().get_gateway();
    let elapsed = start.elapsed();
    ctx.reply(format!("Pong! Latency is {:?}", elapsed)).await?;
    Ok(())
}
