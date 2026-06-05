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
