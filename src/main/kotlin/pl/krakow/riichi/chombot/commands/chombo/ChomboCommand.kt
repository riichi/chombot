package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.event.domain.message.MessageCreateEvent
import kotlinx.serialization.UnstableDefault
import pl.krakow.riichi.chombot.commands.Command
import pl.krakow.riichi.chombot.commands.kcc3client.Chombo
import pl.krakow.riichi.chombot.commands.kcc3client.Kcc3Client
import reactor.core.publisher.Mono
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter

@UnstableDefault
class ChomboCommand(private val formatter: Formatter, private val kcc3Client: Kcc3Client) : Command {
    companion object {
        private val DATE_FORMATTER = DateTimeFormatter.ofPattern("EEE, yyyy-MM-dd HH:mm")
    }

    override fun execute(event: MessageCreateEvent): Mono<Void> {
        val subcommand = event.message.content.get().split(Regex("""\s+""")).getOrNull(1).orEmpty()

        return when {
            isMention(subcommand) -> addChombo(event)
            subcommand == "list" -> listChombos(event)
            else -> displayCounter(event)
        }
    }

    private fun isMention(word: String): Boolean {
        return word.startsWith("<@") && word.endsWith(">")
    }

    private fun addChombo(event: MessageCreateEvent): Mono<Void> {
        val discordId = event.message.userMentionIds.first().asString()
        val players = kcc3Client.getPlayers()
        val player = players.find { it.discordId == discordId }
        val comment = event.message.content.get().substringAfter("<@$discordId>").trim()

        if (player == null) {
            return event.message.channel.flatMap { channel ->
                channel.createMessage("Mentioned Discord user (ID $discordId) is not a player (yet)!")
            }.then()
        }

        kcc3Client.addChombo(
            Chombo(
                ZonedDateTime.now(),
                player.id,
                comment
            )
        )

        return displayCounter(event)
    }

    private fun getChomboCounter(): Map<String, Int> =
        kcc3Client.getChombos().map { event -> event.player }.groupingBy { x -> x }.eachCount()

    private fun displayCounter(event: MessageCreateEvent): Mono<Void> {
        val counter = getChomboCounter()
        val players = kcc3Client.getPlayerMap()
        val mapping = counter.mapKeys { (key, _) -> players.getValue(key) }

        return event.message.channel.flatMap { channel ->
            channel.createEmbed(formatter.format(mapping))
        }.then()
    }

    private fun listChombos(event: MessageCreateEvent): Mono<Void> {
        val chombos = kcc3Client.getChombos()
        val players = kcc3Client.getPlayerMap()

        return event.message.channel.flatMap { channel ->
            channel.createMessage(chombos.joinToString("\n") { chombo ->
                val playerName = players[chombo.player]?.name
                val timestampString = DATE_FORMATTER.format(chombo.timestamp)
                val comment = chombo.comment

                var s = "**$playerName** at $timestampString"
                if (comment.isNotEmpty()) {
                    s += ": *$comment*"
                }

                s
            })
        }.then()
    }
}
