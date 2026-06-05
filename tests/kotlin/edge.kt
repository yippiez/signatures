package com.example.edge

// Edge cases: unusual but valid Kotlin

const val EMPTY_STRING = ""
const val NEGATIVE = -1
const val MAX_INT = Int.MAX_VALUE

// Single-expression functions
fun square(n: Int): Int = n * n
fun cube(n: Int): Int = n * n * n
fun identity(x: String): String = x

// Extension functions on nullable receivers
fun String?.orEmpty2(): String = this ?: ""
fun Int?.orZero(): Int = this ?: 0

// Operator overloading
data class Vector(val x: Double, val y: Double) {
    operator fun plus(other: Vector): Vector = Vector(x + other.x, y + other.y)
    operator fun minus(other: Vector): Vector = Vector(x - other.x, y - other.y)
    operator fun times(scalar: Double): Vector = Vector(x * scalar, y * scalar)
    operator fun unaryMinus(): Vector = Vector(-x, -y)
    operator fun get(index: Int): Double = when (index) {
        0 -> x
        1 -> y
        else -> throw IndexOutOfBoundsException(index)
    }
}

// Infix functions
infix fun Int.until2(to: Int): IntRange = this until to
infix fun String.appendWith(separator: String): String.(String) -> String = { other -> this + separator + other }

// Annotation class
annotation class Retry(val times: Int = 3, val delay: Long = 100L)

// Value class
value class UserId(val value: Long)
value class Email(val address: String)

// Abstract class with abstract and concrete members
abstract class AbstractProcessor<T, R> {
    abstract fun process(input: T): R
    abstract val name: String
    open fun validate(input: T): Boolean = true
    fun run(input: T): R? {
        return if (validate(input)) process(input) else null
    }
}

// Object with invoke
object Factory {
    operator fun invoke(type: String): Any = when (type) {
        "A" -> object {}
        else -> object {}
    }
    fun create(name: String): String = name
}

// Tailrec function
tailrec fun factorial(n: Long, acc: Long = 1L): Long {
    return if (n <= 1L) acc else factorial(n - 1L, acc * n)
}

// Function with vararg
fun <T> listOfNonNull(vararg items: T?): List<T> {
    return items.filterNotNull()
}

// suspend + inline
inline fun <T> measure(block: () -> T): Pair<T, Long> {
    val start = System.nanoTime()
    val result = block()
    return result to (System.nanoTime() - start)
}

suspend fun delayedEcho(message: String, delayMs: Long): String {
    return message
}
