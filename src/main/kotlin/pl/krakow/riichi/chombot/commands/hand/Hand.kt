package pl.krakow.riichi.chombot.commands.hand

enum class TileStyle(val catalog: String) {
    REGULAR("Regular"),
    BLACK("Black")
}

enum class Suite(val filenamePrefix: String) {
    MANZU("Man"),
    PINZU("Pin"),
    SOUZU("Sou"),
    HONOR("Honor"),
    ANY("Any"),
    UNKNOWN("Unknown")
}

data class Tile(var suite: Suite, var value: Int, var rotated: Boolean)

data class Hand(var style: TileStyle, var groups: List<List<Tile>>) {
    val numberOfTiles: Int
        get() = groups.map { x -> x.size }.sum()

    val isEmpty: Boolean
        get() = this.numberOfTiles == 0
}
