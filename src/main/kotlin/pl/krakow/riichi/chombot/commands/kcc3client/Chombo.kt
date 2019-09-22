package pl.krakow.riichi.chombot.commands.kcc3client

import kotlinx.serialization.Serializable
import java.time.temporal.TemporalAccessor

@Serializable
data class Chombo(
    @Serializable(with = DateSerializer::class)
    val timestamp: TemporalAccessor,
    val player: String,
    val comment: String
)
