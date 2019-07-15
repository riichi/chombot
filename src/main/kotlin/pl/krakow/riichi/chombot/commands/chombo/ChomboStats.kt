package pl.krakow.riichi.chombot.commands.chombo

import kotlinx.serialization.Serializable
import java.util.*
import kotlin.collections.ArrayList

@Serializable
class ChomboStats {
    private val list: ArrayList<ChomboEvent> = ArrayList()

    fun addEvent(event: ChomboEvent) {
        list.add(event)
    }

    val chomboCounter: Map<Long, Int>
        get() = list.map { event -> event.userId }.groupingBy { x -> x }.eachCount()

    val chomboList: List<ChomboEvent>
        get() = Collections.unmodifiableList(list)
}
