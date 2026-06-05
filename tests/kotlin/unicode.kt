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
