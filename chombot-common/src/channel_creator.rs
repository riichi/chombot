use anyhow::Result;
use poise::serenity_prelude::{
    CacheHttp, ChannelId, ChannelType, CreateChannel, Guild, GuildChannel, GuildId, Member,
};

pub const CHANNEL_NAME_MAX_LENGTH: usize = 100;
const CHANNEL_TOPIC_MAX_LENGTH: usize = 1024;

/// A text channel name normalized the way Discord does it, so that existing
/// channels can be found by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelName(String);

impl ChannelName {
    #[must_use]
    pub fn new(raw: &str) -> Option<Self> {
        let mut name = String::with_capacity(raw.len());
        let mut pending_separator = false;
        for c in raw.chars() {
            if is_separator(c) {
                pending_separator = true;
            } else if !is_dropped(c) {
                if pending_separator && !name.is_empty() {
                    name.push('-');
                }
                pending_separator = false;
                name.extend(c.to_lowercase());
            }
        }

        let name = truncate_chars(&name, CHANNEL_NAME_MAX_LENGTH);
        let name = name.trim_end_matches('-');
        (!name.is_empty()).then(|| Self(name.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whitespace, punctuation and en/em dash (`–`, `—`) characters: become `-`.
const fn is_separator(c: char) -> bool {
    c.is_whitespace()
        || matches!(c, '-' | '\u{2013}' | '\u{2014}')
        || (c.is_ascii_punctuation() && c != '_' && !is_dropped(c))
}

/// Control and quote (including `‘`, `’`, `“`, `”`) characters: removed.
const fn is_dropped(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\'' | '"' | '`' | '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}'
        )
}

pub struct TextChannelRequest<'a> {
    pub name: ChannelName,
    pub category: Option<ChannelId>,
    pub topic: Option<String>,
    pub audit_log_reason: &'a str,
}

pub enum ChannelCreation {
    Created(GuildChannel),
    AlreadyExists(GuildChannel),
}

/// Creates a text channel, unless one with the same name already exists in
/// the same category.
pub async fn create_text_channel(
    cache_http: impl CacheHttp,
    guild_id: GuildId,
    request: TextChannelRequest<'_>,
) -> Result<ChannelCreation> {
    let channels = guild_id.channels(cache_http.http()).await?;
    if let Some(existing) = find_text_channel(channels.values(), &request.name, request.category) {
        return Ok(ChannelCreation::AlreadyExists(existing.clone()));
    }

    let mut builder = CreateChannel::new(request.name.as_str())
        .kind(ChannelType::Text)
        .audit_log_reason(request.audit_log_reason);
    if let Some(category) = request.category {
        builder = builder.category(category);
    }
    if let Some(topic) = request.topic {
        builder = builder.topic(truncate_chars(&topic, CHANNEL_TOPIC_MAX_LENGTH));
    }

    let channel = guild_id.create_channel(cache_http, builder).await?;
    Ok(ChannelCreation::Created(channel))
}

fn find_text_channel<'a>(
    channels: impl IntoIterator<Item = &'a GuildChannel>,
    name: &ChannelName,
    category: Option<ChannelId>,
) -> Option<&'a GuildChannel> {
    channels.into_iter().find(|channel| {
        matches!(channel.kind, ChannelType::Text | ChannelType::News)
            && channel.parent_id == category
            && channel.name == name.as_str()
    })
}

/// The category of `channel`, or of its parent channel if it's a thread.
fn default_category(guild: &Guild, channel: &GuildChannel) -> Option<ChannelId> {
    if matches!(
        channel.kind,
        ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread
    ) {
        guild.channels.get(&channel.parent_id?)?.parent_id
    } else {
        channel.parent_id
    }
}

fn can_create_channels_in(guild: &Guild, member: &Member, category: Option<&GuildChannel>) -> bool {
    category
        .map_or_else(
            || guild.member_permissions(member),
            |category| guild.user_permissions_in(category, member),
        )
        .manage_channels()
}

/// The category for a new channel, `None` meaning outside any category.
///
/// That's `requested`, or else the category of `current_channel`, if both the
/// member and the bot could create a channel there; otherwise an error
/// explaining to the member why not.
pub fn decide_placement(
    guild: &Guild,
    author: &Member,
    bot: &Member,
    requested: Option<ChannelId>,
    current_channel: Option<&GuildChannel>,
) -> Result<Option<ChannelId>, String> {
    let category_id =
        requested.or_else(|| current_channel.and_then(|channel| default_category(guild, channel)));
    let category = match category_id.map(|id| guild.channels.get(&id)) {
        Some(None) => return Err("I couldn't find that category.".to_owned()),
        Some(category) => category,
        None => None,
    };
    let place = category.map_or_else(
        || "outside a category".to_owned(),
        |category| format!("in the **{}** category", category.name),
    );

    if !can_create_channels_in(guild, author, category) {
        Err(format!(
            "You don't have permission to create channels {place}, so I can't do it for you."
        ))
    } else if !can_create_channels_in(guild, bot, category) {
        Err(format!(
            "I don't have the Manage Channels permission {place}."
        ))
    } else {
        Ok(category_id)
    }
}

#[must_use]
pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use poise::serenity_prelude::{
        PermissionOverwrite, PermissionOverwriteType, Permissions, Role, RoleId, UserId,
    };

    use super::*;

    fn name(raw: &str) -> Option<String> {
        ChannelName::new(raw).map(|name| name.0)
    }

    #[test]
    fn normalizes_names() {
        assert_eq!(
            name("Kraków Riichi Open 2026").as_deref(),
            Some("kraków-riichi-open-2026")
        );
        assert_eq!(
            name("ERMC 2026 (Copenhagen) / Tom's  -- Cup").as_deref(),
            Some("ermc-2026-copenhagen-toms-cup")
        );
        assert_eq!(name("🀄 ŁÓDŹ_CUP").as_deref(), Some("🀄-łódź_cup"));
        assert_eq!(name("!!! ''"), None);

        let long = name(&format!("{} tail", "a".repeat(99))).unwrap();
        assert_eq!(long, "a".repeat(99));
    }

    fn channel(id: u64, kind: ChannelType, parent: Option<u64>) -> GuildChannel {
        let mut channel = GuildChannel::default();
        channel.id = ChannelId::new(id);
        channel.kind = kind;
        channel.name = format!("channel-{id}");
        channel.parent_id = parent.map(ChannelId::new);
        channel
    }

    #[test]
    fn finds_existing_channel_in_same_category_only() {
        let channels = [
            channel(1, ChannelType::Text, Some(100)),
            channel(2, ChannelType::Voice, None),
        ];
        let find = |raw: &str, category: Option<u64>| {
            find_text_channel(
                &channels,
                &ChannelName::new(raw).unwrap(),
                category.map(ChannelId::new),
            )
            .map(|channel| channel.id.get())
        };

        assert_eq!(find("Channel 1", Some(100)), Some(1));
        assert_eq!(find("channel-1", None), None);
        assert_eq!(find("channel-2", None), None);
    }

    const ORGANIZERS: u64 = 2000;
    const TOURNAMENTS: u64 = 100;
    const MODS: u64 = 200;

    /// Organizers may manage channels only in the tournaments category.
    fn guild() -> Guild {
        let mut tournaments = channel(TOURNAMENTS, ChannelType::Category, None);
        tournaments.name = "Turnieje".to_owned();
        tournaments.permission_overwrites = vec![PermissionOverwrite {
            allow: Permissions::MANAGE_CHANNELS,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(RoleId::new(ORGANIZERS)),
        }];

        let mut guild = Guild::default();
        guild.id = GuildId::new(1);
        guild.owner_id = UserId::new(999);
        guild.roles = [1, ORGANIZERS]
            .map(|id| {
                let mut role = Role::default();
                role.id = RoleId::new(id);
                (role.id, role)
            })
            .into();
        guild.channels = [
            tournaments,
            channel(MODS, ChannelType::Category, None),
            channel(10, ChannelType::Text, Some(TOURNAMENTS)),
            channel(11, ChannelType::Text, None),
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
    fn places_channel_where_member_and_bot_may_create_it() {
        let guild = guild();
        let general = &guild.channels[&ChannelId::new(10)];
        let lobby = &guild.channels[&ChannelId::new(11)];
        let thread = channel(12, ChannelType::PublicThread, Some(10));
        let organizer = member(5, &[ORGANIZERS]);
        let bot = member(6, &[ORGANIZERS]);
        let tournaments = Some(ChannelId::new(TOURNAMENTS));
        let place =
            |author: &Member, bot: &Member, requested: Option<u64>, current: &GuildChannel| {
                decide_placement(
                    &guild,
                    author,
                    bot,
                    requested.map(ChannelId::new),
                    Some(current),
                )
            };

        assert_eq!(place(&organizer, &bot, None, general), Ok(tournaments));
        assert_eq!(place(&organizer, &bot, None, &thread), Ok(tournaments));
        assert_eq!(
            place(&organizer, &bot, Some(TOURNAMENTS), lobby),
            Ok(tournaments)
        );
        assert_eq!(
            place(&organizer, &bot, None, lobby),
            Err(
                "You don't have permission to create channels outside a category, so I can't do \
                 it for you."
                    .to_owned()
            )
        );
        assert!(place(&organizer, &bot, Some(MODS), general).is_err());
        assert!(place(&member(7, &[]), &bot, None, general).is_err());
        assert_eq!(
            place(&organizer, &member(8, &[]), None, general),
            Err(
                "I don't have the Manage Channels permission in the **Turnieje** category."
                    .to_owned()
            )
        );
        assert_eq!(
            place(&organizer, &bot, Some(999), general),
            Err("I couldn't find that category.".to_owned())
        );
    }
}
