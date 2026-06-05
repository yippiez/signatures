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
