package com.example.edge;

import java.lang.annotation.*;

// Malformed-but-parseable: annotation type declaration
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.METHOD)
public @interface Timed {
    String value() default "";
    long timeoutMs() default 5000L;
}

@interface Marker {
    String description();
}

public sealed class Shape permits Shape.Circle, Shape.Rectangle {

    public static final double PI = 3.141592653589793;

    public abstract double area();

    public final class Circle extends Shape {

        private final double radius;

        public Circle(double radius) {
            this.radius = radius;
        }

        @Override
        public double area() {
            return PI * radius * radius;
        }

        public double circumference() {
            return 2 * PI * radius;
        }
    }

    public final class Rectangle extends Shape {

        private final double width;
        private final double height;

        public Rectangle(double width, double height) {
            this.width = width;
            this.height = height;
        }

        @Override
        public double area() {
            return width * height;
        }

        public double perimeter() {
            return 2 * (width + height);
        }
    }
}

enum Planet {
    MERCURY(3.303e+23, 2.4397e6),
    VENUS(4.869e+24, 6.0518e6),
    EARTH(5.976e+24, 6.37814e6);

    private final double mass;
    private final double radius;

    Planet(double mass, double radius) {
        this.mass = mass;
        this.radius = radius;
    }

    static final double G = 6.67300E-11;

    double surfaceGravity() {
        return G * mass / (radius * radius);
    }

    double surfaceWeight(double otherMass) {
        return otherMass * surfaceGravity();
    }
}

record Point(int x, int y) implements Comparable<Point> {

    public static final Point ORIGIN = new Point(0, 0);

    public Point {
        if (x < 0 || y < 0) throw new IllegalArgumentException("negative coords");
    }

    public double distanceTo(Point other) {
        int dx = this.x - other.x;
        int dy = this.y - other.y;
        return Math.sqrt(dx * dx + dy * dy);
    }

    @Override
    public int compareTo(Point other) {
        int cmp = Integer.compare(this.x, other.x);
        return cmp != 0 ? cmp : Integer.compare(this.y, other.y);
    }
}
