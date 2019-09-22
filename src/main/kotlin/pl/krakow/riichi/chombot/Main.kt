package pl.krakow.riichi.chombot

import discord4j.core.DiscordClientBuilder
import discord4j.core.event.domain.lifecycle.ReadyEvent
import discord4j.core.event.domain.message.MessageCreateEvent
import kotlinx.serialization.UnstableDefault
import pl.krakow.riichi.chombot.commands.AkagiInflationRate
import pl.krakow.riichi.chombot.commands.AtEveryoneAngryReactions
import pl.krakow.riichi.chombot.commands.chombo.ChomboCommand
import pl.krakow.riichi.chombot.commands.chombo.SimpleEmbedFormatter
import pl.krakow.riichi.chombot.commands.hand.DrawHandCommand
import pl.krakow.riichi.chombot.commands.kcc3client.Kcc3Client
import reactor.core.publisher.Flux
import reactor.core.publisher.Mono
import java.net.URI
import kotlin.system.exitProcess


@UnstableDefault
fun main() {

    val discordToken = getEnvVariable("CHOMBOT_TOKEN")
    val kcc3Url = getEnvVariable("KCC3_URL")
    val kcc3Token = getEnvVariable("KCC3_TOKEN")

    val client = DiscordClientBuilder(discordToken).build()
    val kcc3Client = Kcc3Client(URI(kcc3Url), kcc3Token)

    client.eventDispatcher.on(ReadyEvent::class.java)
        .subscribe { ready -> println("Logged in as " + ready.self.username) }

    val commandMap = mapOf(
        "chombo" to ChomboCommand(SimpleEmbedFormatter(), kcc3Client),
        "hand" to DrawHandCommand(),
        "_inflation" to AkagiInflationRate(),
        "_everyone" to AtEveryoneAngryReactions()
    )

    client.eventDispatcher.on(MessageCreateEvent::class.java)
        .flatMap { event ->
            if (event.message.author.get().isBot) {
                Mono.empty()
            } else {
                Flux.fromIterable(commandMap.entries)
                    .filter { entry -> entry.value.isApplicable(event, entry.key) }
                    .flatMap { entry -> entry.value.execute(event) }
                    .onErrorResume { error ->
                        event.message.channel.flatMap { channel ->
                            channel.createMessage("Error occurred when executing the command: `$error`")
                        }.then()
                    }
                    .doOnError { error ->
                        error.printStackTrace()
                    }
                    .next()
            }
        }
        .subscribe()

    client.login().block()
}

private fun getEnvVariable(name: String): String {
    val value = System.getenv(name)

    if (value == null) {
        System.err.println("Environment variable not set: $name")
        exitProcess(1)
    }

    return value
}
