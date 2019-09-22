package pl.krakow.riichi.chombot.commands.kcc3client

import com.github.kittinunf.fuel.core.FuelError
import com.github.kittinunf.fuel.core.FuelManager
import com.github.kittinunf.fuel.core.extensions.jsonBody
import com.github.kittinunf.fuel.serialization.responseObject
import com.github.kittinunf.result.Result
import kotlinx.serialization.UnstableDefault
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonConfiguration
import kotlinx.serialization.list
import java.net.URI

@UnstableDefault
class Kcc3Client(kcc3Url: URI, private val token: String) {
    private val fuelManager = FuelManager().apply {
        baseHeaders = mapOf(
            "Accept" to "application/json",
            "Authorization" to "Token $token"
        )
    }
    private val apiUrl = kcc3Url.resolve("/api/")

    private val json = Json(JsonConfiguration.Stable)

    fun getPlayers(): List<Player> {
        val request = fuelManager.get(apiUrl.resolve("players/").toString())
        val (_, _, result) = request.responseObject(loader = Player.serializer().list, json = Json.nonstrict)

        return obtainResult(result)
    }

    fun getPlayerMap(): Map<String, Player> = getPlayers().associateBy { it.id }

    fun getChombos(): List<Chombo> {
        val request = fuelManager.get(apiUrl.resolve("chombos/").toString())
        val (_, _, result) = request.responseObject(loader = Chombo.serializer().list, json = Json.nonstrict)

        return obtainResult(result)
    }

    fun addChombo(chombo: Chombo) {
        val (_, _, result) = fuelManager
            .post(apiUrl.resolve("chombos/").toString())
            .jsonBody(json.stringify(Chombo.serializer(), chombo))
            .responseString()

        obtainResult(result)
    }

    private fun <T: Any> obtainResult(result: Result<T, FuelError>): T {
        when (result) {
            is Result.Failure -> {
                throw result.getException()
            }
            is Result.Success -> {
                return result.get()
            }
        }
    }
}
