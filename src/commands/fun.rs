use crate::{Context, Error, serenity::{User, Mentionable}};
use rand::{
    Rng,
    thread_rng,
    seq::SliceRandom
};
use poise::{command, serenity_prelude::{EditMember, Member}, ChoiceParameter};

#[derive(ChoiceParameter, Debug)]
enum Balls {
    #[name = "Balls a single person"]
    Single,

    #[name = "Balls multiple people"]
    Multiple,

    #[name = "Balls everyone"]
    All
}

// Replies with hi!
#[command(slash_command, category="Fun")]
pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Hi!").await?;
    Ok(())
}

// Balls people!!!
#[command(slash_command, category="Fun")]
pub async fn balls(ctx: Context<'_>,  balls_choice: Option<Balls>, specific: Option<Member>) -> Result<(), Error> {
    let names_for_balls = ["tokhme".to_string(), "balls".to_string(), "rocks".to_string(), "nuts".to_string(), "testicles".to_string(), "family jewels".to_string(), "bollocks".to_string(), "ballocks".to_string(), "cullions".to_string(), "jewels".to_string(), "orbs".to_string()];
    let guild_id = ctx.guild_id().expect("Can't get guild_id");
    let users = guild_id.members(&ctx.http(), None, None).await?;
    let mut users: Vec<User> = users.iter().map(|user| user.user.clone()).collect();
    users.retain(|user| user != ctx.author() && user.id != 871488289125838898);
    let length = users.len();

    let who_to_balls = match balls_choice {
        Some(balls_choices) => {
            match balls_choices {
                Balls::Single => {
                    pick_random(1, users).await?
                },
                Balls::Multiple => {
                    let amount = thread_rng().gen_range(3..length);
                    pick_random(amount.try_into()?, users).await?
                },
                Balls::All => {
                    users
                }
            }
        },
        None => {
            if specific.is_some() {
                vec![specific.unwrap().user]
            } else {
                ctx.reply("Invalid user.., picking random.").await?;
                pick_random(1, users).await?
            }
        }
    };
    
    for user in who_to_balls.clone() {
        let mut member = guild_id.member(&ctx.serenity_context(), user.id).await?;
        let random_nickname = {
            let mut rng = thread_rng();
            names_for_balls.choose(&mut rng).expect("Failed to pick random name")
        };
        let edit_builder = EditMember::new().nickname(random_nickname);
        member.edit(&ctx.http(), edit_builder).await?;
    }

    let mention_users: String = who_to_balls.iter().map(|user| user.mention().to_string()).collect::<Vec<String>>().join(", ");
    let reply = format!("Successfully ballsed: {}", mention_users);
    ctx.reply(reply).await?;
    Ok(())
}

pub async fn pick_random(amount: u32, users: Vec<User>) -> Result<Vec<User>, Error> {
    let users = if amount > 1 {
        let users: Vec<User> = {
            let mut rng = thread_rng();
            let pick: Vec<&User> = users.choose_multiple(&mut rng, amount.try_into()?).collect();
            let derefd: Vec<User> = pick.into_iter().cloned().collect();
            derefd
        };
        users
    } else {
        let user = {
            let mut rng = thread_rng();
            users.choose(&mut rng)
        };
        let users: Vec<User> = vec![user.unwrap().clone()];
        users
    };
    Ok(users)
}
