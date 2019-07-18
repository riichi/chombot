package pl.krakow.riichi.chombot.commands.hand

import java.awt.geom.AffineTransform
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import java.io.InputStream
import javax.imageio.ImageIO

class PNGHandRenderer {
    companion object {
        const val RESOURCE_PATH_PREFIX = "/riichi-mahjong-tiles/Export"
        const val TILE_SYMBOLS_SCALE = 0.8
    }

    private fun getTileBasename(tile: Tile): String {
        if (tile.suite == Suite.ANY)
            return "Back.png"

        if (tile.suite == Suite.UNKNOWN)
            throw Exception("Invalid tile suite: unknown")

        if (tile.suite == Suite.HONOR)
            return when(tile.value) {
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

    private fun getTileTransform(scale: Double, rotated: Boolean, xOffset: Int): AffineTransform {
        val realScale = scale / 2
        // We scale around the origin which moves tile center a bit – so we have to make up for it.
        val shiftH = 400 * (1.0 - scale) / 2
        val shiftW = 300 * (1.0 - scale) / 2
        return if (rotated)
            AffineTransform(0.0, -realScale, realScale, 0.0, xOffset.toDouble() + shiftH, 400.0 - shiftW)
        else
            AffineTransform(realScale, 0.0, 0.0, realScale, xOffset.toDouble() + shiftW, shiftH)
    }

    private fun getTileTransform(tile:Tile, xOffset: Int): AffineTransform {
        // Suite.ANY means here either back of a tile or a background (Front.png).
        return if (tile.suite == Suite.ANY)
            getTileTransform(1.0, tile.rotated, xOffset)
        else
            getTileTransform(TILE_SYMBOLS_SCALE, tile.rotated, xOffset)
    }

    fun renderHand(hand: Hand): ByteArray {
        var width: Int = 100 * (hand.groups.size - 1)
        val height = 400
        for (group in hand.groups) {
            width += group.map { tile -> if (tile.rotated) 400 else 300 }.sum()
        }
        val bi = BufferedImage(width, height, BufferedImage.TYPE_INT_ARGB)
        val graphics = bi.createGraphics()

        var xOffset = 0
        for (group in hand.groups) {
            for (tile in group) {
                if (tile.suite != Suite.ANY) {
                    val frontImage = ImageIO.read(getFrontImageInputStream(hand.style))
                    graphics.drawImage(frontImage, getTileTransform(1.0, tile.rotated, xOffset), null)
                }
                val tileImage = ImageIO.read(getTileImageInputStream(tile, hand.style))
                graphics.drawImage(tileImage, getTileTransform(tile, xOffset), null)
                xOffset += if (tile.rotated) 400 else 300
            }
            xOffset += 100
        }
        val out = ByteArrayOutputStream()
        ImageIO.write(bi, "PNG", out)
        return out.toByteArray()
    }
}