package pl.krakow.riichi.chombot.commands.hand

import discord4j.core.`object`.entity.MessageChannel
import discord4j.core.event.domain.message.MessageCreateEvent
import pl.krakow.riichi.chombot.commands.Command
import reactor.core.publisher.Mono

/*
 * Usage: !hand [-w | -b | hand_description]...
 *  - -w sets style to white (default),
 *  - -b sets style to black.
 */
class DrawHandCommand : Command {
    class InvalidParameterException(parameter: String) : Exception("Invalid parameter: '$parameter'")

    companion object {
        const val MAX_TILES = 50

        val tileStyleMapping = mapOf(
            'w' to TileStyle.REGULAR,
            'b' to TileStyle.BLACK
        )

        val tileSuiteMapping = mapOf(
            'm' to Suite.MANZU,
            'p' to Suite.PINZU,
            's' to Suite.SOUZU,
            'z' to Suite.HONOR
        )

        val specialHonorSymbolsMapping = mapOf(
            'E' to 1,
            'S' to 2,
            'W' to 3,
            'N' to 4,
            'w' to 5,
            'g' to 6,
            'r' to 7
        )
    }

    private fun parseArgs(message: String): List<Hand> {
        var tileStyle = TileStyle.REGULAR
        val ret = ArrayList<Hand>()

        message.split(Regex("\\s+")).drop(1).forEach {
            if (it.startsWith('-')) {
                if (it.length != 2 || !tileStyleMapping.containsKey(it[1]))
                    throw InvalidParameterException(it)
                tileStyle = tileStyleMapping.getValue(it[1])
            } else {
                val hand = parseHandDescription(it, tileStyle)
                if (hand.isEmpty || hand.numberOfTiles >= MAX_TILES) {
                    throw InvalidParameterException(it)
                }

                ret.add(hand)
            }
        }

        return ret
    }

    private fun isTileValid(value: Int, suite: Suite): Boolean {
        return when (suite) {
            Suite.MANZU -> value in 0..9
            Suite.PINZU -> value in 0..9
            Suite.SOUZU -> value in 0..9
            Suite.HONOR -> value in 1..7
            Suite.ANY -> true
            Suite.UNKNOWN -> false
        }
    }

    private fun parseHandDescription(description: String, style: TileStyle): Hand {
        // Tiles for which we don't know the suite yet - it is given
        // at the end of the group. When suite type is encountered,
        // we set it for elements of the list and clear the list.
        val unknownSuite = ArrayList<Tile>()
        // Whether to skip current iteration. This is necessary for
        // parsing [0-9]\* tiles - with classic for loop we'd just
        // increment the variable once.
        var skip = false
        val groups = ArrayList<ArrayList<Tile>>()
        var currentGroup = ArrayList<Tile>()
        for (i in 0 until description.length) {
            if (skip) {
                skip = false
                continue
            }
            val cur = description[i]
            if (cur in '0'..'9') {
                skip = (i < description.length - 1 && description[i + 1] == '*')
                val tile = Tile(Suite.UNKNOWN, cur - '0', skip)
                unknownSuite.add(tile)
                currentGroup.add(tile)
                continue
            }
            if (cur == '?') {
                skip = (i < description.length - 1 && description[i + 1] == '*')
                currentGroup.add(Tile(Suite.ANY, 0, skip))
                continue
            }
            if (cur == '_') {
                groups.add(currentGroup)
                currentGroup = ArrayList()
                continue
            }
            if (cur in tileSuiteMapping.keys) {
                val suite = tileSuiteMapping.getValue(cur)
                for (tile in unknownSuite) {
                    if (!isTileValid(tile.value, suite))
                        throw InvalidParameterException(description)
                    tile.suite = suite
                }
                unknownSuite.clear()
                continue
            }
            if (cur in specialHonorSymbolsMapping) {
                skip = (i < description.length - 1 && description[i + 1] == '*')
                currentGroup.add(Tile(Suite.HONOR, specialHonorSymbolsMapping.getValue(cur), skip))
                continue
            }
            throw InvalidParameterException(description)
        }
        if (unknownSuite.isNotEmpty())
            throw InvalidParameterException(description)
        if (currentGroup.isNotEmpty())
            groups.add(currentGroup)
        return Hand(style, groups)
    }

    private fun sendHand(hand: Hand, channel: MessageChannel): Mono<Void> {
        val handBytes = PNGHandRenderer().renderHand(hand)
        return channel.createMessage { spec ->
            spec.addFile("hand.png", handBytes.inputStream())
        }.then()
    }

    override fun execute(event: MessageCreateEvent): Mono<Void> {
        if (!event.message.content.isPresent)
            return Mono.empty()
        return try {
            val hands = parseArgs(event.message.content.get())
            event.message.channel.flatMap { channel ->
                hands.forEach { hand -> sendHand(hand, channel).block() }
                Mono.empty<Void>()
            }
        } catch (e: InvalidParameterException) {
            event.message.channel.flatMap { channel ->
                channel.createMessage(e.localizedMessage)
            }.then()
        }
    }
}
