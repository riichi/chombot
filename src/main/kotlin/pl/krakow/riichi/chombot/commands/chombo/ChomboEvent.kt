package pl.krakow.riichi.chombot.commands.chombo

import kotlinx.serialization.Serializable
import java.util.*

@Serializable
data class ChomboEvent(
    @Serializable(with=DateSerializer::class) val timestamp: Date,
    val userId: Long,
    val comment: String
)
