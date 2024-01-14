use poise::serenity_prelude::{ChannelId, Context, CreateMessage, Error as SerenityError};

use crate::data::DISCORD_MESSAGE_SIZE_LIMIT;

pub async fn send_with_overflow(
    channel_id: ChannelId,
    ctx: &Context,
    text: &str,
) -> Result<(), SerenityError> {
    let mut message = String::new();
    for line in text.lines() {
        if message.len() + line.len() + "\n".len() > DISCORD_MESSAGE_SIZE_LIMIT {
            channel_id
                .send_message(ctx, CreateMessage::new().content(&message))
                .await?;
            message.clear();
        }

        message.push_str(line);
        message.push('\n');
    }
    if !message.is_empty() {
        channel_id
            .send_message(ctx, CreateMessage::new().content(&message))
            .await?;
    }

    Ok(())
}
