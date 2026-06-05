<?php

const MAX = 100;

class Account {
    public const RATE = 0.05;
    private $balance = 0;

    public function __construct($id) {}

    public function balance() {
        return $this->balance;
    }
}

interface Greeter {
    public function greet(): string;
}

function add($a, $b) {
    return $a + $b;
}
