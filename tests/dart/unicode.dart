// Non-ASCII (Unicode) identifiers in Dart — fully valid per the language spec.
// Tests that the signature extractor handles multi-byte characters safely.

/// Top-level constant with a Unicode name.
const double π = 3.141592653589793;
const double τ = 2 * π;
const String kÄpfel = 'apples';

/// Class with non-ASCII name: Geometrie (German for Geometry).
class Geometrie {
  final double breite;   // width
  final double höhe;     // height

  const Geometrie({required this.breite, required this.höhe});

  double get fläche => breite * höhe;    // area
  double get umfang => 2 * (breite + höhe);  // perimeter
}

/// Circle using π as a field name.
class Kreis {
  static const double π = 3.141592653589793;
  final double radius;

  const Kreis(this.radius);

  double berechneFlaeche() => π * radius * radius;
  double berechneUmfang() => 2 * π * radius;
}

/// Abstract class using Japanese-inspired names.
abstract class 図形 {
  double get 面積;       // area
  double get 周囲長;     // perimeter
  bool 含む(double x, double y);   // contains
}

/// Spanish-influenced names.
class Configuración {
  final String nombre;
  final int máximo;
  final int mínimo;

  const Configuración({
    required this.nombre,
    this.máximo = 100,
    this.mínimo = 0,
  });

  bool esVálido(int valor) {
    return valor >= mínimo && valor <= máximo;
  }
}

/// Mixin with accented identifiers.
mixin Étiquetable {
  String get étiquette;
  String get description => 'Étiquette: $étiquette';
}

/// Enum with emoji-adjacent name and accented members.
enum Saison {
  printemps,
  été,
  automne,
  hiver;

  String get nomComplet {
    switch (this) {
      case Saison.printemps: return 'Printemps';
      case Saison.été: return 'Été';
      case Saison.automne: return 'Automne';
      case Saison.hiver: return 'Hiver';
    }
  }
}

/// Function with non-ASCII parameter names.
double calculer(double largeur, double hauteur) => largeur * hauteur;

int comparer(Comparable<dynamic> α, Comparable<dynamic> β) => α.compareTo(β);
