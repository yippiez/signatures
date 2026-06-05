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
