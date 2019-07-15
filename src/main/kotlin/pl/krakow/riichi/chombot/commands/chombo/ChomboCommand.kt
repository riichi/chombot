package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.DiscordClient
import discord4j.core.`object`.entity.User
import discord4j.core.event.domain.message.MessageCreateEvent
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonConfiguration
import pl.krakow.riichi.chombot.commands.Command
import reactor.core.publisher.Mono
import java.io.File
import java.util.*

class ChomboCommand(private val formatter: Formatter) : Command {
    companion object {
        private const val CHOMBOS_FILENAME = "chombos.json"
    }

    private val stats = if (File(CHOMBOS_FILENAME).exists()) {
        loadState()
    } else {
        ChomboStats()
    }

    private fun prepareMapping(mapping: Map<Long, Int>, client: DiscordClient): Mono<Map<User, Int>> {
        return client.users
            .filter { user -> mapping.containsKey(user.id.asLong()) }
            .collectMap({ user -> user }, { user -> mapping.getValue(user.id.asLong()) })
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
        val user = event.message.userMentionIds.first()
        val comment = event.message.content.get().substringAfter("<@${user.asLong()}>").trim()

        stats.addEvent(
            ChomboEvent(
                Calendar.getInstance().time,
                user.asLong(),
                comment
            )
        )

        saveState()

        return displayCounter(event)
    }

    private fun displayCounter(event: MessageCreateEvent): Mono<Void> {
        val userToScore = prepareMapping(stats.chomboCounter, event.client)
        return event.message.channel.flatMap { channel ->
            userToScore.flatMap { mapping -> channel.createEmbed(formatter.format(mapping)) }
        }.then()
    }

    private fun listChombos(event: MessageCreateEvent): Mono<Void> {
        return event.message.channel.flatMap { channel ->
            channel.createMessage(stats.chomboList.joinToString("\n") { chombo ->
                "<@${chombo.userId}> at ${chombo.timestamp}: ${chombo.comment}"
            })
        }.then()
    }

    private fun loadState(): ChomboStats {
        val json = Json(JsonConfiguration.Stable)
        val jsonString = File(CHOMBOS_FILENAME).readText()
        return json.parse(ChomboStats.serializer(), jsonString)
    }

    private fun saveState() {
        val json = Json(JsonConfiguration.Stable)
        val jsonString = json.stringify(ChomboStats.serializer(), stats)
        File(CHOMBOS_FILENAME).writeText(jsonString)
    }
}
