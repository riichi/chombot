use serenity::client::Context;
use serenity::model::id::ChannelId;

use crate::data::DISCORD_MESSAGE_SIZE_LIMIT;

pub async fn send_with_overflow(channel_id: ChannelId, ctx: &Context, text: String) {
    let mut message = String::new();
    for line in text.lines() {
        if message.len() + line.len() + "\n".len() > DISCORD_MESSAGE_SIZE_LIMIT {
            channel_id
                .send_message(ctx, |m| m.content(&message))
                .await
                .unwrap();
            message.clear();
        }

        message.push_str(line);
        message.push('\n');
    }
    if !message.is_empty() {
        channel_id
            .send_message(ctx, |m| m.content(message))
            .await
            .unwrap();
    }
}
