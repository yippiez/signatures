package com.example.unicode

// Non-ASCII identifiers: Greek letters, subscripts, math symbols, emoji-free CJK

trait 形状 {
  def 面積(): Double
  def 周囲(): Double
}

class 円(半径: Double) extends 形状 {
  val π: Double = math.Pi
  def 面積(): Double = π * 半径 * 半径
  def 周囲(): Double = 2 * π * 半径
  def 直径(): Double = 2 * 半径
}

case class 点(x座標: Double, y座標: Double) {
  def 距離(other: 点): Double =
    math.sqrt(math.pow(x座標 - other.x座標, 2) + math.pow(y座標 - other.y座標, 2))
}

object 数学定数 {
  val π: Double = math.Pi
  val e: Double = math.E
  val φ: Double = (1.0 + math.sqrt(5.0)) / 2.0
  val √2: Double = math.sqrt(2.0)

  def 絶対値(x: Double): Double = math.abs(x)
  def 平方根(x: Double): Double = math.sqrt(x)
}

trait Ψ[A] {
  def ψ(a: A): A
  def φ(a: A, b: A): A
}

class Ω(val λ: Int) {
  val μ: Double = λ.toDouble / 100.0
  def Δ(x: Int): Int = x + λ
  def ∑(xs: List[Int]): Int = xs.sum
}

object Σ {
  val α: String = "alpha"
  val β: String = "beta"
  val γ: String = "gamma"

  def Φ(n: Int): Long = (1L to n).product
}
