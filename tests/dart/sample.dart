const int max = 100;
final String name = "app";

class Greeter {
  final String name;
  Greeter(this.name);

  String greet() => "hi $name";
  Future<void> load() async {}
}

abstract class Shape {
  double area();
}

int add(int a, int b) => a + b;
