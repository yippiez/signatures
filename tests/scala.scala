@@CASE@@ comments_strings
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
@@CASE@@ edge
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
@@CASE@@ generics
package com.example.generics

import scala.collection.mutable

// Generics with type bounds, variance, context bounds, given/implicit

trait Functor[F[_]] {
  def map[A, B](fa: F[A])(f: A => B): F[B]
}

trait Monad[F[_]] extends Functor[F] {
  def pure[A](a: A): F[A]
  def flatMap[A, B](fa: F[A])(f: A => F[B]): F[B]
}

trait Ordering[A] {
  def compare(x: A, y: A): Int
  def lt(x: A, y: A): Boolean = compare(x, y) < 0
  def gt(x: A, y: A): Boolean = compare(x, y) > 0
}

class SortedList[A: Ordering] {
  private val underlying: mutable.ArrayBuffer[A] = mutable.ArrayBuffer.empty

  def add(elem: A): Unit = ()
  def toList: List[A] = underlying.toList
  def min: Option[A] = underlying.headOption
  def max: Option[A] = underlying.lastOption
}

trait Codec[A] {
  def encode(a: A): String
  def decode(s: String): Either[String, A]
}

object Codec {
  def apply[A](using c: Codec[A]): Codec[A] = c

  given Codec[Int] with {
    def encode(a: Int): String = a.toString
    def decode(s: String): Either[String, Int] = s.toIntOption.toRight(s"Not an int: $s")
  }

  given Codec[Boolean] with {
    def encode(a: Boolean): String = a.toString
    def decode(s: String): Either[String, Boolean] =
      s.toBooleanOption.toRight(s"Not a boolean: $s")
  }
}

class Pair[+A, +B](val first: A, val second: B) {
  def swap: Pair[B, A] = Pair(second, first)
  def mapFirst[C](f: A => C): Pair[C, B] = Pair(f(first), second)
  def mapSecond[C](f: B => C): Pair[A, C] = Pair(first, f(second))
}

trait Bounded[A] {
  def minValue: A
  def maxValue: A
}

object Bounded {
  given Bounded[Int] with {
    def minValue: Int = Int.MinValue
    def maxValue: Int = Int.MaxValue
  }

  given Bounded[Double] with {
    def minValue: Double = Double.MinValue
    def maxValue: Double = Double.MaxValue
  }

  def clamp[A: Bounded: Ordering](value: A): A = value
}

def identity[A](a: A): A = a
def const[A, B](a: A)(b: B): A = a
def compose[A, B, C](f: B => C)(g: A => B): A => C = a => f(g(a))

class Container[A <: AnyRef](private var value: A) {
  def get: A = value
  def set(newValue: A): Unit = value = newValue
  def transform[B <: AnyRef](f: A => B): Container[B] = Container(f(value))
}
@@CASE@@ nested
package com.example.nested

// Deeply nested objects, classes and traits

object Outer {
  val outerVal: String = "outer"

  class Inner {
    val innerVal: Int = 42

    object DeepInner {
      val deepVal: Boolean = true

      def deepMethod(x: Int): Int = x * 2

      class Deepest {
        def hello(): String = "deepest"
      }
    }

    def innerMethod(): Unit = ()
  }

  trait InnerTrait {
    def abstractMethod(): String
    def concreteMethod(): Int = 0
  }

  object InnerObject extends InnerTrait {
    val name: String = "inner-object"
    def abstractMethod(): String = name
    def extra(): Boolean = true
  }

  def outerMethod(i: Inner): String = i.innerMethod().toString
}

class Outer2 {
  class Level1 {
    class Level2 {
      def method(): Int = 99
      val constant: Long = 0L
    }

    val l2Instance: Level2 = new Level2()
    def level1Method(): String = "L1"
  }

  trait Level1Trait {
    trait Nested {
      def nestedAbstract(): Double
    }
    def parentMethod(): Unit
  }

  def makeLevel1(): Level1 = new Level1()
}

object Config {
  object Database {
    val host: String = "localhost"
    val port: Int = 5432

    object Pool {
      val maxConnections: Int = 10
      val timeout: Long = 5000L
      def isHealthy(): Boolean = true
    }
  }

  object Server {
    val host: String = "0.0.0.0"
    val port: Int = 8080
    def bindAddress(): String = s"${host}:${port}"
  }
}
@@CASE@@ realworld
package com.example.store

import scala.concurrent.Future
import scala.concurrent.ExecutionContext

// A realistic e-commerce domain model

trait Repository[A, ID] {
  def findById(id: ID): Future[Option[A]]
  def findAll(): Future[List[A]]
  def save(entity: A): Future[A]
  def delete(id: ID): Future[Boolean]
}

case class ProductId(value: Long)
case class UserId(value: Long)

case class Product(
  id: ProductId,
  name: String,
  price: BigDecimal,
  stock: Int
)

case class User(
  id: UserId,
  email: String,
  name: String
)

case class OrderLine(product: Product, quantity: Int) {
  def subtotal: BigDecimal = product.price * quantity
}

case class Order(id: Long, user: User, lines: List[OrderLine]) {
  def total: BigDecimal = lines.map(_.subtotal).sum
  def itemCount: Int = lines.map(_.quantity).sum
}

trait ProductRepository extends Repository[Product, ProductId] {
  def findByName(name: String): Future[List[Product]]
  def findInStock(): Future[List[Product]]
  def updateStock(id: ProductId, delta: Int): Future[Boolean]
}

trait OrderRepository extends Repository[Order, Long] {
  def findByUser(userId: UserId): Future[List[Order]]
  def findRecent(limit: Int): Future[List[Order]]
}

class OrderService(
  orders: OrderRepository,
  products: ProductRepository
)(using ec: ExecutionContext) {
  val MaxOrderLines: Int = 50
  val MinimumOrderAmount: BigDecimal = BigDecimal("0.01")

  def placeOrder(user: User, lines: List[OrderLine]): Future[Order] =
    Future.failed(new NotImplementedError)

  def cancelOrder(orderId: Long): Future[Boolean] =
    Future.failed(new NotImplementedError)

  def getOrderSummary(userId: UserId): Future[String] =
    Future.failed(new NotImplementedError)

  private def validateLines(lines: List[OrderLine]): Boolean =
    lines.nonEmpty && lines.size <= MaxOrderLines
}

object OrderService {
  val ServiceVersion: String = "1.0.0"

  def apply(
    orders: OrderRepository,
    products: ProductRepository
  )(using ec: ExecutionContext): OrderService =
    new OrderService(orders, products)
}
@@CASE@@ sample
package app

object Constants {
  val Max = 100
}

class Greeter(name: String) {
  def greet(): String = s"hi $name"
}

trait Shape {
  def area: Double
}

case class Point(x: Int, y: Int)
@@CASE@@ sealed
package com.example.sealed

// Sealed hierarchies, enums, traits, case classes, objects

sealed trait Expr {
  def eval: Double
  def stringify: String
}

case class Num(value: Double) extends Expr {
  def eval: Double = value
  def stringify: String = value.toString
}

case class Add(left: Expr, right: Expr) extends Expr {
  def eval: Double = left.eval + right.eval
  def stringify: String = s"(${left.stringify} + ${right.stringify})"
}

case class Mul(left: Expr, right: Expr) extends Expr {
  def eval: Double = left.eval * right.eval
  def stringify: String = s"(${left.stringify} * ${right.stringify})"
}

case class Neg(expr: Expr) extends Expr {
  def eval: Double = -expr.eval
  def stringify: String = s"(-${expr.stringify})"
}

object Expr {
  val Zero: Expr = Num(0.0)
  val One: Expr = Num(1.0)

  def simplify(e: Expr): Expr = e match {
    case Add(Num(0.0), r) => r
    case Add(l, Num(0.0)) => l
    case Mul(Num(1.0), r) => r
    case Mul(l, Num(1.0)) => l
    case Neg(Neg(inner)) => inner
    case other           => other
  }
}

enum Color {
  case Red
  case Green
  case Blue
  case Custom(r: Int, g: Int, b: Int)

  def toHex: String = this match {
    case Red            => "#FF0000"
    case Green          => "#00FF00"
    case Blue           => "#0000FF"
    case Custom(r,g,b)  => f"#$r%02X$g%02X$b%02X"
  }
}

enum Shape(val name: String) {
  case Circle(radius: Double) extends Shape("circle")
  case Rectangle(width: Double, height: Double) extends Shape("rectangle")
  case Triangle(base: Double, height: Double) extends Shape("triangle")

  def area: Double = this match {
    case Circle(r)        => math.Pi * r * r
    case Rectangle(w, h)  => w * h
    case Triangle(b, h)   => 0.5 * b * h
  }
}

sealed abstract class Result[+A] {
  def isOk: Boolean
  def getOrElse[B >: A](default: B): B
}

final case class Ok[+A](value: A) extends Result[A] {
  def isOk: Boolean = true
  def getOrElse[B >: A](default: B): B = value
}

final case class Err(message: String) extends Result[Nothing] {
  def isOk: Boolean = false
  def getOrElse[B](default: B): B = default
}

object Result {
  def fromOption[A](opt: Option[A], msg: String): Result[A] = opt match {
    case Some(v) => Ok(v)
    case None    => Err(msg)
  }

  def sequence[A](results: List[Result[A]]): Result[List[A]] =
    results.foldRight(Ok(List.empty[A]): Result[List[A]]) {
      case (Ok(a), Ok(acc)) => Ok(a :: acc)
      case (Err(e), _)      => Err(e)
      case (_, err)         => err
    }
}
@@CASE@@ unicode
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
@@CASE@@ enum_and_function_types
enum Color {
  case Red, Green, Blue
  def isWarm: Boolean = this == Red
}
trait Funcs {
  def adder: Int => Int
  def higher(f: Int => Int): Int => Int
}
def topLevel(): Int = 42
@@CASE@@ operator_method_and_access_qualifier
class Ord {
  def <(other: Ord): Boolean = true
  def lostMethod(): String = "here"
  private[this] val secret: Int = 1
  protected[pkg] def helper(): Int = 2
}
@@CASE@@ val_function_type_annotation
// Bug: function-type annotation in val truncated at =>
object Funcs {
  val f1: String => Int = _.length
  val f2: (Int, Int) => Boolean = _ < _
  val f3: Int => String = _.toString
}
@@CASE@@ abstract_val
// Bug: abstract val shown with = … despite having no initializer
trait Abs {
  val x: Int
  val y: String
  def z: Boolean
}
@@CASE@@ backtick_quoted_val
// Bug: backtick-quoted keyword val declarations silently dropped
class Bt {
  val `class`: String = "x"
  val `val`: Int = 1
  val normal: Int = 2
}
@@CASE@@ opaque_type
// Bug: opaque type declarations silently dropped
object O {
  opaque type Meters = Double
  opaque type Kilograms <: Double = Double
  type Normal = Int
}
@@CASE@@ transparent_modifier
// Bug: transparent modifier causes def to be dropped
class C {
  transparent inline def f(): Int = 1
  transparent def g(): Int = 2
  inline def h(): Int = 3
  def i(): Int = 4
}
@@CASE@@ multiline_extends_with
// Bug: multi-line extends/with continuation lines dropped
class MyClass
  extends Base
  with Mixin1
  with Mixin2 {
  def method(): Int = 42
}
