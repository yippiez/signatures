const val MAX = 5

class Greeter(val name: String) {
    fun greet(): String = "hi $name"
    suspend fun load() {}
}

data class Point(val x: Int, val y: Int)

interface Shape {
    fun area(): Double
}

object Registry {
    fun register() {}
}
