use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::commands::queue::{self, Queue};
use crate::CurrentUserId;
use anyhow::anyhow;
use regex::Regex;
use rustube::{Id, Video};
use serenity::{
    framework::standard::{macros::*, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use songbird::input::Restartable;
#[allow(unused_imports)]
use tracing::info;
use url::Url;

#[group("Music")]
#[only_in(guild)]
#[commands(join, leave, play, stop, skip, queue, now_playing)]
struct Music;

/// Makes h4bot join the channel you're in.

/// ### Example Usage
/// ```rust
/// // Make the bot join the channel
/// !join
/// ```
#[allow(dead_code)]
async fn find_file(dir: &Path, name: &str) -> Result<Option<PathBuf>, anyhow::Error> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    if stem == name {
                        return Ok(Some(path.to_path_buf()));
                    }
                }
            } else {
                return Err(anyhow!("File doesn't exist"));
            }
        }
        Err(anyhow!("Directory doesn't have entries"))
    } else {
        fs::create_dir(dir)?;
        Err(anyhow!("Directory doesn't exist, creating it"))
    }
}

async fn make_video_embed(ctx: &Context, msg: &Message, video: Video) {
    let video_info = video.video_details();
    let thumbnails = &video_info.thumbnails;
    let thumbnail = &thumbnails[3].url;

    msg.channel_id
        .send_message(&ctx.http, |message| {
            message
                .content("Playing:")
                .embed(|embed| {
                    embed
                        .author(|author| {
                            author.name(&video_info.author).url(format!(
                                "https://www.youtube.com/channel/{}",
                                &video_info.channel_id
                            ))
                        })
                        .title(&video_info.title)
                        .url(format!(
                            "https://www.youtube.com/watch?v={}",
                            video_info.video_id
                        ))
                        .image(thumbnail)
                        .description(&video_info.short_description)
                        .footer(|footer| footer.text("h4bot, made with ❤ by h4rl"))
                })
                .reference_message(msg)
                .allowed_mentions(|mentions| mentions.empty_parse())
        })
        .await
        .expect("Error sending message");
}

fn validate_url(mut args: Args) -> Option<String> {
    let mut url: String = args.single().ok()?;

    if url.starts_with('<') && url.ends_with('>') {
        url = url[1..url.len() - 1].to_string();
    }

    Url::parse(&url).ok()?;

    Some(url)
}

async fn handle_url(
    msg: &Message,
    ctx: &Context,
    arg: Args,
) -> Result<Restartable, Box<dyn std::error::Error>> {
    let re = Regex::new(r"v=([a-zA-Z0-9_-]+)").expect("Invalid Regex!");
    let url = validate_url(arg.clone()).expect("Invalid URL");
    match url.find("https://www.youtube.com") {
        Some(_) => {
            let video_id = if let Some(captures) = re.captures(&url) {
                match captures.get(1) {
                    Some(id) => id.as_str(),
                    None => {
                        msg.reply(&ctx.http, "Something went wrong getting the video id")
                            .await?;
                        return Err(anyhow!("Something went wrong getting the video id").into());
                    }
                }
            } else {
                msg.reply(&ctx.http, "Invalid URL").await?;
                return Err(anyhow!("Invalid URL").into());
            };
            match Id::from_str(video_id) {
                Ok(id) => {
                    let video = Video::from_id(id.into_owned()).await?;
                    let url = format!("https://www.youtube.com/watch?v={}", video_id);
                    let audio = Restartable::ytdl(url, true)
                        .await
                        .expect("Error creating input");
                    make_video_embed(ctx, msg, video).await;
                    Ok(audio)
                }
                Err(_) => {
                    let url = validate_url(arg.clone()).expect("Invalid URL");
                    let audio = Restartable::ffmpeg(url, true)
                        .await
                        .expect("Error creating input");
                    Ok(audio)
                }
            }
        }
        None => {
            msg.reply(&ctx.http, "Invalid URL").await?;
            Err(anyhow!("Invalid URL").into())
        }
    }
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = if let Some(guild) = msg.guild(&ctx.cache) {
        guild
    } else {
        return Err(anyhow!("guild was not found").into());
    };

    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

/// Makes h4bot leave the channel you're in.

/// ### Example Usage
/// ```rust
/// !leave
/// ```

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = if let Some(guild) = msg.guild(&ctx.cache) {
        guild
    } else {
        return Err(anyhow!("guild was not found").into());
    };

    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _handler = manager.leave(guild_id).await;

    Ok(())
}

/// Makes h4bot join the channel if not in a channel and play a song,
/// queue up the song
/// or play the song in the currently joined channel.
/// Work in progress, currently does nothing!!!!!!

/// ### Example Usage
/// ```rust
/// !p <youtube-url> | !play <youtube-url>
/// ```

#[command]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let manager = songbird::get(ctx).await.unwrap().clone();
    let data_read = ctx.data.read().await;
    let guild = if let Some(guild) = msg.guild(&ctx.cache) {
        guild
    } else {
        return Err(anyhow!("guild was not found").into());
    };
    let guild_id = guild.id;
    let bot_id = match data_read.get::<CurrentUserId>() {
        Some(id) => *id,
        None => {
            eprintln!("Something went wrong getting the bot id");
            return Ok(());
        }
    };

    let bot_voice_channel = guild
        .voice_states
        .get(&bot_id)
        .and_then(|voice_state| voice_state.channel_id);

    let user_voice_channel = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match user_voice_channel {
        Some(channel) => channel,
        None => {
            msg.reply(&ctx.http, "Not in a voice channel").await?;
            return Ok(());
        }
    };

    let is_user_in_channel = user_voice_channel.is_some();
    let is_bot_in_channel = bot_voice_channel.is_some();

    match (is_user_in_channel, is_bot_in_channel) {
        (true, false) => {
            let audio = handle_url(msg, ctx, args)
                .await
                .expect("Error creating input");
            let _handler = manager.join(guild_id, connect_to).await;
            if let Some(handler_lock) = manager.get(guild_id) {
                let mut handler = handler_lock.lock().await;
                handler.play_source(audio.into());
                info!("Playing Music!");
            };
        }
        (false, _) => {
            msg.reply(&ctx.http, "You're not in any voice channel!")
                .await?;
        }
        (true, true) if user_voice_channel != bot_voice_channel => {
            msg.reply(&ctx.http, "Not in the same voice channel!")
                .await?;
        }
        _ => {
            let url = validate_url(args.clone()).expect("Invalid URL");
            let len = queue::play(ctx, guild_id, url, Queue::Back).await?;
            let reply = if len == 1 {
                "Started playing the song".to_string()
            } else {
                format!("Added song to queue: position {}", len - 1)
            };
            msg.reply(&ctx.http, reply).await?;
            info!("Playing Music!");
        }
    }

    //https://www.youtube.com/watch?v=dQw4w9WgXcQ
    //msg.reply(&ctx.http, format!("Id: {}", id)).await?;
    //msg.reply(&ctx.http, format!("Thumbnails {:?}", thumbnails)).await?;
    Ok(())
}

/// Makes h4bot stop the song in the channel you're in and leave.
/// Currently does nothing!

/// ### Example Usage
/// ```rust
/// !stop
/// ```

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data_read = ctx.data.read().await;
    let guild = if let Some(guild) = msg.guild(&ctx.cache) {
        guild
    } else {
        return Err(anyhow!("guild was not found").into());
    };
    let guild_id = guild.id;

    let bot_id = match data_read.get::<CurrentUserId>() {
        Some(id) => *id,
        None => {
            eprintln!("Something went wrong getting the bot id");
            return Ok(());
        }
    };

    let bot_voice_channel = guild
        .voice_states
        .get(&bot_id)
        .and_then(|voice_state| voice_state.channel_id);

    let user_voice_channel = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let in_same_voice = { user_voice_channel == bot_voice_channel };
    if in_same_voice {
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;
            handler.stop();
            handler.leave().await?;
            msg.reply(&ctx.http, "Stopped playing!").await?;
        }
    }
    Ok(())
}

/// Makes h4bot skip the song.
/// Will probably be a voting command.
/// Currently does nothing!

/// ### Example Usage
/// ```rust
/// !skip
/// ```

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
    Ok(())
}

/// Shows the queue
/// Currently does nothing!

/// ### Example Usage
/// ```rust
/// !q | !queue
/// ```

#[command]
#[aliases(q)]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
    Ok(())
}

/// Shows the currently playing song
/// Currently does nothing!

/// ### Example Usage
/// ```rust
/// !np | !now_playing
/// ```

#[command]
#[aliases("np")]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
    Ok(())
}
