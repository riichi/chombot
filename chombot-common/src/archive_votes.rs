use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use chrono::{Datelike, Utc};
use log::info;
use poise::serenity_prelude::{
    ChannelId, ChannelType, Context, CreateAllowedMentions, CreateMessage, EditChannel, Guild,
    GuildChannel, Member, Message, ReactionType, UserId,
};
use slug::slugify;

use crate::ChombotPoiseUserData;

const REQUIRED_VOTES: usize = 3;
const VOTE_LIFETIME: Duration = Duration::from_hours(24 * 7);

fn is_archive_vote(content: &str) -> bool {
    let content = content.to_lowercase();
    content.match_indices("#archiwum").any(|(index, word)| {
        !content[index + word.len()..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric)
    })
}

/// The year after the leading emoji (`🏆2026-02-…`), or else the first year
/// anywhere in the name (`hatsumi-taikai-2027`).
fn year_from_channel_name(name: &str) -> Option<i32> {
    let as_year = |word: &str| {
        let year = word.parse().ok()?;
        (word.len() == 4 && (2000..2100).contains(&year)).then_some(year)
    };
    let mut words = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty());
    let leading = words.clone().next().and_then(|word| {
        let digits: String = word.chars().take_while(char::is_ascii_digit).collect();
        as_year(&digits)
    });
    leading.or_else(|| words.find_map(as_year))
}

/// Matched ignoring case and emoji, so `❗Archiwum 2026` counts.
fn archive_category(guild: &Guild, year: i32) -> Option<&GuildChannel> {
    let wanted = format!("archiwum-{year}");
    guild
        .channels
        .values()
        .find(|channel| channel.kind == ChannelType::Category && slugify(&channel.name) == wanted)
}

fn is_archived(guild: &Guild, channel: &GuildChannel) -> bool {
    channel
        .parent_id
        .and_then(|id| guild.channels.get(&id))
        .is_some_and(|category| slugify(&category.name).starts_with("archiwum"))
}

/// Has a role with "Zarząd" in its name, or may manage the channel.
fn is_zarzad(guild: &Guild, member: &Member, channel: &GuildChannel) -> bool {
    member
        .roles
        .iter()
        .filter_map(|role_id| guild.roles.get(role_id))
        .any(|role| slugify(&role.name).split('-').any(|word| word == "zarzad"))
        || guild.user_permissions_in(channel, member).manage_channels()
}

#[derive(Clone, Copy)]
struct Vote {
    user_id: UserId,
    zarzad: bool,
    at: Instant,
}

struct Tally {
    passes: bool,
    newly_counted: bool,
}

/// `#archiwum` votes per channel, kept in memory.
#[derive(Default)]
pub struct ArchiveVotes {
    votes: Mutex<HashMap<ChannelId, Vec<Vote>>>,
}

impl ArchiveVotes {
    fn vote(&self, channel_id: ChannelId, vote: Vote) -> Tally {
        let mut all = self.lock();
        let votes = all.entry(channel_id).or_default();
        votes.retain(|old| vote.at.saturating_duration_since(old.at) < VOTE_LIFETIME);
        let newly_counted = !votes.iter().any(|old| old.user_id == vote.user_id);
        if newly_counted {
            votes.push(vote);
        }

        let tally = Tally {
            passes: votes.len() >= REQUIRED_VOTES && votes.iter().any(|vote| vote.zarzad),
            newly_counted,
        };
        drop(all);
        tally
    }

    fn clear(&self, channel_id: ChannelId) {
        self.lock().remove(&channel_id);
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<ChannelId, Vec<Vote>>> {
        self.votes.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

struct VoteContext {
    zarzad: bool,
    year: i32,
    archive: Option<GuildChannel>,
}

/// `None` if the channel can't be archived.
fn vote_context(guild: &Guild, member: &Member, channel_id: ChannelId) -> Option<VoteContext> {
    let channel = guild.channels.get(&channel_id)?;
    if channel.kind == ChannelType::Category || is_archived(guild, channel) {
        return None;
    }
    let year = year_from_channel_name(&channel.name).unwrap_or_else(|| Utc::now().year());
    Some(VoteContext {
        zarzad: is_zarzad(guild, member, channel),
        year,
        archive: archive_category(guild, year).cloned(),
    })
}

/// Counts `#archiwum` votes; once 3 members including someone from Zarząd
/// have voted, moves the channel to `Archiwum <year>` (the year from its
/// name). Needs the Message Content intent.
pub async fn handle_message<U: ChombotPoiseUserData>(
    ctx: &Context,
    message: &Message,
    data: &U,
) -> Result<()> {
    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };
    if message.author.bot || !is_archive_vote(&message.content) {
        return Ok(());
    }

    let member = guild_id.member(ctx, message.author.id).await?;
    let Some(context) = guild_id
        .to_guild_cached(ctx)
        .map(|guild| vote_context(&guild, &member, message.channel_id))
        .ok_or_else(|| anyhow!("Guild {guild_id} is not cached"))?
    else {
        return Ok(());
    };

    let votes = data.chombot().archive_votes();
    let tally = votes.vote(
        message.channel_id,
        Vote {
            user_id: message.author.id,
            zarzad: context.zarzad,
            at: Instant::now(),
        },
    );
    if !tally.passes {
        if tally.newly_counted
            && let Err(err) = message
                .react(ctx, ReactionType::Unicode("🗳️".to_owned()))
                .await
        {
            info!("Could not confirm an archive vote: {err}");
        }
        return Ok(());
    }

    let text = match context.archive {
        None => format!(
            "There's no **Archiwum {}** category to move this channel to.",
            context.year
        ),
        Some(archive) => {
            let moved = EditChannel::new()
                .category(archive.id)
                .permissions(archive.permission_overwrites);
            match message.channel_id.edit(ctx, moved).await {
                Ok(_) => {
                    votes.clear(message.channel_id);
                    format!("📦 Moved to **{}**.", archive.name)
                }
                Err(err) => format!(
                    "Couldn't move this channel to **{}** (I need Manage Channels and Manage \
                     Roles): {err}",
                    archive.name
                ),
            }
        }
    };
    message
        .channel_id
        .send_message(
            ctx,
            CreateMessage::new()
                .content(text)
                .allowed_mentions(CreateAllowedMentions::new()),
        )
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use poise::serenity_prelude::{
        GuildId, PermissionOverwrite, PermissionOverwriteType, Permissions, Role, RoleId,
    };

    use super::*;

    #[test]
    fn recognizes_votes() {
        assert!(is_archive_vote("to już do #Archiwum!"));
        assert!(!is_archive_vote("archiwum"));
        assert!(!is_archive_vote("#archiwum2025"));
    }

    #[test]
    fn reads_years_from_channel_names() {
        assert_eq!(
            year_from_channel_name("🏆2026-12-kraków-cup-2027"),
            Some(2026)
        );
        assert_eq!(year_from_channel_name("🎉-2025-wigilia"), Some(2025));
        assert_eq!(year_from_channel_name("hatsumi-taikai-2027"), Some(2027));
        assert_eq!(year_from_channel_name("pokój-12345"), None);
    }

    const ZARZAD: u64 = 2000;
    const MODERATORS: u64 = 2001;

    fn channel(id: u64, kind: ChannelType, name: &str, parent: Option<u64>) -> GuildChannel {
        let mut channel = GuildChannel::default();
        channel.id = ChannelId::new(id);
        channel.kind = kind;
        channel.name = name.to_owned();
        channel.parent_id = parent.map(ChannelId::new);
        channel
    }

    fn guild() -> Guild {
        let mut hatsumi = channel(30, ChannelType::Text, "🏆2026-02-hatsumi", Some(10));
        hatsumi.permission_overwrites = vec![PermissionOverwrite {
            allow: Permissions::MANAGE_CHANNELS,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(RoleId::new(MODERATORS)),
        }];

        let mut guild = Guild::default();
        guild.id = GuildId::new(1);
        guild.owner_id = UserId::new(999);
        guild.roles = [
            (1, "@everyone"),
            (ZARZAD, "Zarząd"),
            (MODERATORS, "Moderatorzy"),
        ]
        .map(|(id, name)| {
            let mut role = Role::default();
            role.id = RoleId::new(id);
            role.name = name.to_owned();
            (role.id, role)
        })
        .into();
        guild.channels = [
            channel(10, ChannelType::Category, "Turnieje", None),
            channel(20, ChannelType::Category, "❗Archiwum 2026", None),
            hatsumi,
            channel(31, ChannelType::Text, "🏆2024-05-thorn", Some(10)),
            channel(32, ChannelType::Text, "🏆2026-11-cup", Some(20)),
        ]
        .map(|channel| (channel.id, channel))
        .into();
        guild
    }

    fn member(user_id: u64, roles: &[u64]) -> Member {
        let mut member = Member::default();
        member.user.id = UserId::new(user_id);
        member.roles = roles.iter().copied().map(RoleId::new).collect();
        member
    }

    #[test]
    fn finds_zarzad_and_the_archive_for_the_year() {
        let guild = guild();
        let context = |member: &Member, channel: u64| {
            vote_context(&guild, member, ChannelId::new(channel))
                .map(|context| (context.zarzad, context.archive.map(|c| c.id.get())))
        };

        assert_eq!(context(&member(1, &[ZARZAD]), 30), Some((true, Some(20))));
        assert_eq!(
            context(&member(2, &[MODERATORS]), 30),
            Some((true, Some(20)))
        );
        assert_eq!(context(&member(3, &[]), 30), Some((false, Some(20))));
        assert_eq!(context(&member(3, &[]), 31), Some((false, None)));
        // Already archived.
        assert_eq!(context(&member(1, &[ZARZAD]), 32), None);
    }

    #[test]
    fn passes_with_three_recent_voters_including_zarzad() {
        let votes = ArchiveVotes::default();
        let channel = ChannelId::new(30);
        let now = Instant::now();
        let vote = |user_id: u64, zarzad: bool, at: Instant| {
            votes.vote(
                channel,
                Vote {
                    user_id: UserId::new(user_id),
                    zarzad,
                    at,
                },
            )
        };

        assert!(vote(1, false, now).newly_counted);
        assert!(!vote(1, false, now).newly_counted);
        assert!(!vote(2, false, now).passes);
        assert!(!vote(3, false, now).passes);
        assert!(vote(4, true, now).passes);

        let later = now + VOTE_LIFETIME + Duration::from_secs(1);
        assert!(!vote(5, true, later).passes);
    }
}
