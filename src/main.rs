use anyhow::anyhow;
use serenity::{
    prelude::*,
    async_trait, model::{
        gateway::Ready,
        channel::Message,
        application::{
            command::Command, interaction::{
                Interaction, InteractionResponseType
            }
        }
    }
};
use shuttle_secrets::SecretStore;
use tracing::{error, info};
use std::time::Instant;

struct Bot;

async fn measure_latency(ctx: &Context, msg: &Message) {
    let start_time = Instant::now();
    let response = msg.reply(&ctx.http, "Pong!").await;
    let end_time = Instant::now();


    if let Ok(mut response) = response {
        let latency = end_time.duration_since(start_time).as_millis();
        response
            .edit(&ctx.http, |m| {
                m.content(format!("Pong! {}ms", latency))
                    .allowed_mentions(|f| f.empty_parse());
                m
            })
            .await
            .unwrap();
    }
}



#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            "!hello" => if let Err(e) = msg.reply(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            },
            "!ping" => measure_latency(&ctx, &msg).await,
            &_ => {}
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Recieved Command: {:#?}", command);
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
