package com.example.ignored

// This file tests that declarations inside comments and strings are ignored.

/**
 * class FakeInComment {
 *   def shouldBeIgnored(): Int = 42
 * }
 * trait AlsoIgnored extends FakeInComment
 */
trait RealTrait {
  def realMethod(): String
}

// def notAMethod(): Boolean = true
// val notAVal: Int = 0

object StringLiterals {
  val greeting: String = "def notReal(): Int = 0"
  val template: String = "class Fake { val x = 1 }"

  // Interpolated strings with fake declarations
  val name: String = "world"
  val msg: String = s"trait FakeInInterp { def nope(): Unit }"

  val multiLineIgnored: String = """
    object InTripleQuote {
      val hidden = 42
      def alsoHidden(): Boolean = false
    }
    case class NotReal(x: Int)
  """

  def realDef(x: Int): Int = x + 1
}

class BlockCommentHolder {
  /*
   * object InsideBlockComment {
   *   def hiddenMethod(): String = "nope"
   * }
   */
  val realVal: Double = 3.14

  /* trait FakeBlockTrait { def fake(): Unit } */

  def realMethod(): Boolean = true

  val anotherReal: String = {
    // val fakeInsideBlock: Int = 99
    "result"
  }
}

object NestedStringEdgeCases {
  val tricky: String = "val x = \"nested quote\""
  val raw: String = """class Raw { def m(): Unit = () }"""

  // s"object FakeInLineComment { }"
  val legitimate: Int = 42

  def compute(s: String): Int = s.length
}
