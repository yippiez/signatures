package com.example.unicode;

/**
 * Demonstrates non-ASCII identifiers and string content in Java.
 */
public class Unicode {

    public static final String LANGUAGE = "日本語";

    private String données;

    public Unicode(String données) {
        this.données = données;
    }

    public String getDonnées() {
        return données;
    }

    public void setDonnées(String données) {
        this.données = données;
    }

    public int longueur() {
        return données.length();
    }
}

interface Beschreibbar {
    String beschreiben();
}

enum Richtung {
    NORD,
    SÜD,
    OST,
    WEST;

    public boolean istVertical() {
        return this == NORD || this == SÜD;
    }
}

record Koordinate(double breite, double länge) {
    public double distanzZum(Koordinate andere) {
        double db = this.breite - andere.breite;
        double dl = this.länge - andere.länge;
        return Math.sqrt(db * db + dl * dl);
    }
}
