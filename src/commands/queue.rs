use serenity::{client::Context, model::id::GuildId};
use songbird::input::Restartable;

#[derive(Debug)]
pub enum Queue {
    Front,
    Back,
}

pub async fn play(
    ctx: &Context,
    guild_id: GuildId,
    url: String,
    queue_direction: Queue,
) -> anyhow::Result<usize, anyhow::Error> {
    let source = Restartable::ytdl(url, true).await?;
    let songbird = songbird::get(ctx).await.expect("Couldn't get songbird");

    let call_m = songbird.get(guild_id).expect("guild_id was None");

    let mut call = call_m.lock().await;

    match queue_direction {
        Queue::Front => {
            call.enqueue_source(source.into());
            call.queue().modify_queue(|q| {
                if let Some(track) = q.pop_back() {
                    q.push_front(track);
                    if q.len() > 1 {
                        q.swap(0, 1);
                    }
                }
            });
        }
        Queue::Back => {
            call.enqueue_source(source.into());
        }
    };
    Ok(call.queue().len())
}
