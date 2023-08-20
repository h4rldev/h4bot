use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{macros::*, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::{sync::Arc, time::Instant};
use tracing::info;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group("Latency")]
#[commands(ping, shard_ping)]
struct Latency;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Recieved !ping command");
    let start_time = Instant::now();
    let response = msg.reply(&ctx.http, "Pong!").await;
    let end_time = Instant::now();
    let latency = end_time.duration_since(start_time).as_millis();
    if let Ok(mut response) = response {
        response
            .edit(&ctx.http, |m| {
                m.content(format!("Pong! {}ms", latency))
                    .allowed_mentions(|f| f.empty_parse());
                m
            })
            .await
            .unwrap();
    }
    Ok(())
}

#[command]
async fn shard_ping(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Recieved !shard_ping command");
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    msg.reply(ctx, &format!("Pong! {:?}", runner.latency.unwrap()))
        .await?;

    Ok(())
}
