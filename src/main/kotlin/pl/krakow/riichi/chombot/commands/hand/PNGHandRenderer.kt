package pl.krakow.riichi.chombot.commands.hand

import java.awt.geom.AffineTransform
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import java.io.InputStream
import javax.imageio.ImageIO

class PNGHandRenderer {
    companion object {
        const val RESOURCE_PATH_PREFIX = "/tiles"
        const val TILE_SYMBOLS_SCALE = 0.8
        const val TILE_WIDTH = 300
        const val TILE_HEIGHT = 400
        const val GROUP_SKIP = 100
    }

    enum class TileRotation {
        NONE,
        ROTATED,
        INVERTED,
    }

    private val TilePosition.tileRotation: TileRotation
        get() = if (this != TilePosition.NORMAL) TileRotation.ROTATED else TileRotation.NONE

    private val TilePosition.tileRotationInverted: TileRotation
        get() = if (this != TilePosition.NORMAL) TileRotation.INVERTED else TileRotation.NONE

    private fun getTileBasename(tile: Tile): String {
        if (tile.suite == Suite.ANY)
            return "Back.png"

        if (tile.suite == Suite.UNKNOWN)
            throw Exception("Invalid tile suite: unknown")

        if (tile.suite == Suite.HONOR)
            return when (tile.value) {
                1 -> "Ton.png"
                2 -> "Nan.png"
                3 -> "Shaa.png"
                4 -> "Pei.png"
                5 -> "Haku.png"
                6 -> "Hatsu.png"
                7 -> "Chun.png"
                else -> throw Exception("Invalid honor tile value: ${tile.value}")
            }

        if (tile.value == 0)
            return "${tile.suite.filenamePrefix}5-Dora.png"

        return "${tile.suite.filenamePrefix}${tile.value}.png"
    }

    private fun getFrontImageInputStream(style: TileStyle): InputStream {
        return this::class.java.getResourceAsStream("$RESOURCE_PATH_PREFIX/${style.catalog}/Front.png")
    }

    private fun getTileImageInputStream(tile: Tile, style: TileStyle): InputStream {
        return this::class.java.getResourceAsStream(getTileResourcePath(tile, style))
    }

    private fun getTileResourcePath(tile: Tile, style: TileStyle): String {
        return "$RESOURCE_PATH_PREFIX/${style.catalog}/${getTileBasename(tile)}"
    }

    private fun getTileTransform(scale: Double, rotation: TileRotation, position: TilePosition, xOffset: Int, yOffset: Int): AffineTransform {
        val realScale = scale / 2
        // We scale around the origin which moves tile center a bit – so we have to make up for it.
        val shiftH = TILE_HEIGHT * (1.0 - scale) / 2
        val shiftW = TILE_WIDTH * (1.0 - scale) / 2

        return when (rotation) {
            TileRotation.NONE -> AffineTransform(
                realScale,
                0.0,
                0.0,
                realScale,
                xOffset.toDouble() + shiftW,
                yOffset.toDouble() + shiftH
            )
            TileRotation.ROTATED -> AffineTransform(
                0.0,
                -realScale,
                realScale,
                0.0,
                xOffset.toDouble() + shiftH,
                yOffset.toDouble() + TILE_WIDTH.toDouble() - shiftW
            )
            TileRotation.INVERTED -> AffineTransform(
                0.0,
                realScale,
                realScale,
                0.0,
                xOffset.toDouble() + shiftH,
                yOffset.toDouble() + shiftW
            )
        }
    }

    private fun getTileTransform(tile: Tile, xOffset: Int, yOffset: Int): AffineTransform {
        // Suite.ANY means here either back of a tile or a background (Front.png).
        return if (tile.suite == Suite.ANY)
            getTileTransform(1.0, tile.position.tileRotation, tile.position, xOffset, yOffset)
        else
            getTileTransform(TILE_SYMBOLS_SCALE, tile.position.tileRotation, tile.position, xOffset, yOffset)
    }

    private fun needToStartNewGroup(tilePosition: TilePosition, lower: Boolean, upper: Boolean): Boolean {
        return when (tilePosition) {
            TilePosition.NORMAL -> lower or upper
            TilePosition.ROTATED -> lower
            TilePosition.ROTATED_SHIFTED -> upper
        }
    }

    private fun groupWidth(group: List<Tile>): Int {
        var lowerRotated = false
        var upperRotated = false
        var width = 0
        for (tile in group) {
            if (needToStartNewGroup(tile.position, lowerRotated, upperRotated)) {
                width += TILE_HEIGHT
                lowerRotated = false
                upperRotated = false
            }
            when (tile.position) {
                TilePosition.NORMAL -> width += TILE_WIDTH
                TilePosition.ROTATED -> lowerRotated = true
                TilePosition.ROTATED_SHIFTED -> upperRotated = true
            }
        }
        if (lowerRotated or upperRotated)
            width += TILE_HEIGHT
        return width
    }

    fun renderHand(hand: Hand): ByteArray {
        val width: Int = GROUP_SKIP * (hand.groups.size - 1) + hand.groups.map { g -> groupWidth(g) }.sum()
        val height = (
                if (hand.groups.any { g -> g.any { t -> t.position == TilePosition.ROTATED_SHIFTED } })
                    2 * TILE_WIDTH
                else
                    TILE_HEIGHT
                )
        val bi = BufferedImage(width, height, BufferedImage.TYPE_INT_ARGB)
        val graphics = bi.createGraphics()

        var xOffset = 0
        for (group in hand.groups) {
            var lowerRotated = false
            var upperRotated = false
            for (tile in group) {
                if (needToStartNewGroup(tile.position, lowerRotated, upperRotated)) {
                    lowerRotated = false
                    upperRotated = false
                    xOffset += TILE_HEIGHT
                }
                val yOffset = when(tile.position) {
                    TilePosition.NORMAL -> height - TILE_HEIGHT
                    TilePosition.ROTATED -> height - TILE_WIDTH
                    TilePosition.ROTATED_SHIFTED -> 0
                };
                if (tile.suite != Suite.ANY) {
                    val frontImage = ImageIO.read(getFrontImageInputStream(hand.style))
                    graphics.drawImage(
                        frontImage,
                        getTileTransform(1.0, tile.position.tileRotationInverted, tile.position, xOffset, yOffset),
                        null
                    )
                }
                val tileImage = ImageIO.read(getTileImageInputStream(tile, hand.style))
                graphics.drawImage(tileImage, getTileTransform(tile, xOffset, yOffset), null)
                when (tile.position) {
                    TilePosition.NORMAL -> xOffset += TILE_WIDTH
                    TilePosition.ROTATED -> lowerRotated = true
                    TilePosition.ROTATED_SHIFTED -> upperRotated = true
                }
            }
            if (lowerRotated or upperRotated)
                xOffset += TILE_HEIGHT
            xOffset += GROUP_SKIP
        }
        val out = ByteArrayOutputStream()
        ImageIO.write(bi, "PNG", out)
        return out.toByteArray()
    }
}
