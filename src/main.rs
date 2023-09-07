use anyhow::anyhow;
use poise::CreateReply;
use poise::{
    serenity_prelude::GatewayIntents, Framework, FrameworkOptions, PrefixFrameworkOptions,
};
use serenity::model::id::GuildId;
use shuttle_poise::ShuttlePoise;
use std::collections::HashMap;
use std::sync::Mutex;
/*use serenity::{
    async_trait,
    framework::standard::{
        help_commands, macros::*, Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, prelude::UserId},
    prelude::*,
};*/
use shuttle_secrets::SecretStore;
struct Bot;

struct Data {
    prefix: String,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/*struct CurrentUserId;

struct Wood;

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

impl TypeMapKey for Wood {
    type Value = bool;
}

impl TypeMapKey for CurrentUserId {
    type Value = serenity::model::id::UserId;
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data
        .get_mut::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

async fn get_owner(token: String) -> HashSet<UserId> {
    let http = Http::new(&token);
    match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            owners
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    }
}

#[help]
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
//#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}*/

async fn get_prefix(ctx: Context<'_, Data, Error>) -> Result<Option<String>, Error> {
    let prefix = ctx.data().prefix.lock.expect("Can't lock prefix");
    Ok(Some(prefix))
}

#[poise::command(slash_command, prefix_command)]
async fn prefix(
    ctx: Context<'_>,
    #[description = "prefix to change into"] prefix: String,
) -> Result<(), Error> {
    ctx.data().prefix = prefix;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn wood(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttlePoise<Data, Error> {
    //#[tokio::main]
    //async fn main() {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };
    //let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    //let owners = get_owner(token.clone()).await;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::all();

    let bot_data = BotData {
        prefixes: Mutex::new(HashMap::new()),
    };

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![wood()],
            prefix_options: PrefixFrameworkOptions {
                prefix: None,
                dynamic_prefix: {
                    match get_prefix {
                        Some(prefix) => prefix,
                        None => "!",
                    }
                },
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                )),
                case_insensitive_commands: true,
                mention_as_prefix: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data { prefix })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
    /*if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }*/
}
