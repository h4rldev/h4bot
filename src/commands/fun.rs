use crate::{Context, Error};
use poise::{command, serenity_prelude::Mention};

#[command(slash_command)]
pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Hi!").await?;
    Ok(())
}

#[command(slash_command)]
pub async fn balls(ctx: Context<'_>, amount: Option<String>) -> Result<(), Error> {
    let users = ctx.cache().users();
    let users: Vec<Mention> = users.iter().map(|uid| Mention::from(*uid.key())).collect();
    let reply = format!("users: {:?} \nparam: {:?}", users, amount);
    ctx.reply(reply).await?;
    Ok(())
}
