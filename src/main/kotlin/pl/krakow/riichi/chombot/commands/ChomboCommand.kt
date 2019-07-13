package pl.krakow.riichi.chombot.commands

import discord4j.core.event.domain.message.MessageCreateEvent
import reactor.core.publisher.Mono

class ChomboCommand : Command {
    override fun execute(event: MessageCreateEvent): Mono<Void> {
        return event.message.channel.flatMap { channel -> channel.createMessage("Ron!") }.then()
    }
}
