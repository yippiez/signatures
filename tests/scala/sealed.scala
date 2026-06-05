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
