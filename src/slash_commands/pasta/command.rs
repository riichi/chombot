use anyhow::Result;
use poise::serenity_prelude::CacheHttp;
use poise::ChoiceParameter;

use crate::data::DISCORD_MESSAGE_SIZE_LIMIT;
use crate::PoiseContext;

#[derive(Debug, ChoiceParameter)]
pub enum Pasta {
    #[name = "O jgamesach na conach"]
    JGamesCon,
    #[name = "Expert 1 do wora a wór do jeziora"]
    Tanjalo,
    #[name = "Tylko nie zakładajcie mu stowarzyszenia"]
    Stowarzyszenie,
    #[name = "Yostar rant"]
    Yostar,
}

impl Pasta {
    pub fn content(&self) -> &'static str {
        match self {
            Self::JGamesCon => include_str!("jgamescon.txt"),
            Self::Tanjalo => include_str!("tanjalo.txt"),
            Self::Stowarzyszenie => include_str!("stowarzyszenie.txt"),
            Self::Yostar => include_str!("yostar.txt"),
        }
    }
}

/// Paste a pasta 🍝
#[poise::command(slash_command)]
pub async fn pasta(
    ctx: PoiseContext<'_>,
    #[description = "Copypasta to output"] pasta: Pasta,
) -> Result<()> {
    let pasta_content = format!("{}\n||#pasta||", pasta.content().trim());

    let mut first = true;

    let mut message = String::new();
    for line in pasta_content.lines() {
        if message.len() + line.len() + "\n".len() > DISCORD_MESSAGE_SIZE_LIMIT {
            send_pasta_slice(&ctx, &message, &mut first).await?;
            message.clear();
        }

        message.push_str(line);
        message.push('\n');
    }
    if !message.is_empty() {
        send_pasta_slice(&ctx, &message, &mut first).await?;
    }

    Ok(())
}

async fn send_pasta_slice(ctx: &PoiseContext<'_>, message: &str, first: &mut bool) -> Result<()> {
    if *first {
        *first = false;
        ctx.say(message).await?;
    } else {
        ctx.channel_id()
            .send_message(&ctx.http(), |m| m.content(message))
            .await?;
    }

    Ok(())
}
