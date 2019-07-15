package pl.krakow.riichi.chombot.commands

import discord4j.core.event.domain.message.MessageCreateEvent
import reactor.core.publisher.Mono

class AkagiInflationRate : Command {
    companion object {
        const val MIN_VALUE: Long = 1000L
        const val MAX_VALUE: Long = 99_999_999_999L
    }

    private fun findNumber(message: String): Long? {
        return message.split(Regex("\\W+"))
            .mapNotNull { word -> word.toLongOrNull() }
            .find { number -> number in MIN_VALUE..MAX_VALUE }
    }

    private fun roundDiv(p: Long, q: Long): Long {
        val div: Long = p / q
        val mod: Long = p % q
        if (2 * mod >= q) {
            return div + 1
        }
        return div
    }

    private fun formatNumber(number: Long): String {
        return when {
            number >= 1_000_000_000 -> "${roundDiv(number, 1_000_000_000)} billion"
            number >= 1_000_000 -> "${roundDiv(number, 1_000_000)} million"
            else -> "${roundDiv(number, 1_000)} thousand"
        }
    }

    override fun execute(event: MessageCreateEvent): Mono<Void> {
        return event.message.content.orElse(null)?.let {
            findNumber(it)?.let {
                event.message.channel.flatMap { channel ->
                    channel.createMessage(
                        "This ${formatNumber(it)} in 1965 would equate to ${formatNumber(it * 10)} today."
                    )
                }.then()
            }
        } ?: Mono.empty()
    }

    override fun isApplicable(event: MessageCreateEvent, commandName: String): Boolean {
        return event.message.content.orElse(null)?.let { findNumber(it) } != null
    }
}
