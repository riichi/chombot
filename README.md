# chombot
Discord bot with many features built for Riichi Mahjong servers.

## Building
The project is written in Rust and uses Cargo build system.
```
cargo build --release
```

## Running
Chombot requires a few settings passed via environment variables. Those are:
* `CHOMBOT_TOKEN` - the Discord bot token. You can obtain one by [creating a Discord app](https://discord.com/developers/applications). Make sure to copy your Bot token, not a client secret.
* `KCC3_URL` - the URL of the [kcc3 instance](https://github.com/riichi/kcc3) that you want to use with Chombot. The official instance is `https://fanpai.chombo.club`.
* `KCC3_TOKEN` - the API token for kcc3 that can be obtained via kcc3 admin page.

Example:

```
export CHOMBOT_TOKEN=yourdiscordtoken
export KCC3_URL=https://fanpai.chombo.club
export KCC3_TOKEN=yourkcc3token
cargo run --release
```
