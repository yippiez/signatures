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
