#![allow(clippy::struct_excessive_bools)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use async_trait::async_trait;
use chombot_common::tournaments_watcher::notifier::TournamentWatcherChannelListProvider;
use log::info;
use poise::serenity_prelude::{ChannelId, GuildId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Guild-specific configs
    #[serde(default)]
    pub guilds: HashMap<GuildId, GuildConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct GuildConfig {
    /// Tournaments watcher channel ID
    pub tournaments_watcher_channel_id: Option<ChannelId>,
}

#[async_trait]
impl TournamentWatcherChannelListProvider for ChombotConfig {
    type TournamentWatcherChannelList = Vec<ChannelId>;

    async fn tournament_watcher_channels(&self) -> Self::TournamentWatcherChannelList {
        self.config()
            .guilds
            .values()
            .filter_map(|config| config.tournaments_watcher_channel_id)
            .collect()
    }
}

#[derive(Debug)]
pub struct ChombotConfig {
    path: PathBuf,
    config: Config,
}

impl ChombotConfig {
    #[must_use]
    pub const fn new(path: PathBuf, config: Config) -> Self {
        Self { path, config }
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let config = if path.try_exists()? {
            info!("Loading config file: {}", path.to_string_lossy());
            let file_contents = fs::read_to_string(&path)?;
            toml::from_str(&file_contents)?
        } else {
            info!(
                "Config file {} not found; using default config",
                path.to_string_lossy()
            );
            Config::default()
        };

        Ok(Self { path, config })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let contents = toml::to_string(&self.config)?;
        fs::write(&self.path, contents)?;

        Ok(())
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Mutates the config with `f` and saves it to disk afterwards.
    ///
    /// # Errors
    ///
    /// Returns an error if the config could not be serialized or written to
    /// disk. Note that in that case the mutation is still applied in memory.
    pub fn update<T>(&mut self, f: impl FnOnce(&mut Config) -> T) -> anyhow::Result<T> {
        let result = f(&mut self.config);
        self.save()?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chombot_common::tournaments_watcher::notifier::TournamentWatcherChannelListProvider;
    use poise::serenity_prelude::ChannelId;
    use tempfile::NamedTempFile;

    use crate::config::{ChombotConfig, Config, GuildConfig, GuildId};

    #[test]
    fn test_config() {
        let file = NamedTempFile::new().unwrap();
        let path = file.into_temp_path();

        let config = Config {
            guilds: HashMap::from([
                (
                    GuildId::new(69),
                    GuildConfig {
                        tournaments_watcher_channel_id: Some(ChannelId::new(2137)),
                    },
                ),
                (
                    GuildId::new(420),
                    GuildConfig {
                        tournaments_watcher_channel_id: Some(ChannelId::new(69)),
                    },
                ),
            ]),
        };

        {
            let chombot_config = ChombotConfig::new(path.to_path_buf(), config.clone());
            chombot_config.save().unwrap();
        }
        {
            let chombot_config = ChombotConfig::load(path.to_path_buf()).unwrap();
            assert_eq!(*chombot_config.config(), config);
        }

        path.close().unwrap();
    }

    #[test]
    fn test_update_saves_config() {
        let file = NamedTempFile::new().unwrap();
        let path = file.into_temp_path();

        {
            let mut chombot_config = ChombotConfig::load(path.to_path_buf()).unwrap();
            let returned = chombot_config
                .update(|config| {
                    config
                        .guilds
                        .entry(GuildId::new(69))
                        .or_default()
                        .tournaments_watcher_channel_id = Some(ChannelId::new(2137));
                    "returned from the closure"
                })
                .unwrap();
            assert_eq!(returned, "returned from the closure");
        }
        {
            let chombot_config = ChombotConfig::load(path.to_path_buf()).unwrap();
            assert_eq!(
                chombot_config.config().guilds[&GuildId::new(69)].tournaments_watcher_channel_id,
                Some(ChannelId::new(2137)),
            );
        }

        path.close().unwrap();
    }

    #[test]
    fn test_tournament_watcher_channel_list_provider_for_chombo_config() -> std::io::Result<()> {
        let file = NamedTempFile::new().unwrap();
        let path = file.into_temp_path();

        let config = Config {
            guilds: HashMap::from([
                (
                    GuildId::new(69),
                    GuildConfig {
                        tournaments_watcher_channel_id: Some(ChannelId::new(2137)),
                    },
                ),
                (
                    GuildId::new(420),
                    GuildConfig {
                        tournaments_watcher_channel_id: Some(ChannelId::new(69)),
                    },
                ),
            ]),
        };

        let channel_ids: Vec<ChannelId> = vec![ChannelId::new(69), ChannelId::new(2137)];

        {
            let chombot_config = ChombotConfig::new(path.to_path_buf(), config);
            let mut ids = tokio::runtime::Builder::new_current_thread()
                .build()?
                .block_on(async { chombot_config.tournament_watcher_channels().await });
            ids.sort();
            assert_eq!(ids, channel_ids);
        }

        path.close().unwrap();

        Ok(())
    }
}
