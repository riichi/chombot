package pl.krakow.riichi.chombot.commands

import discord4j.core.`object`.reaction.ReactionEmoji
import discord4j.core.event.domain.message.MessageCreateEvent
import reactor.core.publisher.Mono

class AtEveryoneAngryReactions : Command {
    companion object {
        val reactions = setOf(
            "Ichiangry",
            "Mikiknife"
        )
    }

    override fun execute(event: MessageCreateEvent): Mono<Void> {
        return event.guild.flatMap { guild ->
            guild.emojis.filter { guildEmoji ->
                reactions.contains(guildEmoji.name)
            }.toIterable().forEach { emoji ->
                event.message.addReaction(ReactionEmoji.custom(emoji)).block()
            }
            Mono.empty<Void>()
        }.then()
    }

    override fun isApplicable(event: MessageCreateEvent, commandName: String): Boolean {
        return event.message.mentionsEveryone()
    }
}