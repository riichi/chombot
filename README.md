chombot
=======

[![Build Status](https://github.com/riichi/chombot/workflows/Rust%20CI/badge.svg)](https://github.com/riichi/chombot/actions)
[![Docker Builds](https://github.com/riichi/chombot/workflows/Docker/badge.svg)](https://github.com/riichi/chombot/actions)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/riichi/chombot/blob/master/LICENSE)
[![codecov](https://codecov.io/gh/riichi/chombot/branch/master/graph/badge.svg)](https://codecov.io/gh/riichi/chombot)

Discord bot with many features built for Riichi Mahjong servers.

## Building
The project is written in Rust and uses Cargo build system.
```shell
cargo build --release
```

## Running
Chombot requires a few settings passed via environment variables. Those are:
* `DISCORD_TOKEN` - the Discord bot token. You can obtain one by [creating a Discord app](https://discord.com/developers/applications). Make sure to copy your Bot token, not a client secret.
* `KCC3_URL` - the URL of the [kcc3 instance](https://github.com/riichi/kcc3) that you want to use with Chombot. The official instance is `https://fanpai.chombo.club`.
* `KCC3_TOKEN` - the API token for kcc3 that can be obtained via kcc3 admin page.
* `GUILD_ID` - your guild ID.
* `RANKING_WATCHER_CHANNEL_ID` - ID of the channel used for notifications about ranking updates.
* `TOURNAMENTS_WATCHER_CHANNEL_ID` - ID of the channel used for notifications about EMA tournament updates.

Example:

```shell
export CHOMBOT_TOKEN=yourdiscordtoken
export KCC3_URL=https://fanpai.chombo.club
export KCC3_TOKEN=yourkcc3token
export GUILD_ID=12345
export RANKING_WATCHER_CHANNEL_ID=54321
export TOURNAMENTS_WATCHER_CHANNEL_ID=98765
cargo run --release
```

## Developing
### `pre-commit`
We encourage contributors to use predefined [`pre-commit`](https://pre-commit.com/)
hooks --- to install them in your local repo, make sure you have `pre-commit`
installed and run
```shell
pre-commit install
```
