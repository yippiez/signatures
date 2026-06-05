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
