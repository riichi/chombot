package pl.krakow.riichi.chombot.commands.chombo

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonConfiguration
import kotlinx.serialization.list

class ChomboEventSerializer {
    companion object {
        fun serializeEvents(list: List<ChomboEvent>): String {
            val json = Json(JsonConfiguration.Stable)
            return json.stringify(ChomboEvent.serializer().list, list)
        }

        fun deserializeEvents(string: String): List<ChomboEvent> {
            val json = Json(JsonConfiguration.Stable)
            return json.parse(ChomboEvent.serializer().list, string)
        }
    }
}
