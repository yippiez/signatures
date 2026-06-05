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
