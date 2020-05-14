# chombot
Discord bot with many features built for Riichi Mahjong servers.

## Building
The project is written in Kotlin and uses Gradle build system.
```
./gradlew build
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
./gradlew run
```

## Attribution
This project uses modified [riichi-mahjong-tiles](https://github.com/FluffyStuff/riichi-mahjong-tiles) by [FluffyStuff](https://github.com/FluffyStuff), licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
