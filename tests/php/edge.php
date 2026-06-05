<?php

// Edge cases: unusual but valid PHP constructs

namespace Edge\Cases;

define('RUNTIME_CONST', PHP_INT_MAX);
define('ANOTHER_CONST', true);

const TYPED_CONST = 3.14;

abstract class AbstractBase
{
    abstract public function mustImplement(): void;
    abstract protected function mustImplementProtected(int $x): string;

    public function concreteMethod(): string
    {
        return static::class;
    }

    final public function finalMethod(): bool
    {
        return false;
    }

    public static function abstractStatic(): void
    {
    }
}

final class ConcreteChild extends AbstractBase
{
    public const VERSION = '1.0.0';

    public function __construct(
        private readonly string $name,
        protected int $count = 0,
    ) {}

    public function mustImplement(): void
    {
        echo $this->name;
    }

    protected function mustImplementProtected(int $x): string
    {
        return (string) ($x * $this->count);
    }

    public function __toString(): string
    {
        return $this->name;
    }

    public function __clone(): void
    {
        $this->count = 0;
    }

    public static function withName(string $name): static
    {
        return new static($name);
    }
}

interface Countable
{
    public function count(): int;
}

interface Stringable
{
    public function __toString(): string;
}

interface Both extends Countable, Stringable
{
    public function isEmpty(): bool;
}

class NullableTypes
{
    public function maybeNull(?string $s): ?int
    {
        return $s !== null ? strlen($s) : null;
    }

    public function unionTypes(int|string $val): bool|null
    {
        return null;
    }

    public function intersectionTypes(\Countable&\Iterator $col): void
    {
    }

    public function neverReturn(): never
    {
        throw new \RuntimeException('never');
    }

    public static function voidStatic(): void
    {
    }
}

function variadic(string $first, mixed ...$rest): array
{
    return [$first, ...$rest];
}

function withDefaultComplex(
    array $opts = [],
    ?callable $fn = null,
    int $flags = PHP_INT_SIZE,
): mixed {
    return $fn ? $fn($opts) : $opts;
}
