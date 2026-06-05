package com.example.nested

const val ROOT_VERSION = 1

class Outer(val name: String) {

    val outerProp: Int = 42

    fun outerMethod(): String {
        return name
    }

    inner class Inner(val value: Double) {
        fun innerMethod(): Double {
            return value * outerProp
        }

        object InnerSingleton {
            const val MAGIC = "inner-magic"
            fun create(): InnerSingleton = InnerSingleton
        }
    }

    class StaticNested {
        val count: Int = 0

        fun increment(): Int {
            return count + 1
        }

        class DeepNested {
            fun deepMethod() {}

            interface DeepInterface {
                fun deepAbstract(): Boolean
            }
        }
    }

    companion object {
        const val TAG = "Outer"
        var instanceCount: Int = 0

        fun newInstance(name: String): Outer {
            instanceCount++
            return Outer(name)
        }
    }

    interface Callback {
        fun onSuccess(result: String)
        fun onError(error: Throwable)
        fun onComplete()
    }

    enum class State {
        IDLE,
        LOADING,
        SUCCESS,
        ERROR,
    }

    sealed class Event {
        data class Success(val data: String) : Event()
        data class Failure(val error: Throwable) : Event()
        object Loading : Event()
    }
}

object TopLevelSingleton {
    val value: String = "singleton"

    fun getValue(): String {
        return value
    }

    object Nested {
        fun nestedFun() {}
    }
}
