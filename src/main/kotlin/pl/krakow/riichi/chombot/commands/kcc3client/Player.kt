package pl.krakow.riichi.chombot.commands.kcc3client

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class Player(
    val id: String,
    @SerialName("first_name")
    val firstName: String?,
    @SerialName("last_name")
    val lastName: String?,
    val nickname: String?,
    @SerialName("discord_id")
    val discordId: String?
) {
    val name: String
        get() {
            if (!firstName.isNullOrBlank() && !lastName.isNullOrBlank()) {
                var s = "$firstName $lastName"

                if (!nickname.isNullOrBlank()) {
                    s += " ($nickname)"
                }

                return s
            }

            return requireNotNull(nickname)
        }

    val shortName: String
        get() {
            if (!nickname.isNullOrBlank()) {
                return nickname
            }

            return "$firstName $lastName"
        }
}
