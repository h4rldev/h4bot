use crate::CurrentUserId;
use anyhow::anyhow;
use rustube::{Id, VideoFetcher};
use serenity::{
    framework::standard::{macros::*, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tracing::info;

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

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        return Err(anyhow!("guild_id was not found").into());
    };

    let guild = if let Some(guild) = guild_id.to_guild_cached(ctx) {
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
            let manager = songbird::get(ctx)
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

/// Makes h4bot leave the channel you're in.

/// ### Example Usage
/// ```rust
/// !leave
/// ```

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let data_read = ctx.data.read().await;
    let guild_id = if let Some(guild_id) = msg.guild_id {
        guild_id
    } else {
        return Err(anyhow!("guild_id was not found").into());
    };

    let guild = if let Some(guild) = guild_id.to_guild_cached(ctx) {
        guild
    } else {
        return Err(anyhow!("guild was not found").into());
    };

    let bot_id = match data_read.get::<CurrentUserId>() {
        Some(id) => *id,
        None => {
            eprintln!("Something went wrong getting the bot id");
            return Ok(());
        }
    };

    if let Some(bot_voice_state) = guild.voice_states.get(&bot_id) {
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
                let manager = songbird::get(ctx)
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
    //https://www.youtube.com/watch?v=dQw4w9WgXcQ
    let arg = args.single::<String>()?;
    let video_id = arg.split('=').collect::<Vec<&str>>()[1];
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

/// Makes h4bot stop the song in the channel you're in and leave.
/// Currently does nothing!

/// ### Example Usage
/// ```rust
/// !stop
/// ```

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "lul").await?;
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
