@@CASE@@ comments_strings
package com.example.tricky

// fun notAFunction() {}
// class NotAClass {}
// val notAConst = 0

/**
 * This is real documentation.
 * fun fakeInDoc() {}
 * class FakeClass {}
 * val fakeVal = 42
 */
const val REAL_CONST = "real"

/* val anotherFake = 999 */

class RealClass {

    // private fun hiddenMethod() {}

    val template: String = """
        fun fakeInTripleQuote() {}
        class FakeInTripleQuote
        val fakeTriple = 0
    """.trimIndent()

    val singleLine: String = "fun notReal() {}"

    /**
     * interface FakeInterface {
     *     fun fakeMethod()
     * }
     */
    fun realMethod(input: String): String {
        val msg = "class Decoy { fun trick() {} }"
        val sql = """
            SELECT * FROM fun_table
            WHERE class = 'object'
            AND val = 'const'
        """.trimIndent()
        return msg + sql
    }

    // object FakeSingleton { val x = 1 }
    fun anotherReal(): Int {
        return 42
    }
}

// sealed class FakeSealed
interface RealInterface {
    // fun commentedOut()
    fun realAbstract(): Boolean
    fun realWithDefault(): String
}

/*
 * object BigFakeBlock {
 *   fun doStuff() {}
 *   val x = 1
 * }
 */
object RealObject {
    val prop: Int = 0
    fun doReal() {}
}
@@CASE@@ edge
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
@@CASE@@ generics
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
@@CASE@@ malformed
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
@@CASE@@ nested
package com.example.nested

const val ROOT_VERSION = 1

class Outer(val name: String) {

    val outerProp: Int = 42

    fun outerMethod(): String {
        return name
    }

    inner class Inner(val value: Double) {
        fun innerMethod(): Double {
            return value * outerProp
        }

        object InnerSingleton {
            const val MAGIC = "inner-magic"
            fun create(): InnerSingleton = InnerSingleton
        }
    }

    class StaticNested {
        val count: Int = 0

        fun increment(): Int {
            return count + 1
        }

        class DeepNested {
            fun deepMethod() {}

            interface DeepInterface {
                fun deepAbstract(): Boolean
            }
        }
    }

    companion object {
        const val TAG = "Outer"
        var instanceCount: Int = 0

        fun newInstance(name: String): Outer {
            instanceCount++
            return Outer(name)
        }
    }

    interface Callback {
        fun onSuccess(result: String)
        fun onError(error: Throwable)
        fun onComplete()
    }

    enum class State {
        IDLE,
        LOADING,
        SUCCESS,
        ERROR,
    }

    sealed class Event {
        data class Success(val data: String) : Event()
        data class Failure(val error: Throwable) : Event()
        object Loading : Event()
    }
}

object TopLevelSingleton {
    val value: String = "singleton"

    fun getValue(): String {
        return value
    }

    object Nested {
        fun nestedFun() {}
    }
}
@@CASE@@ realworld
package com.example.app

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

const val API_BASE_URL = "https://api.example.com/v1"
const val DEFAULT_TIMEOUT_MS = 5000L

data class User(
    val id: Long,
    val name: String,
    val email: String,
    val role: UserRole,
)

data class PagedResult<T>(
    val items: List<T>,
    val total: Int,
    val page: Int,
    val pageSize: Int,
)

enum class UserRole {
    ADMIN,
    EDITOR,
    VIEWER,
}

interface UserRepository {
    suspend fun findById(id: Long): User?
    suspend fun findAll(page: Int, pageSize: Int): PagedResult<User>
    suspend fun save(user: User): User
    suspend fun delete(id: Long): Boolean
    fun observeChanges(): Flow<User>
}

interface AuthService {
    suspend fun login(email: String, password: String): String
    suspend fun logout(token: String)
    suspend fun validateToken(token: String): Boolean
    fun currentUser(): User?
}

class UserRepositoryImpl(
    private val db: Database,
    private val cache: Cache,
) : UserRepository {

    override suspend fun findById(id: Long): User? {
        val cached = cache.get("user:$id")
        if (cached != null) return cached as User
        return db.query("SELECT * FROM users WHERE id = ?", id)
    }

    override suspend fun findAll(page: Int, pageSize: Int): PagedResult<User> {
        val offset = page * pageSize
        val items = db.queryList("SELECT * FROM users LIMIT ? OFFSET ?", pageSize, offset)
        val total = db.queryScalar("SELECT COUNT(*) FROM users") as Int
        return PagedResult(items, total, page, pageSize)
    }

    override suspend fun save(user: User): User {
        return db.upsert(user)
    }

    override suspend fun delete(id: Long): Boolean {
        cache.evict("user:$id")
        return db.execute("DELETE FROM users WHERE id = ?", id) > 0
    }

    override fun observeChanges(): Flow<User> {
        return db.watchTable("users").map { row -> row.toUser() }
    }

    private fun Row.toUser(): User {
        return User(
            id = getLong("id"),
            name = getString("name"),
            email = getString("email"),
            role = UserRole.valueOf(getString("role")),
        )
    }
}

object AppConfig {
    val instance: AppConfig get() = this
    fun load(path: String): Map<String, String> {
        return emptyMap()
    }
    fun get(key: String, default: String = ""): String {
        return default
    }
}

fun <T> retry(times: Int, block: suspend () -> T): T {
    var lastError: Throwable? = null
    repeat(times) {
        try {
            return block()
        } catch (e: Throwable) {
            lastError = e
        }
    }
    throw lastError ?: IllegalStateException("retry failed")
}

suspend fun fetchUserProfile(id: Long, repo: UserRepository): User {
    return repo.findById(id) ?: throw NoSuchElementException("User $id not found")
}
@@CASE@@ sample
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
@@CASE@@ unicode
package com.example.unicode

const val ПРИВЕТСТВИЕ = "Привет, мир!"
const val 最大値 = 100
const val GREETING_AR = "مرحبا"

data class Точка(val х: Double, val у: Double)

data class 座標(val x: Double, val y: Double)

interface Вычисляемый {
    fun вычислить(): Double
    fun описание(): String
}

class Геометрия : Вычисляемый {
    private val π = 3.14159265358979

    fun площадьКруга(радиус: Double): Double {
        return π * радиус * радиус
    }

    fun длинаОкружности(радиус: Double): Double {
        return 2 * π * радиус
    }

    override fun вычислить(): Double {
        return π
    }

    override fun описание(): String {
        return "Геометрия"
    }
}

object 数学ユーティリティ {
    const val 円周率 = 3.14159

    fun 面積を計算(半径: Double): Double {
        return 円周率 * 半径 * 半径
    }

    fun 距離を計算(点1: Точка, 点2: Точка): Double {
        val дх = 点1.х - 点2.х
        val ду = 点1.у - 点2.у
        return Math.sqrt(дх * дх + ду * ду)
    }
}

fun сложить(а: Int, б: Int): Int {
    return а + б
}

fun `fun with spaces`(input: String): String {
    return input.reversed()
}
@@CASE@@ unicode_const
const val ÜBER_LIMIT = 100
const val 最大値 = 9
const val MAX = 10

object Math {
    const val π = 3.14159
}
@@CASE@@ typealias_decls
typealias StringList = List<String>
typealias Transform<T, R> = (T) -> R
fun use(s: StringList): Int = s.size
@@CASE@@ init_block_locals
class Foo {
    init {
        val x: Int = 42
        var y: String = "hi"
    }
    fun bar() {}
}
@@CASE@@ property_custom_getter
class Foo {
    val fullName: String
        get() = "hello"
    val normal: Int = 5
}
@@CASE@@ where_clause_same_line_brace
class A<T>
    where T : Any {
    fun foo(): T? = null
}
class B {
    fun bar(): Int = 1
}
@@CASE@@ secondary_constructors_and_generic_ext_property
class Foo(val x: Int) {
    constructor() : this(0) {}
    constructor(s: String) : this(s.length) {}
    fun bar(): Int = x
}
val <T> List<T>.head: T? get() = firstOrNull()
@@CASE@@ fun_multiline_type_params
fun <A,
     B> pair(a: A, b: B): Pair<A, B> = Pair(a, b)
val x = 1
@@CASE@@ val_backtick_identifiers
val `val` = 42
val `interface` = "reserved"
val `my property` = "hello"
val normal = 99
@@CASE@@ getter_body_locals_suppressed
class C {
    val p: Int get() {
        var x = 1
        return x
    }
    fun m(): Int {
        var y = 2
        return y
    }
}
