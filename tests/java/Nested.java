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
