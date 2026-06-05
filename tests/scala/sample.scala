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
