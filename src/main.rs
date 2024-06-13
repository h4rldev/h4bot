use poise::serenity_prelude as serenity;
use tracing::info;

mod commands;
use commands::{
    fun::{balls, hello},
    utility::{ping, status, get_week},
};

#[poise::command(slash_command, track_edits)]
async fn help(
    ctx: Context<'_>, 
    #[description = "Command you need help about"] 
    command: Option<String>
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(()) 
}

async fn pre_command(ctx: Context<'_>) {
    info!("Running command: {} from {}", ctx.command().qualified_name, ctx.author().name)
}

async fn post_command(ctx: Context<'_>) {
    info!("Ran command: {} from {}", ctx.command().qualified_name, ctx.author().name)
}

pub struct Data {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");
    tracing_subscriber::fmt::init();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            pre_command: |ctx| Box::pin(pre_command(ctx)),
            post_command: |ctx| Box::pin(post_command(ctx)),
            commands: vec![help(), hello(), balls(), ping(), status(), get_week()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.expect("Can't construct client").start().await.expect("Can't start.");
}
