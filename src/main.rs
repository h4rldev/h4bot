use anyhow::anyhow;
use serenity::{
    prelude::*,
    async_trait, model::{
        gateway::Ready,
        channel::Message,
        application::interaction::{
            Interaction, InteractionResponseType
        }
    }
};
use std::error::Error;
use shuttle_secrets::SecretStore;
use tracing::{error, info};
use std::time::Instant;

struct Bot;

enum Replyable {
    Message(Message),
    Interaction(Interaction),
}

impl Replyable {
    async fn reply(&self, ctx: &Context, content: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self {
            Replyable::Message(msg) => {
                msg.reply(&ctx.http, content).await?;
            }
            Replyable::Interaction(interaction) => {
                if let Interaction::ApplicationCommand(command) = interaction {
                    let response = command.create_interaction_response(&ctx.http, |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| d.content(content))
                    }).await;
                    if let Err(e) = response {
                        error!("Error sending message: {:?}", e);
                    }
                }
            }
        }
        Ok(())
    }
}

async fn measure_latency(ctx: &Context, replyable: Replyable) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start_time = Instant::now();
    replyable.reply(ctx, "Pong!").await?;
    let end_time = Instant::now();
    let latency = end_time.duration_since(start_time).as_millis();
    match replyable {
        Replyable::Interaction(interaction) => {
            if let Interaction::ApplicationCommand(command) = interaction {
                let response = command.edit_original_interaction_response(&ctx.http, |r| {
                    r.content(format!("Pong!, Latency {}ms", latency).as_str())
                }).await;
                if let Err(e) = response {
                    error!("Error editing message: {:?}", e);
                }
            }
        }
        Replyable::Message(mut msg) => {
            let response = msg.edit(&ctx.http, |m| {
                m.content(format!("Pong!, Latency {}ms", latency).as_str())
            }).await;
            if let Err(e) = response {
                error!("Error editing message: {:?}", e);
            }
        }
    }
    Ok(())
}


#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            "!hello" => if let Err(e) = msg.reply(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            },
            "!ping" => if let Err(e) = measure_latency(&ctx, Replyable::Message(msg)).await {
                error!("Error sending message: {:?}", e);
            }
            &_ => {}
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = &interaction {
            println!("Recieved Command: {:#?}", command);
            match command.data.name.as_str() {
                "ping" => {
                    if let Err(e) = measure_latency(&ctx, Replyable::Interaction(interaction)).await {
                        error!("Error measuring latency: {:?}", e);
                    }
                }
                &_ => {}
            }
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

    /*let application_id = if let Some(application_id) = secret_store.get("APPLICATION_ID") {
        application_id
    } else {
        return Err(anyhow!("'APPLICATION_ID' was not found").into());
    };*/

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        //.application_id(application_id.parse::<u64>().unwrap())
        .await
        .expect("Err creating client");

    Ok(client.into())
}
