use std::time::Duration;

use anyhow::{Result, anyhow};
use log::info;
use poise::serenity_prelude::{Context, CreateAllowedMentions, CreateMessage, Reaction, UserId};

use crate::ChombotPoiseUserData;
use crate::channel_creator::{
    ChannelCreation, ChannelName, TextChannelRequest, create_text_channel, decide_placement,
};
use crate::tournament_channel::{tournament_channel_name, tournament_topic};

const TOURNAMENT_LOOKUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Creates a tournament's channel when its number is clicked under a
/// TOURNAMENTS UPDATE, like `/create_channel` run in the update's channel.
pub async fn handle_reaction_add<U: ChombotPoiseUserData>(
    ctx: &Context,
    reaction: &Reaction,
    data: &U,
    bot_id: UserId,
) -> Result<()> {
    let (Some(guild_id), Some(user_id)) = (reaction.guild_id, reaction.user_id) else {
        return Ok(());
    };
    if user_id == bot_id {
        return Ok(());
    }
    let Some(posted) = data
        .chombot()
        .tournament_updates()
        .tournament_for_reaction(reaction.message_id, &reaction.emoji)
    else {
        return Ok(());
    };

    let member = match &reaction.member {
        Some(member) => member.clone(),
        None => guild_id.member(ctx, user_id).await?,
    };
    let bot = guild_id.member(ctx, bot_id).await?;
    let placement = guild_id
        .to_guild_cached(ctx)
        .map(|guild| {
            let update_channel = guild.channels.get(&reaction.channel_id);
            decide_placement(&guild, &member, &bot, None, update_channel)
        })
        .ok_or_else(|| anyhow!("Guild {guild_id} is not cached"))?;
    let placement = match placement {
        Ok(placement) => placement,
        Err(reason) => {
            refuse(ctx, reaction, user_id, &reason).await;
            return Ok(());
        }
    };

    let calendar = data
        .chombot()
        .tournaments()
        .get_within(TOURNAMENT_LOOKUP_TIMEOUT)
        .await;
    let entry = posted.current_entry(calendar.as_ref().map_or(&[], |calendar| &calendar.0));
    let name = entry
        .as_ref()
        .and_then(tournament_channel_name)
        .or_else(|| ChannelName::new(&posted.name))
        .ok_or_else(|| anyhow!("{:?} is not usable as a channel name", posted.name))?;
    let creation = create_text_channel(
        ctx,
        guild_id,
        TextChannelRequest {
            name,
            placement,
            topic: entry.as_ref().map(tournament_topic),
            audit_log_reason: &format!(
                "Requested by {} via a TOURNAMENTS UPDATE reaction",
                member.user.name
            ),
        },
    )
    .await?;

    let content = match creation {
        ChannelCreation::Created(channel) => format!(
            "<@{user_id}> created <#{}> for **{}**.",
            channel.id, posted.name
        ),
        ChannelCreation::AlreadyExists(channel) => format!("<#{}> already exists.", channel.id),
    };
    reaction
        .channel_id
        .send_message(
            ctx,
            CreateMessage::new()
                .content(content)
                .reference_message((reaction.channel_id, reaction.message_id))
                .allowed_mentions(CreateAllowedMentions::new()),
        )
        .await?;
    Ok(())
}

/// Removes the reaction and tells the member why in a DM, as far as Discord
/// allows (removing it needs Manage Messages; DMs may be closed).
async fn refuse(ctx: &Context, reaction: &Reaction, user_id: UserId, reason: &str) {
    if let Err(err) = reaction.delete(ctx).await {
        info!("Could not remove a refused tournament reaction: {err}");
    }
    let dm = match user_id.create_dm_channel(ctx).await {
        Ok(dm) => dm.say(ctx, reason).await.map(|_| ()),
        Err(err) => Err(err),
    };
    if let Err(err) = dm {
        info!("Could not tell {user_id} why their tournament reaction was refused: {err}");
    }
}
