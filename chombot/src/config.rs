#![allow(clippy::struct_excessive_bools)]

use std::collections::HashMap;
use std::fs;
use std::ops::{Deref, DerefMut};
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
        let channel_ids = self
            .config
            .guilds
            .iter()
            .filter_map(|(_, config)| config.tournaments_watcher_channel_id);

        channel_ids.collect()
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

    pub const fn config_mut(&mut self) -> ConfigUpdateGuard<'_> {
        ConfigUpdateGuard::new(self)
    }
}

#[derive(Debug)]
#[must_use]
pub struct ConfigUpdateGuard<'a> {
    config: &'a mut ChombotConfig,
}

impl<'a> ConfigUpdateGuard<'a> {
    pub const fn new(config: &'a mut ChombotConfig) -> Self {
        Self { config }
    }
}

impl Drop for ConfigUpdateGuard<'_> {
    fn drop(&mut self) {
        self.config.save().expect("Could not save Chombot config");
    }
}

impl Deref for ConfigUpdateGuard<'_> {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config.config
    }
}

impl DerefMut for ConfigUpdateGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config.config
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
            let mut chombot_config = ChombotConfig::load(path.to_path_buf()).unwrap();
            let config_guard = chombot_config.config_mut();
            assert_eq!(*config_guard, config);
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
