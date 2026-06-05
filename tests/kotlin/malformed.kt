@file:Suppress("ALL")
package com.example.malformed

// This file has unusual formatting and edge-case syntax that is malformed
// but should not cause the signatures tool to panic.

const val MISSING_TYPE_ANNOTATION = 42
val noExplicitType = "inferred"
var mutableNoType = listOf<Int>()

// Function missing return type (valid in Kotlin when returning Unit)
fun noReturnType(x: Int) {
    println(x)
}

// Very long parameter list split across many lines
fun multiLineParams(
    param1: String,
    param2: Int,
    param3: Double,
    param4: Boolean,
    param5: List<String>,
    param6: Map<String, Any>,
): String {
    return param1
}

// Unusual whitespace and formatting
class   WeirdlyFormatted   (   val x : Int ) {
    fun   spaceyMethod (  )  :  String  {
        return "weird"
    }
    val   spaceyProp   :   Int   =   0
}

// Class with no body (valid in Kotlin)
class EmptyBody

// Interface with only default implementations
interface AllDefaults {
    fun one(): Int = 1
    fun two(): String = "two"
    fun three(): Boolean = false
}

// Nested generics
fun deepGeneric(input: Map<String, List<Pair<Int, Set<Double>>>>): Map<String, Int> {
    return emptyMap()
}

// Lambda-heavy signatures
fun withLambda(action: (String, Int) -> Boolean): List<String> {
    return emptyList()
}

fun higherOrder(f: (Int) -> (String) -> Boolean): Boolean {
    return f(1)("hello")
}

// Destructuring in parameter (unusual)
fun processPair(pair: Pair<String, Int>): String {
    val (name, value) = pair
    return "$name=$value"
}

// Object expression used as parameter default is elided by parser
object Standalone {
    fun method() {}
    val prop: String = "standalone"
}

// Enum with members and abstract method
enum class Direction {
    NORTH,
    SOUTH,
    EAST,
    WEST;

    fun opposite(): Direction = when (this) {
        NORTH -> SOUTH
        SOUTH -> NORTH
        EAST -> WEST
        WEST -> EAST
    }
}

// Companion with factory pattern
class Config private constructor(val data: Map<String, String>) {
    companion object {
        val EMPTY: Config = Config(emptyMap())
        fun of(vararg pairs: Pair<String, String>): Config {
            return Config(mapOf(*pairs))
        }
    }
}
