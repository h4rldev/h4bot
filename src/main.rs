use anyhow::anyhow;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use rustube::{Id, VideoFetcher};
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands, macros::*, Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, prelude::UserId},
    prelude::*,
};
use songbird::SerenityInit;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

const BOT_ID: UserId = UserId(871488289125838898);

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
    let owners = get_owner(token.clone()).await;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::all();

    let framework = StandardFramework::new()
        .configure(|config| {
            config
                .with_whitespace(true)
                .allow_dm(false)
                .on_mention(Some(BOT_ID))
                .prefix("!")
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .help(&MY_HELP)
        .group(&LATENCY_GROUP)
        .group(&MUSIC_GROUP)
        .group(&FUN_GROUP);

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .framework(framework)
        .register_songbird()
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

#[group("Music")]
#[commands(join, leave, play, stop, skip, queue, now_playing)]
struct Music;

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        return Err(anyhow!("guild_id was not found").into());
    };

    let guild = if let Some(guild) = guild_id.to_guild_cached(&ctx) {
        guild
    } else {
        return Err(anyhow!("guild was not found").into());
    };

    if let Some(voice_state) = guild.voice_states.get(&msg.author.id) {
        if let Some(channel_id) = voice_state.channel_id {
            info!("User is in voice channel with id {}", channel_id.0);
            msg.reply(
                &ctx.http,
                format!("Joined channel {}", channel_id.mention()),
            )
            .await
            .expect("Couldn't reply to user!");
            let manager = songbird::get(&ctx)
                .await
                .expect("Songbird Voice client was not initialized.")
                .clone();
            let _handler = manager.join(guild_id, channel_id).await;
        }
    } else {
        info!("User is not in a voice channel");
        msg.reply(&ctx.http, "You're not in a voice channel!")
            .await?;
    }
    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        return Err(anyhow!("guild_id was not found").into());
    };

    let guild = if let Some(guild) = guild_id.to_guild_cached(&ctx) {
        guild
    } else {
        return Err(anyhow!("guild was not found").into());
    };

    if let Some(bot_voice_state) = guild.voice_states.get(&BOT_ID) {
        if let Some(author_voice_state) = guild.voice_states.get(&msg.author.id) {
            if let Some(bot_channel_id) = bot_voice_state.channel_id {
                info!("h4bot is in voice channel with id {}", bot_channel_id.0);
            }
            if let Some(author_channel_id) = author_voice_state.channel_id {
                info!("User is in voice channel with id {}", author_channel_id.0);
                msg.reply(
                    &ctx.http,
                    format!("Left channel {}", author_channel_id.mention()),
                )
                .await?;
                let manager = songbird::get(&ctx)
                    .await
                    .expect("Songbird Voice client was not initialized.")
                    .clone();
                let _handler = manager.leave(guild_id).await;
            }
        } else {
            info!("User is not in a voice channel");
            msg.reply(&ctx.http, "You're not in a voice channel!")
                .await?;
        }
    } else {
        info!("Not in a voice channel!");
        msg.reply(&ctx.http, "I'm not in a voice channel!").await?;
    }
    Ok(())
}

#[command]
#[aliases("p")]
#[description = "Plays a song from a youtube url"]
#[usage = "!play <youtube_url>"]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    //https://www.youtube.com/watch?v=dQw4w9WgXcQ
    let arg = args.single::<String>()?;
    let video_id = arg.split("=").collect::<Vec<&str>>()[1];
    match Id::from_str(video_id) {
        Ok(video_id) => {
            let fetcher = VideoFetcher::from_id(video_id.into_owned())?;
            let video = fetcher.fetch().await?.descramble()?;
            let video_info = video.video_details();

            msg.reply(&ctx.http, format!("Video info: {:?}", video_info))
                .await?;
        }
        Err(why) => {
            msg.reply(
                &ctx.http,
                format!("Something occured or I couldn't find video\nError: {}", why),
            )
            .await?;
        }
    }
    msg.reply(&ctx.http, "").await?;
    Ok(())
}

#[command]
#[description = "Stops the media player"]
#[usage = "!stop"]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
    Ok(())
}

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
    Ok(())
}

#[command]
#[aliases("np")]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
    Ok(())
}

#[group("Fun")]
#[commands(balls)]
struct Fun;

#[command]
async fn balls(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        return Err(anyhow!("guild_id was not found").into());
    };
    let nicknames = vec!["testicles", "balls", "nuts"];
    let members = guild_id.members(&ctx.http, Some(1000), None).await?;

    // Iterate over each member and change the member's nickname
    for member in members {
        let mut rng = StdRng::from_entropy();
        let new_nickname = match nicknames.choose(&mut rng) {
            Some(nicknames) => nicknames,
            None => "balls",
        };
        guild_id
            .edit_member(&ctx.http, member.user.id, |m| {
                m.nickname(new_nickname.clone())
            })
            .await?;
        msg.reply(&ctx.http, new_nickname.clone()).await?;
    }
    Ok(())
}
