use futures::future::try_join_all;
use rand::{
    rngs::{OsRng, StdRng},
    seq::SliceRandom,
    SeedableRng,
};
use serenity::{
    framework::standard::{macros::*, Args, CommandResult},
    model::{
        channel::Message,
        prelude::{Member, Mention},
    },
    prelude::*,
};
use std::sync::Arc;
use tokio::task;

use tracing::{error, info};

#[group("Fun")]
#[commands(balls)]
struct Fun;

/// funny.

/// ### Example Usage
/// ```rust
/// // Execute the command with the "single" argument
/// !balls single
///
/// //Execute the command with the "multiple" argument
/// !balls multiple
///
/// // Execute the command with no arguments
/// !balls
/// ```

#[command]
async fn balls(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(guild_id) => guild_id,
        None => return Ok(()),
    };
    let guild = guild_id.to_partial_guild(&ctx.http).await?;
    let members = guild_id
        .members(&ctx.http, Some(1000), None)
        .await?
        .into_iter()
        .filter(|member| !member.user.bot && member.user.id != guild.owner_id)
        .collect::<Vec<Member>>();
    const NICKNAMES: [&str; 8] = [
        "testicles",
        "balls",
        "nuts",
        "tokhme",
        "bollocks",
        "cullions",
        "rocks",
        "gonads",
    ];
    let msg_clone = msg.clone();
    let ctx_clone = ctx.clone();
    let mut rng = StdRng::from_rng(OsRng).expect("Welp that's awkward");
    match args.single::<String>() {
        Ok(argument) => match argument.as_str() {
            "single" => {
                info!("ballsing 1 peple");
                let user = &members.choose(&mut rng).unwrap();
                let new_nickname = match NICKNAMES.choose(&mut rng) {
                    Some(nickname) => *nickname,
                    None => "balls",
                };
                if let Err(why) = guild_id
                    .edit_member(&ctx.http, user.user.id, |m| {
                        m.nickname(new_nickname.to_string())
                    })
                    .await
                {
                    msg.reply(&ctx.http, format!("Couldn't edit?: {:#}", why))
                        .await?;
                }
                msg.reply(
                    &ctx.http,
                    format!(
                        "uhh, this peple got ballsed: {}!1!!11!!!1",
                        user.user.mention()
                    ),
                )
                .await?;
                return Ok(());
            }
            "multiple" => {
                let amount = args.single::<usize>()?;
                if amount == 1 {
                    msg.reply(
                        &ctx.http,
                        "use `!balls single` instead of `!balls multiple 1` :)",
                    )
                    .await?;
                    return Ok(());
                } else if amount > members.len() {
                    msg.reply(
                        &ctx.http,
                        format!("use `!balls` instead of `!balls multiple {}` :)", amount),
                    )
                    .await?;
                    return Ok(());
                }
                info!("ballsing {} peple", amount);

                let mut rng = StdRng::from_rng(OsRng).expect("Welp that's awkward");
                let nicknames: Arc<Mutex<Vec<&'static str>>> =
                    Arc::new(Mutex::new(NICKNAMES.to_vec()));
                let users: Vec<Arc<Member>> = members
                    .choose_multiple(&mut rng, amount)
                    .map(|member| Arc::new(member.clone()))
                    .collect();
                let changed_nicknames: Arc<Mutex<Vec<Mention>>> = Arc::new(Mutex::new(Vec::new()));
                let futures = users.into_iter().map(|user| {
                    let user = Arc::clone(&user);
                    let msg = msg_clone.clone();
                    let ctx = ctx_clone.clone();
                    let nicknames = Arc::clone(&nicknames);
                    let changed_nicknames = Arc::clone(&changed_nicknames);
                    task::spawn(async move {
                        // Perform your operation here
                        let mut rng = StdRng::from_rng(OsRng).expect("Welp that's awkward");
                        let mut nicknames = nicknames.lock().await;
                        let new_nickname = match nicknames.choose_mut(&mut rng) {
                            Some(nickname) => *nickname,
                            None => "balls",
                        };
                        if let Err(why) = guild_id
                            .edit_member(&ctx.http, user.user.id, |m| {
                                m.nickname(new_nickname.to_string())
                            })
                            .await
                        {
                            msg.reply(&ctx.http, format!("Couldn't edit?: {:#}", why))
                                .await
                                .expect("Welp, you goofed up");
                        } else {
                            let mut changed_nicknames = changed_nicknames.lock().await;
                            changed_nicknames.push(user.user.mention());
                        }
                    })
                });
                let results = try_join_all(futures).await;
                match results {
                    Ok(_) => info!("Successfully changed the name of multiple people!"),
                    Err(why) => error!("Task failed! {}", why),
                };

                msg.reply(
                    &ctx.http,
                    format!(
                        "uhh, these people got ballsed: {}!1!!11!!!1",
                        changed_nicknames
                            .lock()
                            .await
                            .iter()
                            .map(|mention| mention.to_string())
                            .collect::<String>()
                    ),
                )
                .await
                .expect("Couldn't reply to user with ballsed people");
                return Ok(());
            }
            _ => {
                msg.reply(&ctx.http, "uhh, u're ballsing wrong!!1!!11!!!1")
                    .await?;
                return Ok(());
            }
        },
        Err(_) => {
            info!("ballsing everyone");
            let mut rng = StdRng::from_rng(OsRng).expect("Hello");
            let bot_nickname = match NICKNAMES.choose(&mut rng) {
                Some(nickname) => *nickname,
                None => "balls",
            };
            match guild.edit_nickname(&ctx.http, Some(bot_nickname)).await {
                Ok(_) => info!("Changed nickname to {}", bot_nickname),
                Err(err) => error!("Failed to change nickname: {:?}", err),
            }
            let nicknames: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(NICKNAMES.to_vec()));

            let users: Vec<Arc<Member>> = members
                .iter()
                .map(|member| Arc::new(member.clone()))
                .collect();
            let changed_nicknames: Arc<Mutex<Vec<Mention>>> = Arc::new(Mutex::new(Vec::new()));
            let futures = users.into_iter().map(|user| {
                let user = Arc::clone(&user);
                let msg = msg_clone.clone();
                let ctx = ctx_clone.clone();
                let nicknames = Arc::clone(&nicknames);
                let changed_nicknames = Arc::clone(&changed_nicknames);
                task::spawn(async move {
                    // Perform your operation here
                    let mut rng = StdRng::from_rng(OsRng).expect("Hello");
                    let mut nicknames = nicknames.lock().await;
                    let new_nickname = match nicknames.choose_mut(&mut rng) {
                        Some(nickname) => *nickname,
                        None => "balls",
                    };

                    if let Err(why) = guild_id
                        .edit_member(&ctx.http, user.user.id, |m| {
                            m.nickname(new_nickname.to_string())
                        })
                        .await
                    {
                        msg.reply(&ctx.http, format!("Couldn't edit?: {:#}", why))
                            .await
                            .expect("Welp, you goofed up");
                    } else {
                        let mut changed_nicknames = changed_nicknames.lock().await;
                        changed_nicknames.push(user.user.mention());
                    }
                })
            });
            let results = try_join_all(futures).await;
            match results {
                Ok(_) => info!("Successfully changed the name of multiple people!"),
                Err(why) => error!("Task failed! {}", why),
            };
            let bot_mention = ctx.cache.current_user().mention();
            msg.reply(
                &ctx.http,
                format!(
                    "uhh, these people got ballsed: {}{}!1!!11!!!1",
                    bot_mention,
                    changed_nicknames
                        .lock()
                        .await
                        .iter()
                        .map(|mention| mention.to_string())
                        .collect::<String>()
                ),
            )
            .await
            .expect("Couldn't reply to user with ballsed people");
            return Ok(());
        }
    }
}
