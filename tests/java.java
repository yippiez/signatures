@@CASE@@ CommentsStrings
package com.example.comments;

// class FakeInComment { void fakeMethod() {} }

/**
 * Javadoc with fake code:
 *   interface Hidden { void nope(); }
 *   static final int IGNORE = 99;
 */
public class CommentsStrings {

    public static final String GREETING = "Hello, World!";

    // private int notAField = 42;

    private String template;

    public CommentsStrings() {
        // class Inner {} -- this is in a comment, ignore it
        this.template = "class Fake { static final int X = 1; }";
    }

    public String buildQuery(String table) {
        /* interface InBlock { void m(); } */
        String query = "SELECT * FROM " + table + " WHERE class = 'active'";
        return query;
    }

    public String getTextBlock() {
        String block = """
                class InsideTextBlock {
                    static final int FAKE = 100;
                    void fakeMethod() {}
                }
                """;
        return block;
    }

    public int compute(int x) {
        String s = "enum Direction { NORTH, SOUTH }";
        /* enum Hidden { A, B } */
        // record FakeRecord(int a) {}
        return x * 2;
    }

    /*
     * block comment with:
     * public class MultiLineHidden {
     *     public void hiddenMethod() {}
     * }
     */
    public void realMethod() {
        String fake = "public record Ghost(int id) {}";
        System.out.println(fake);
    }
}

/* interface AlsoHidden { void x(); } */
@@CASE@@ Edge
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
@@CASE@@ Generics
package com.example.generics;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;

public class Generics {

    public static final int DEFAULT_CAPACITY = 16;

    public static <T> Optional<T> findFirst(List<T> items, java.util.function.Predicate<T> pred) {
        for (T item : items) {
            if (pred.test(item)) {
                return Optional.of(item);
            }
        }
        return Optional.empty();
    }

    public static <K, V> Map<V, K> invertMap(Map<K, V> original) {
        Map<V, K> result = new java.util.HashMap<>();
        for (Map.Entry<K, V> entry : original.entrySet()) {
            result.put(entry.getValue(), entry.getKey());
        }
        return result;
    }

    public static <T extends Comparable<T>> T max(T a, T b) {
        return a.compareTo(b) >= 0 ? a : b;
    }

    public static <A, B, C> Function<A, C> compose(Function<A, B> f, Function<B, C> g) {
        return x -> g.apply(f.apply(x));
    }
}

class Pair<A, B> {

    public static final String TYPE_NAME = "Pair";

    private final A first;
    private final B second;

    public Pair(A first, B second) {
        this.first = first;
        this.second = second;
    }

    public A getFirst() {
        return first;
    }

    public B getSecond() {
        return second;
    }

    public <C> Pair<C, B> mapFirst(Function<A, C> f) {
        return new Pair<>(f.apply(first), second);
    }

    public Pair<B, A> swap() {
        return new Pair<>(second, first);
    }

    @Override
    public String toString() {
        return "(" + first + ", " + second + ")";
    }
}

interface Repository<T, ID> {
    Optional<T> findById(ID id);
    List<T> findAll();
    T save(T entity);
    void deleteById(ID id);
}

record Result<T>(T value, String error) {
    public boolean isSuccess() {
        return error == null;
    }

    public static <T> Result<T> ok(T value) {
        return new Result<>(value, null);
    }

    public static <T> Result<T> fail(String error) {
        return new Result<>(null, error);
    }
}
@@CASE@@ Nested
package com.example.nested;

public class Outer {

    public static final int VERSION = 3;

    private int outerField;

    public Outer(int outerField) {
        this.outerField = outerField;
    }

    public int getOuterField() {
        return outerField;
    }

    public static class StaticNested {

        private String name;

        public StaticNested(String name) {
            this.name = name;
        }

        public String getName() {
            return name;
        }

        public class DeepInner {

            public void greet() {
                System.out.println("Hello from " + name);
            }
        }
    }

    public class Inner {

        public static final String LABEL = "inner";

        private int value;

        public Inner(int value) {
            this.value = value;
        }

        public int combined() {
            return outerField + value;
        }
    }

    public interface Callback {
        void onEvent(String event);
        default void onError(String msg) {
            System.err.println(msg);
        }
    }

    public enum Priority {
        LOW,
        MEDIUM,
        HIGH;

        public boolean isUrgent() {
            return this == HIGH;
        }
    }

    public static Outer create(int v) {
        return new Outer(v);
    }
}

interface TopLevel {
    void execute();
}
@@CASE@@ RealWorld
package com.example.bank;

import java.util.List;
import java.util.ArrayList;

/**
 * Represents a bank account with deposit, withdrawal, and transfer support.
 * Usage: new BankAccount("ACC-001", 1000.0)
 */
public class BankAccount {

    public static final String CURRENCY = "USD";
    public static final double MIN_BALANCE = 0.0;
    private static final int MAX_TRANSACTIONS = 500;

    private final String accountId;
    private double balance;
    private final List<String> log;

    public BankAccount(String accountId, double initialBalance) {
        this.accountId = accountId;
        this.balance = initialBalance;
        this.log = new ArrayList<>();
    }

    public String getAccountId() {
        return accountId;
    }

    public double getBalance() {
        return balance;
    }

    public void deposit(double amount) {
        if (amount <= 0) {
            throw new IllegalArgumentException("Amount must be positive");
        }
        balance += amount;
        log.add("DEPOSIT " + amount);
    }

    public boolean withdraw(double amount) {
        if (amount <= 0 || balance - amount < MIN_BALANCE) {
            return false;
        }
        balance -= amount;
        log.add("WITHDRAW " + amount);
        return true;
    }

    public boolean transfer(BankAccount target, double amount) {
        if (withdraw(amount)) {
            target.deposit(amount);
            return true;
        }
        return false;
    }

    public List<String> getTransactionLog() {
        return new ArrayList<>(log);
    }

    @Override
    public String toString() {
        return "BankAccount[" + accountId + ", balance=" + balance + "]";
    }
}

interface Auditable {
    List<String> getTransactionLog();
    String getAccountId();
}

enum AccountType {
    CHECKING,
    SAVINGS,
    INVESTMENT
}
@@CASE@@ Unicode
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
@@CASE@@ sample
package app;

public class Account {
    static final double RATE = 0.05;
    private double balance;

    public Account(String id) {
        this.balance = 0;
    }

    public double balance() {
        return balance;
    }
}

interface Greeter {
    String greet();
}

enum Status { OPEN, CLOSED }
@@CASE@@ keyword_like_method_names
public class Factory {
    public static Factory of(String name) { return null; }
    public Factory with(String key) { return this; }
    public void in(int x) {}
    public void process() {}
}
@@CASE@@ annotation_default_array
@interface Ann {
    String[] tags() default {};
    String[] more() default {"one", "two"};
    int level() default 3;
}
