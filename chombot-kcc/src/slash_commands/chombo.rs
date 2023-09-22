use anyhow::Result;
use chombot_common::data::DISCORD_MESSAGE_SIZE_LIMIT;
use poise::serenity_prelude::{Color, CreateEmbed, User};
use slug::slugify;

use crate::chombot::Chombot;
use crate::kcc3::data_types::{Chombo, DiscordId, Player, PlayerId};
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

    ctx.send(|response| response.embed(|embed| create_chombos_embed(embed, entries)))
        .await?;

    Ok(())
}

/// List all chombos.
#[poise::command(slash_command)]
async fn list(ctx: PoiseContext<'_>) -> Result<()> {
    let chombos = create_chombos_list(&ctx.data().kcc_chombot).await?;

    ctx.send(|response| {
        response
            .content(chombos)
            .allowed_mentions(|mentions| mentions.empty_parse())
    })
    .await?;

    Ok(())
}

/// Add a chombo for a user.
#[poise::command(slash_command)]
async fn add(
    ctx: PoiseContext<'_>,
    #[description = "User that made a chombo"] user: User,
    #[description = "Chombo description"] description: String,
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
        )
        .await?;

    let message_content = format_add_message(&user, &description);
    let entries = get_chombos_embed_entries(chombot).await?;

    ctx.send(|response| {
        response
            .content(message_content)
            .embed(|embed| create_chombos_embed(embed, entries))
    })
    .await?;

    Ok(())
}

async fn get_chombos_embed_entries(
    chombot: &Chombot,
) -> Result<impl Iterator<Item = (String, usize, bool)>> {
    let chombo_ranking = chombot.create_chombo_ranking().await?;
    Ok(chombo_ranking
        .into_iter()
        .map(|(player, num)| (player.short_name(), num, true)))
}

fn create_chombos_embed(
    embed: &mut CreateEmbed,
    entries: impl Iterator<Item = (String, usize, bool)>,
) -> &mut CreateEmbed {
    embed
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

fn format_chombo_entry(player: &Player, chombo: &Chombo) -> String {
    let comment = if chombo.comment.is_empty() {
        String::new()
    } else {
        format!(": *{}*", chombo.comment)
    };
    let timestamp = chombo.timestamp.format("%A, %Y-%m-%d %H:%M");

    format!("<@!{}> at {}{}\n", player.discord_id, timestamp, comment)
}
