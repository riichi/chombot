package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.spec.EmbedCreateSpec
import pl.krakow.riichi.chombot.commands.kcc3client.Player
import java.awt.Color
import java.util.function.Consumer

class SimpleEmbedFormatter : Formatter {
    override fun format(stats: Map<Player, Int>): Consumer<EmbedCreateSpec> {
        return Consumer { spec ->
            spec.setTitle("**CHOMBO COUNTER**")
            spec.setColor(Color.RED)
            spec.setThumbnail("https://cdn.discordapp.com/attachments/591385176685281293/597292309792686090/1562356453777.png")
            stats
                .toList()
                .sortedByDescending { pair -> pair.second }
                .forEach { (player, num) -> spec.addField("**${player.shortName}**", num.toString(), true) }
        }
    }
}
