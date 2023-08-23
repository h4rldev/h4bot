use std::{
    fs,
    path::{Path, PathBuf},
};

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

async fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    if stem == name {
                        return Some(path.to_path_buf());
                    }
                }
            }
        }
    }
    None
}

async fn handle_url(arg: &String) -> Result<Restartable, Box<dyn std::error::Error>> {
    let re = Regex::new(r"v=([a-zA-Z0-9_-]+)").expect("Invalid Regex!");
    match arg.find("https://www.youtube.com") {
        Some(_) => {
            let video_id = if let Some(captures) = re.captures(&arg) {
                match captures.get(1) {
                    Some(id) => id.as_str(),
                    None => {
                        return Err(anyhow!("Something went wrong getting the video id").into());
                    }
                }
            } else {
                return Err(anyhow!("Invalid URL").into());
            };
            match Id::from_str(video_id) {
                Ok(id) => {
                    let video = Video::from_id(id.into_owned()).await?;
                    let dir = Path::new("./music");
                    let file = find_file(dir, video_id).await.unwrap();
                    let downloaded_video = if fs::metadata(file.clone()).is_err() {
                        video
                            .best_audio()
                            .unwrap()
                            .download_to_dir("./music")
                            .await?
                    } else {
                        file
                    };
                    let audio = Restartable::ffmpeg(downloaded_video, true)
                        .await
                        .expect("Error creating input");
                    return Ok(audio);
                }
                Err(_) => {
                    let url = arg.to_owned();
                    let audio = Restartable::ffmpeg(url, true)
                        .await
                        .expect("Error creating input");
                    return Ok(audio);
                }
            }
        }
        None => {
            return Err(anyhow!("Invalid URL").into());
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
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    let is_user_in_channel = user_voice_channel.is_some();

    if !is_user_in_channel {
        msg.reply(&ctx.http, "Not in a voice channel").await?;
        return Ok(());
    } else if !in_same_voice {
        msg.reply(&ctx.http, "Not in the same voice channel")
            .await?;
        return Ok(());
    }

    //https://www.youtube.com/watch?v=dQw4w9WgXcQ
    let arg = args.single::<String>()?;
    let re = Regex::new(r"v=([a-zA-Z0-9_-]+)").expect("Invalid Regex!");
    let video_id = if let Some(captures) = re.captures(&arg) {
        match captures.get(1) {
            Some(id) => id.as_str(),
            None => {
                msg.reply(&ctx.http, "Couldn't get video id").await?;
                return Ok(());
            }
        }
    } else {
        msg.reply(&ctx.http, "Invalid URL").await?;
        return Ok(());
    };
    match Id::from_str(video_id) {
        Ok(id) => {
            msg.reply(&ctx.http, format!("Id: {}", id)).await?;
            let video = Video::from_id(id.into_owned()).await?;
            let video_info = video.video_details();
            let thumbnails = &video_info.thumbnails;
            let thumbnail = &thumbnails[3].url;
            //msg.reply(&ctx.http, format!("Thumbnails {:?}", thumbnails)).await?;
            msg.reply(&ctx.http, "getting audio").await?;
            let audio = handle_url(&arg).await.expect("Error creating input");
            let connect_to = match user_voice_channel {
                Some(channel) => channel,
                None => {
                    msg.reply(&ctx.http, "Not in a voice channel").await?;
                    return Ok(());
                }
            };
            let manager = songbird::get(&ctx).await.unwrap().clone();
            let _handler = manager.join(guild_id, connect_to).await;
            if let Some(handler_lock) = manager.get(guild_id) {
                let mut handler = handler_lock.lock().await;
                handler.play_source(audio.into());
                println!("Playing audio");
            };
            msg.channel_id
                .send_message(&ctx.http, |message| {
                    message
                        .embed(|embed| {
                            embed
                                .author(|author| {
                                    author.name(&video_info.author).url(format!(
                                        "https://www.youtube.com/channel/{}",
                                        &video_info.channel_id
                                    ))
                                })
                                .title(&video_info.title)
                                .url(format!("https://www.youtube.com/watch?v={}", video_id))
                                .image(thumbnail)
                                .description(&video_info.short_description)
                                .footer(|footer| footer.text("h4bot, made with ❤ by h4rl"))
                        })
                        .reference_message(msg)
                        .allowed_mentions(|mentions| mentions.empty_parse())
                })
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
