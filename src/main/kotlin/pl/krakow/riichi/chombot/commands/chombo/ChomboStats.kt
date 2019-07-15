package pl.krakow.riichi.chombot.commands.chombo

import kotlinx.serialization.Serializable

@Serializable
class ChomboStats {
    private val list: ArrayList<ChomboEvent> = ArrayList()

    fun addEvent(event: ChomboEvent) {
        list.add(event)
    }

    val chomboCounter: Map<Long, Int>
        get() = list.map { event -> event.userId }.groupingBy { x -> x }.eachCount()

}
