// This file contains fake declarations inside comments and strings that
// must NOT appear in the signatures output.

// class FakeInLineComment { void shouldNotAppear() {} }
/* class FakeInBlockComment {
   int alsoFake(int x) => x;
}
*/

/**
 * class FakeInJavadocStyle {
 *   final String nope = '';
 * }
 */

/// Real top-level constant.
const String kAppName = 'MyApp';

/// Real class — its doc comment contains a fake declaration.
///
/// Example usage:
/// ```dart
/// class FakeInDocComment { void fake() {} }
/// final bad = FakeInDocComment();
/// ```
class Documented {
  /// This field is real.
  final int id;

  // Not a declaration: class Phantom { }
  Documented(this.id);

  /// Returns a string. See also: `void fakeMethod() {}` (not real).
  String label() => 'doc-$id';

  /*
   * Block comment inside class:
   * abstract class AlsoFake extends Object {
   *   int notReal();
   * }
   */
  void realMethod() {}
}

// Strings with embedded fake declarations.
const String raw1 = r'class InRawString { const x = 1; }';
const String raw2 = r"void inRawString(int a, int b) => a + b;";
const String interp = 'class InPlainString { void fake() {} }';
const String multiSingle = 'The pattern '
    'class Fake {} '
    'spans lines but is just a string';

/// Triple-quoted strings should also be masked.
const String tripleDouble = """
class FakeInTripleDouble {
  int shouldNotAppear() => 42;
}
""";

const String tripleSingle = '''
abstract class FakeInTripleSingle {
  void alsoFake();
}
''';

/// Real function after all the comment/string noise.
int realAdd(int a, int b) => a + b;

/// Real class with getters and setters.
class Settings {
  int _timeout = 30;
  bool _debug = false;

  int get timeout => _timeout;
  set timeout(int value) {
    _timeout = value.clamp(1, 300);
  }

  bool get debug => _debug;
  set debug(bool value) => _debug = value;

  // This comment has a fake: void notReal() {}
  String get summary => 'timeout=$_timeout debug=$_debug';
}
