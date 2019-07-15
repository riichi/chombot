package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.event.domain.message.MessageCreateEvent
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonConfiguration
import pl.krakow.riichi.chombot.commands.Command
import reactor.core.publisher.Mono

class ChomboCommand : Command {
    override fun execute(event: MessageCreateEvent): Mono<Void> {
        return event.message.channel.flatMap { channel -> channel.createMessage(event.message.author.get().mention) }
            .then()
    }

    fun serializeChomboEvent(event: ChomboEvent): String {
        val json = Json(JsonConfiguration.Stable)
        return json.stringify(ChomboEvent.serializer(), event)
    }
}
