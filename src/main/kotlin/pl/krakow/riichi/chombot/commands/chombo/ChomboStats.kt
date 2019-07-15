package pl.krakow.riichi.chombot.commands.chombo

import discord4j.core.`object`.util.Snowflake
import reactor.core.publisher.Flux
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger

class ChomboStats {
    private val counter = ConcurrentHashMap<Snowflake, AtomicInteger>()

    fun applyChombo(users: Flux<Snowflake>) {
        users.subscribe { user -> counter.getOrPut(user, { AtomicInteger(0) }).incrementAndGet() }
    }

    val mapping: Map<Snowflake, Int>
        get() = counter.mapValues { entry -> entry.value.get() }
}