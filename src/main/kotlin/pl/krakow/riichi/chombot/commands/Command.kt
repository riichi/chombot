package pl.krakow.riichi.chombot.commands

import discord4j.core.event.domain.message.MessageCreateEvent
import reactor.core.publisher.Mono

interface Command {
    fun execute(event: MessageCreateEvent): Mono<Void>

    fun isApplicable(event: MessageCreateEvent, commandName: String): Boolean {
        if (!event.message.content.isPresent) {
            return false
        }

        return event.message.content.get().startsWith("!$commandName")
    }
}
