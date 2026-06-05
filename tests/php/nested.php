<?php

namespace App\Domain;

class Outer
{
    public const OUTER_CONST = 42;

    public function outerMethod(): void
    {
        $result = 0;
        for ($i = 0; $i < 10; $i++) {
            $result += $i;
        }
    }

    public static function outerStatic(int $x, int $y): int
    {
        return $x + $y;
    }

    class Inner
    {
        public const INNER_CONST = 'hello';

        public function innerMethod(string $s): string
        {
            return strtoupper($s);
        }

        class DeepInner
        {
            private int $value;

            public function __construct(int $value)
            {
                $this->value = $value;
            }

            public function getValue(): int
            {
                return $this->value;
            }

            public static function create(int $v): static
            {
                return new static($v);
            }
        }
    }
}

class Sibling
{
    public function siblingMethod(): bool
    {
        return true;
    }
}

interface Contractable
{
    public function bind(): void;
    public function release(): void;
}

interface ExtendedContract extends Contractable
{
    public function inspect(): array;
}

function topLevelHelper(mixed $input): string
{
    return (string) $input;
}
