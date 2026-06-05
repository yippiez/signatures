package com.example.generics

const val DEFAULT_CAPACITY = 16

interface Container<T> {
    fun add(item: T): Boolean
    fun remove(item: T): Boolean
    fun contains(item: T): Boolean
    fun size(): Int
    fun isEmpty(): Boolean
}

interface Transformer<in A, out B> {
    fun transform(input: A): B
}

interface Comparable2<T> {
    operator fun compareTo(other: T): Int
}

data class Pair2<A, B>(val first: A, val second: B) {
    fun swap(): Pair2<B, A> = Pair2(second, first)
    fun <C> mapFirst(f: (A) -> C): Pair2<C, B> = Pair2(f(first), second)
}

class BoundedList<T : Comparable<T>>(private val capacity: Int = DEFAULT_CAPACITY) : Container<T> {
    private val items = mutableListOf<T>()

    override fun add(item: T): Boolean {
        if (items.size >= capacity) return false
        return items.add(item)
    }

    override fun remove(item: T): Boolean = items.remove(item)

    override fun contains(item: T): Boolean = item in items

    override fun size(): Int = items.size

    override fun isEmpty(): Boolean = items.isEmpty()

    fun min(): T? = items.minOrNull()
    fun max(): T? = items.maxOrNull()

    fun <R : Comparable<R>> sortedBy(selector: (T) -> R): List<T> {
        return items.sortedBy(selector)
    }
}

sealed class Result<out T, out E> {
    data class Ok<T>(val value: T) : Result<T, Nothing>()
    data class Err<E>(val error: E) : Result<Nothing, E>()

    fun isOk(): Boolean = this is Ok
    fun isErr(): Boolean = this is Err

    fun <U> map(transform: (T) -> U): Result<U, E> {
        return when (this) {
            is Ok -> Ok(transform(value))
            is Err -> this
        }
    }
}

inline fun <T, reified R> List<T>.filterIsInstanceAndMap(transform: (R) -> R): List<R> {
    return filterIsInstance<R>().map(transform)
}

fun <T> List<T>.second(): T? = if (size >= 2) get(1) else null

fun <K, V> Map<K, V>.getOrThrow(key: K): V {
    return get(key) ?: throw NoSuchElementException("Key $key not found")
}

fun <T : Any> T?.orThrow(message: String = "Expected non-null value"): T {
    return this ?: throw NullPointerException(message)
}

fun <A, B, C> compose(f: (B) -> C, g: (A) -> B): (A) -> C {
    return { a -> f(g(a)) }
}

object TypeUtils {
    inline fun <reified T> isType(value: Any): Boolean = value is T
    fun <T> identity(value: T): T = value
    fun <T, R> List<T>.mapNotNullTo(transform: (T) -> R?): List<R> {
        return mapNotNull(transform)
    }
}
