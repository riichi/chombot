package pl.krakow.riichi.chombot.commands.hand

enum class TileStyle(val catalog: String) {
    RED("Red"),
    BLACK("Black"),
    YELLOW("Yellow")
}

enum class Suite(val filenamePrefix: String) {
    MANZU("Man"),
    PINZU("Pin"),
    SOUZU("Sou"),
    HONOR("Honor"),
    ANY("Any"),
    UNKNOWN("Unknown")
}

enum class TilePosition {
    NORMAL,
    ROTATED,
    ROTATED_SHIFTED
}

data class Tile(var suite: Suite, var value: Int, var position: TilePosition)

data class Hand(var style: TileStyle, var groups: List<List<Tile>>) {
    val numberOfTiles: Int
        get() = groups.map { x -> x.size }.sum()

    val isEmpty: Boolean
        get() = this.numberOfTiles == 0
}
