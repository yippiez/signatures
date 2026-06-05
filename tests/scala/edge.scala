package com.example.edge

// Edge cases: lazy val, given/using, implicit, type aliases, abstract types,
// vals with complex types, multi-param lists, malformed-but-parseable constructs

trait HasLogger {
  lazy val logger: String = "default-logger"
  def log(msg: String): Unit
}

object LazyVals {
  lazy val expensiveComputation: Int = {
    // pretend this is expensive
    42
  }
  lazy val cachedList: List[String] = List("a", "b", "c")
  val eagerVal: Double = 2.718
  lazy val anotherLazy: Boolean = true

  def useLazy(): Int = expensiveComputation + 1
}

trait TypeMembers {
  type Element
  type Container[A]
  type Mapping = Map[String, Element]

  def createElement(): Element
  def wrap[A](a: A): Container[A]
}

// implicit-style (Scala 2 compatible syntax)
class LegacyImplicits {
  implicit val defaultTimeout: Int = 30

  implicit def stringToInt(s: String): Int = s.length

  def compute(x: Int)(implicit timeout: Int): String = s"$x in $timeout"
  def process[A](items: List[A])(implicit ord: Ordering[A]): List[A] = items.sorted
}

// given/using style (Scala 3)
object GivenUsing {
  given defaultName: String = "anonymous"
  given Int = 0

  trait Show[A] {
    def show(a: A): String
  }

  given Show[Int] with {
    def show(a: Int): String = a.toString
  }

  given [A: Show]: Show[List[A]] with {
    def show(a: List[A]): String = a.map(summon[Show[A]].show).mkString("[", ",", "]")
  }

  def display[A](a: A)(using s: Show[A]): String = s.show(a)
  def displayAll[A: Show](items: List[A]): String = items.map(display(_)).mkString(", ")
}

// Multiple parameter lists and currying
class MultiParam {
  def threeGroups(a: Int)(b: String, c: Boolean)(d: Double): String = ""
  def curriedAdd(x: Int)(y: Int)(z: Int): Int = x + y + z

  def withContext[A](value: A)(using ctx: String, n: Int): String =
    s"$ctx[$n]: $value"
}

// Type aliases and complex signatures
object TypeAliases {
  type StringMap[V] = Map[String, V]
  type Callback[A] = A => Unit
  type BiFunc[A, B, C] = (A, B) => C
  type NestedOption[A] = Option[Option[A]]

  val handlers: StringMap[Callback[Int]] = Map.empty
  lazy val registry: StringMap[List[String]] = Map.empty

  def register(key: String)(handler: Callback[Int]): StringMap[Callback[Int]] =
    handlers + (key -> handler)

  def lookup(key: String): Option[Callback[Int]] = handlers.get(key)
}

// Malformed-but-parseable: missing return type, odd spacing, Unicode ops
class Malformed {
  def noReturnType() = 42
  def   weirdSpacing   (  x : Int  )  :  String  = x.toString
  def +(other: Malformed) = new Malformed
  def unary_- = new Malformed
  val `backtick name`: Int = 7
  val `class`: String = "reserved"
}
