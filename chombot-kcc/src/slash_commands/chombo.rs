use anyhow::Result;
use chombot_common::data::{DISCORD_EMBED_FIELD_LIMIT, DISCORD_MESSAGE_SIZE_LIMIT};
use poise::serenity_prelude::{Color, CreateAllowedMentions, CreateEmbed, User};
use poise::CreateReply;
use slug::slugify;

use crate::chombot::Chombot;
use crate::kcc3::data_types::{Chombo, ChomboWeight, DiscordId, Player, PlayerId};
use crate::PoiseContext;

#[poise::command(slash_command, subcommands("ranking", "list", "add"))]
pub async fn chombo(_: PoiseContext<'_>) -> Result<()> {
    #![allow(clippy::unused_async)]
    Ok(())
}

/// Display the chombo ranking.
#[poise::command(slash_command)]
async fn ranking(ctx: PoiseContext<'_>) -> Result<()> {
    let entries = get_chombos_embed_entries(&ctx.data().kcc_chombot).await?;

    ctx.send(CreateReply::default().embed(create_chombos_embed(entries)))
        .await?;

    Ok(())
}

/// List all chombos.
#[poise::command(slash_command)]
async fn list(ctx: PoiseContext<'_>) -> Result<()> {
    let chombos = create_chombos_list(&ctx.data().kcc_chombot).await?;

    ctx.send(
        CreateReply::default()
            .content(chombos)
            .allowed_mentions(CreateAllowedMentions::new().empty_users().empty_roles()),
    )
    .await?;

    Ok(())
}

/// Add a chombo for a user.
#[poise::command(slash_command)]
async fn add(
    ctx: PoiseContext<'_>,
    #[description = "User that made a chombo"] user: User,
    #[description = "Chombo description"] description: String,
    #[description = "MERS tournament weight (default: 1)"] weight: Option<ChomboWeight>,
) -> Result<()> {
    let chombot = &ctx.data().kcc_chombot;
    chombot
        .add_chombo_for_player(
            |player| player.discord_id.0 == user.id.to_string(),
            || {
                Player::new_from_discord(
                    PlayerId(slugify(&user.name)),
                    user.name.clone(),
                    DiscordId(user.id.to_string()),
                )
            },
            &description,
            weight.unwrap_or_default(),
        )
        .await?;

    let message_content = format_add_message(&user, &description);
    let entries = get_chombos_embed_entries(chombot).await?;

    ctx.send(
        CreateReply::default()
            .content(message_content)
            .embed(create_chombos_embed(entries)),
    )
    .await?;

    Ok(())
}

async fn get_chombos_embed_entries(
    chombot: &Chombot,
) -> Result<impl Iterator<Item = (String, String, bool)>> {
    let chombo_ranking = chombot.create_chombo_ranking().await?;
    Ok(chombo_ranking
        .into_iter()
        .take(DISCORD_EMBED_FIELD_LIMIT)
        .map(|(player, score)| (player.short_name(), score.to_string(), true)))
}

fn create_chombos_embed(entries: impl Iterator<Item = (String, String, bool)>) -> CreateEmbed {
    CreateEmbed::new()
            .title("**CHOMBO COUNTER**")
            .color(Color::RED)
            .thumbnail("https://cdn.discordapp.com/attachments/591385176685281293/597292309792686090/1562356453777.png")
            .fields(entries)
}

fn format_add_message(user: &User, description: &str) -> String {
    format!("Adding chombo for <@!{}>: *{}*", user.id, description)
}

async fn create_chombos_list(chombot: &Chombot) -> Result<String> {
    let chombo_list = chombot.get_chombo_list().await?;
    let mut result = String::new();
    for (player, chombo) in &chombo_list {
        let entry = format_chombo_entry(player, chombo);
        if result.len() + entry.len() <= DISCORD_MESSAGE_SIZE_LIMIT {
            result += &entry;
        } else {
            break;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn test_player() -> Player {
        Player::new_from_discord(
            PlayerId("test".to_string()),
            "TestPlayer".to_string(),
            DiscordId("123456".to_string()),
        )
    }

    fn test_chombo(comment: &str, weight: ChomboWeight) -> Chombo {
        let timestamp = Utc.with_ymd_and_hms(2025, 3, 15, 14, 30, 0).unwrap();
        Chombo::new(timestamp, &PlayerId("test".to_string()), comment, weight)
    }

    #[test]
    fn format_chombo_entry_default_weight_with_comment() {
        let result = format_chombo_entry(
            &test_player(),
            &test_chombo("broke the wall", ChomboWeight::W1),
        );
        assert_eq!(
            result,
            "<@!123456> at Saturday, 2025-03-15 14:30: *broke the wall*\n"
        );
    }

    #[test]
    fn format_chombo_entry_custom_weight_with_comment() {
        let result = format_chombo_entry(
            &test_player(),
            &test_chombo("broke the wall", ChomboWeight::W2_5),
        );
        assert_eq!(
            result,
            "<@!123456> at Saturday, 2025-03-15 14:30 (x2.5): *broke the wall*\n"
        );
    }

    #[test]
    fn format_chombo_entry_default_weight_no_comment() {
        let result = format_chombo_entry(&test_player(), &test_chombo("", ChomboWeight::W1));
        assert_eq!(result, "<@!123456> at Saturday, 2025-03-15 14:30\n");
    }

    #[test]
    fn format_chombo_entry_custom_weight_no_comment() {
        let result = format_chombo_entry(&test_player(), &test_chombo("", ChomboWeight::W2));
        assert_eq!(result, "<@!123456> at Saturday, 2025-03-15 14:30 (x2)\n");
    }
}

fn format_chombo_entry(player: &Player, chombo: &Chombo) -> String {
    let comment = if chombo.comment.is_empty() {
        String::new()
    } else {
        format!(": *{}*", chombo.comment)
    };
    let weight = if chombo.weight == ChomboWeight::default() {
        String::new()
    } else {
        format!(" (x{})", chombo.weight)
    };
    let timestamp = chombo.timestamp.format("%A, %Y-%m-%d %H:%M");

    format!(
        "<@!{}> at {}{}{}\n",
        player.discord_id, timestamp, weight, comment
    )
}
