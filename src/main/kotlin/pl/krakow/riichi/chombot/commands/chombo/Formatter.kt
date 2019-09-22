package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.spec.EmbedCreateSpec
import pl.krakow.riichi.chombot.commands.kcc3client.Player
import java.util.function.Consumer

interface Formatter {
    fun format(stats: Map<Player, Int>): Consumer<EmbedCreateSpec>
}
