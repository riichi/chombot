package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.`object`.entity.User
import discord4j.core.spec.EmbedCreateSpec
import java.awt.Color
import java.util.function.Consumer

class SimpleEmbedFormatter : Formatter {
    override fun format(stats: Map<User, Int>): Consumer<EmbedCreateSpec> {
        return Consumer { spec ->
            spec.setTitle("**CHOMBO COUNTER**")
            spec.setColor(Color.RED)
            spec.setThumbnail("https://cdn.discordapp.com/attachments/591385176685281293/597292309792686090/1562356453777.png")
            stats
                .toList()
                .sortedByDescending { pair -> pair.second }
                .forEach { pair -> spec.addField("**${pair.first.username}**", pair.second.toString(), true) }
        }
    }
}