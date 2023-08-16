use anyhow::anyhow;
use std::{
    time::Instant,
    sync::Arc,
    collections::{
        HashMap,
        HashSet
    }
};
use serenity::{
    http::Http,
    prelude::*,
    async_trait,
    framework::standard::{
        macros::*,
        help_commands,
        Args,
        CommandGroup,
        CommandResult,
        DispatchError,
        HelpOptions,
        StandardFramework
    },
    model::{
        prelude::UserId,
        gateway::Ready,
        channel::Message,
    },
    client::bridge::gateway::{
        ShardId,
        ShardManager
    }
};
use shuttle_secrets::SecretStore;
use tracing::info;

struct Bot;

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
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
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
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

    let http = Http::new(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::all();

    let framework = StandardFramework::new()
        .configure(|config| config
            .with_whitespace(true)
            .allow_dm(false)
            .on_mention(Some(bot_id))
            .prefix("!")
            .owners(owners))
            .before(before)
            .after(after)
            .unrecognised_command(unknown_command)
            .help(&MY_HELP)
            .group(&CMDS_GROUP);


    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .framework(framework)
        //.application_id(application_id.parse::<u64>().unwrap())
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    Ok(client.into())
}


#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    info!("RECIEVED !ping COMMAND");
    let start_time = Instant::now();
    let response =  msg.channel_id.say(&ctx.http, "Pong!").await;
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
    info!("RECIEVED !shard_ping COMMAND");
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager").await?;

            return Ok(());
        },
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        },
    };

    msg.reply(ctx, &format!("Pong! {:?}", runner.latency.unwrap())).await?;

    Ok(())
}

#[group("Cmds")]
#[commands(ping,shard_ping)]
struct Cmds;