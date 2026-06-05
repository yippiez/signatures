<?php

// function fake_line_comment() { this should be ignored }
# function fake_hash_comment($x, $y) { ignored too }

/*
 * class FakeBlockClass {
 *     public function fakeMethod(): void {}
 * }
 * interface FakeInterface {}
 */

/**
 * class DocBlockFake {
 *     public function docFakeMethod() {}
 * }
 */

const REAL_CONST = 'real';

class StringDecoyContainer
{
    public const LABEL = 'hello';

    // function commentedInsideClass() {}
    # const FAKE_HASH = 99;

    public function getDoubleQuotedDecoy(): string
    {
        $code = "function fakeInDouble() { return 1; }";
        $other = "class FakeInDouble { public function x() {} }";
        return $code . $other;
    }

    public function getSingleQuotedDecoy(): string
    {
        $code = 'function fakeInSingle() { return 2; }';
        $cls  = 'class FakeSingle { const X = 1; }';
        return $code . $cls;
    }

    /*
     * function fakeInBlockInsideMethod() {}
     * class FakeBlockInside {}
     */
    public function realMethod(): bool
    {
        // class FakeInLineInsideMethod {}
        return true;
    }

    public function anotherReal(int $n): int
    {
        return $n * 2;
    }
}

interface RealInterface
{
    public function realInterfaceMethod(): string;
    public function anotherRealMethod(int $x): bool;
}

// class FakeAtEnd {}
// function fakeAtEnd() {}

function realFunction(int $n): int
{
    return $n * 2;
}

function anotherRealFunction(string $s, bool $flag = false): ?string
{
    return $flag ? strtoupper($s) : null;
}
