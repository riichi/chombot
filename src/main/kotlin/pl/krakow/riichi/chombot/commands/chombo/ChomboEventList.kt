package pl.krakow.riichi.chombot.commands.chombo

import java.util.function.Function
import java.util.stream.Collectors

fun List<ChomboEvent>.chomboCounter() {
    this.stream()
        .map { event -> event.user }
//        .collect(Collectors.groupingBy(Function.identity(), Collectors.counting()))
}
