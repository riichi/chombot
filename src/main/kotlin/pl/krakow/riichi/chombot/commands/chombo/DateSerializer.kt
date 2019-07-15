package pl.krakow.riichi.chombot.commands.chombo

import kotlinx.serialization.*
import kotlinx.serialization.internal.StringDescriptor
import java.util.*

@Serializer(forClass = Date::class)
object DateSerializer : KSerializer<Date> {
    override val descriptor: SerialDescriptor =
        StringDescriptor.withName("DateSerializer")

    override fun serialize(encoder: Encoder, obj: Date) {
        encoder.encodeString(obj.time.toString())
    }

    override fun deserialize(decoder: Decoder): Date {
        return Date(decoder.decodeString().toLong())
    }
}
