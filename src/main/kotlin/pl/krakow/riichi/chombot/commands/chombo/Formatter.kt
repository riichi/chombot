package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.`object`.entity.User
import discord4j.core.spec.EmbedCreateSpec
import java.util.function.Consumer

interface Formatter {
    fun format(stats: Map<User, Int>): Consumer<EmbedCreateSpec>
}
