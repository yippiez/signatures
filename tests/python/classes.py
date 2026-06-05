"""Classes, methods, inheritance, and nested classes."""

from abc import ABC


class Animal(ABC):
    LEGS = 4

    def __init__(self, name):
        self.name = name

    def speak(self):
        raise NotImplementedError


class Dog(Animal):
    SOUND = "woof"

    def speak(self):
        return self.SOUND

    class Collar:
        COLOR = "red"

        def size(self):
            return 1


class Empty:
    pass
