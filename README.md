chombot
=======

[![Rust Build Status](https://github.com/riichi/chombot/workflows/Rust%20CI/badge.svg)](https://github.com/riichi/chombot/actions/workflows/rust.yml)
[![Docker Build Status](https://github.com/riichi/chombot/workflows/Docker/badge.svg)](https://github.com/riichi/chombot/actions/workflows/docker-publish.yml)
[![MIT licensed](https://img.shields.io/github/license/riichi/chombot)](https://github.com/riichi/chombot/blob/master/LICENSE)
[![codecov](https://codecov.io/gh/riichi/chombot/branch/master/graph/badge.svg)](https://codecov.io/gh/riichi/chombot)

Discord bot with many features built for Riichi Mahjong servers.

## Building
The project is written in Rust and uses Cargo build system.
```shell
cargo build --release
```

## Running
Chombot requires some config values defined as environment variables. These are:

* `DISCORD_TOKEN` - the Discord bot token. You can obtain one by [creating a Discord app](https://discord.com/developers/applications). Make sure to copy your Bot token, not a client secret.

Example:

```shell
export DISCORD_TOKEN=yourdiscordtoken
cargo run --bin chombot --release
```

Please note that the working directory serves as a persistence storage for the bot's state, so the bot process must have write permissions for it.

### chombot-kcc
In addition to the base version of Chombot, there is also an enhanced version called Chombot-kcc available. This is version tailored for the needs of Krakow Chombo Club and contains a few additional (possibly hermetic) features.

Chombot-kcc requires more config values defined as environment variables. Those are:
* `FEATURE_TOURNAMENTS_WATCHER` - `true`, if you want to receive the notification about EMA tournament updates.
* `TOURNAMENTS_WATCHER_CHANNEL_ID` - ID of the channel used for notifications about EMA tournament updates.
* `GUILD_ID` - your guild ID.
* `FEATURE_KCC3` - `true` if you want to enable the integration with [kcc3](https://github.com/riichi/kcc3).
* `KCC3_URL` - the URL of the [kcc3 instance](https://github.com/riichi/kcc3) that you want to use with Chombot. The official instance is `https://fanpai.chombo.club`.
* `KCC3_TOKEN` - the API token for kcc3 that can be obtained via kcc3 admin page.
* `FEATURE_RANKING_WATCHER` - `true` if you want to receive notifications about USMA ranking updates.
* `RANKING_WATCHER_CHANNEL_ID` - ID of the channel used for notifications about ranking updates.
* `FEATURE_PASTA` - `true` if you want to enable the `/pasta` command (extremely hermetic!).

Example:

```shell
export DISCORD_TOKEN=yourdiscordtoken
export GUILD_ID=12345
export FEATURE_KCC3=true
export KCC3_URL=https://fanpai.chombo.club
export KCC3_TOKEN=yourkcc3token
export FEATURE_RANKING_WATCHER=true
export RANKING_WATCHER_CHANNEL_ID=54321
export FEATURE_TOURNAMENTS_WATCHER=true
export TOURNAMENTS_WATCHER_CHANNEL_ID=98765
export FEATURE_PASTA=true
cargo run --bin chombot-kcc --release
```

## Developing
### `pre-commit`
We encourage contributors to use predefined [`pre-commit`](https://pre-commit.com/)
hooks --- to install them in your local repo, make sure you have `pre-commit`
installed and run
```shell
pre-commit install
```
