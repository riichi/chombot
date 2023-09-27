use anyhow::anyhow;
use poise::serenity_prelude::ChannelId;

use crate::{config, PoiseContext};

/// Set the notification channel for the tournament watcher.
#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
pub async fn tournament_watcher(
    ctx: PoiseContext<'_>,
    #[description = "Channel"] channel: Option<ChannelId>,
) -> anyhow::Result<()> {
    let guild = ctx.guild_id().ok_or_else(|| anyhow!("Guild ID is None"))?;

    {
        let mut config = ctx.data().config.write().await;
        config
            .config_mut()
            .guilds
            .entry(config::GuildId::new(guild.0))
            .or_default()
            .tournaments_watcher_channel_id = channel.map(|x| x.0);
    }

    let reply_content = channel.as_ref().map_or_else(
        || "Disabled the tournament watcher.".to_string(),
        |channel| format!("Set the tournament watcher channel to <#{}>.", channel.0),
    );
    ctx.send(|reply| reply.content(reply_content)).await?;

    Ok(())
}
