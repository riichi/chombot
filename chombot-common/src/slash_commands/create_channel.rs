use anyhow::{Result, anyhow};
use poise::CreateReply;
use poise::serenity_prelude::{ChannelId, GuildId};

use crate::channel_creator::{
    ChannelCreation, ChannelName, TextChannelRequest, create_text_channel, decide_placement,
};
use crate::{ChombotPoiseContext, ChombotPoiseUserData};

/// Create a text channel.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_CHANNELS"
)]
pub async fn create_channel<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    #[description = "Channel name"]
    #[max_length = 100]
    name: String,
    #[description = "Category to create the channel in (default: this channel's category)"]
    #[channel_types("Category")]
    category: Option<ChannelId>,
    #[description = "Channel topic"]
    #[max_length = 1024]
    topic: Option<String>,
) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| anyhow!("Guild ID is None"))?;
    let Some(name) = ChannelName::new(&name) else {
        ctx.send(
            CreateReply::default()
                .content("That name doesn't contain any characters usable in a channel name.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };
    let category = match placement(ctx, guild_id, category).await? {
        Ok(category) => category,
        Err(reason) => {
            ctx.send(CreateReply::default().content(reason).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    ctx.defer().await?;
    let creation = create_text_channel(
        ctx.serenity_context(),
        guild_id,
        TextChannelRequest {
            name,
            category,
            topic,
            audit_log_reason: &format!("Requested by {} via /create_channel", ctx.author().name),
        },
    )
    .await?;

    let reply = match creation {
        ChannelCreation::Created(channel) => format!("Created <#{}>.", channel.id),
        ChannelCreation::AlreadyExists(channel) => format!("<#{}> already exists.", channel.id),
    };
    ctx.say(reply).await?;
    Ok(())
}

async fn placement<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    guild_id: GuildId,
    requested: Option<ChannelId>,
) -> Result<Result<Option<ChannelId>, String>> {
    let author = ctx
        .author_member()
        .await
        .ok_or_else(|| anyhow!("Could not get the invoking member"))?
        .into_owned();
    let bot = guild_id
        .member(ctx.serenity_context(), ctx.framework().bot_id())
        .await?;
    let current_channel = match requested {
        Some(_) => None,
        None => ctx.guild_channel().await,
    };

    let placement = ctx
        .guild()
        .map(|guild| decide_placement(&guild, &author, &bot, requested, current_channel.as_ref()));
    placement.ok_or_else(|| anyhow!("Guild {guild_id} is not cached"))
}
