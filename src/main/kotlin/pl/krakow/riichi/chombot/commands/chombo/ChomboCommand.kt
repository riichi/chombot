package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.DiscordClient
import discord4j.core.`object`.entity.User
import discord4j.core.`object`.util.Snowflake
import discord4j.core.event.domain.message.MessageCreateEvent
import pl.krakow.riichi.chombot.commands.Command
import reactor.core.publisher.Mono
import reactor.core.publisher.toFlux

class ChomboCommand(fmt: Formatter) : Command {
    private val stats = ChomboStats()
    private val formatter: Formatter = fmt

    private fun prepareMapping(mapping: Map<Snowflake, Int>, client: DiscordClient): Mono<Map<User, Int>> {
        return client.users
            .filter { user -> mapping.containsKey(user.id) }
            .collectMap({ user -> user }, { user -> mapping.getValue(user.id) })
    }

    override fun execute(event: MessageCreateEvent): Mono<Void> {
        if (event.message.userMentionIds.isEmpty())
            return Mono.empty() // TODO maybe emit some error message
        stats.applyChombo(event.message.userMentionIds.toFlux())
        val userToScore = prepareMapping(stats.mapping, event.client)
        return event.message.channel.flatMap { channel ->
            userToScore.flatMap { mapping -> channel.createEmbed(formatter.format(mapping)) }
        }.then()
    }
}
