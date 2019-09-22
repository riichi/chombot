package pl.krakow.riichi.chombot.commands.kcc3client

import kotlinx.serialization.*
import kotlinx.serialization.internal.StringDescriptor
import java.time.format.DateTimeFormatter.ISO_OFFSET_DATE_TIME
import java.time.temporal.TemporalAccessor

@Serializer(forClass = TemporalAccessor::class)
object DateSerializer : KSerializer<TemporalAccessor> {
    override val descriptor: SerialDescriptor =
        StringDescriptor.withName("DateSerializer")

    override fun serialize(encoder: Encoder, obj: TemporalAccessor) {
        encoder.encodeString(ISO_OFFSET_DATE_TIME.format(obj))
    }

    override fun deserialize(decoder: Decoder): TemporalAccessor {
        return ISO_OFFSET_DATE_TIME.parse(decoder.decodeString())
    }
}
