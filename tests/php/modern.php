<?php

declare(strict_types=1);

namespace App\Enums;

enum Status: string
{
    case Active   = 'active';
    case Inactive = 'inactive';
    case Pending  = 'pending';

    public function label(): string
    {
        return match($this) {
            Status::Active   => 'Active',
            Status::Inactive => 'Inactive',
            Status::Pending  => 'Pending',
        };
    }

    public function isActive(): bool
    {
        return $this === Status::Active;
    }

    public static function fromLabel(string $label): self
    {
        foreach (self::cases() as $case) {
            if ($case->label() === $label) {
                return $case;
            }
        }
        throw new \ValueError("Invalid label: $label");
    }
}

enum Priority: int
{
    case Low    = 1;
    case Medium = 5;
    case High   = 10;

    public function isHigherThan(self $other): bool
    {
        return $this->value > $other->value;
    }
}

enum Suit
{
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;

    public function color(): string
    {
        return match($this) {
            Suit::Hearts, Suit::Diamonds => 'red',
            Suit::Clubs,  Suit::Spades   => 'black',
        };
    }
}

interface HasStatus
{
    public function getStatus(): Status;
    public function setStatus(Status $status): void;
}

readonly class Point
{
    public function __construct(
        public readonly float $x,
        public readonly float $y,
        public readonly float $z = 0.0,
    ) {}

    public function distanceTo(self $other): float
    {
        return sqrt(
            ($this->x - $other->x) ** 2 +
            ($this->y - $other->y) ** 2 +
            ($this->z - $other->z) ** 2
        );
    }

    public function translate(float $dx, float $dy, float $dz = 0.0): self
    {
        return new self($this->x + $dx, $this->y + $dy, $this->z + $dz);
    }
}

class Pipeline
{
    private array $stages = [];

    public function pipe(callable $stage): static
    {
        $clone = clone $this;
        $clone->stages[] = $stage;
        return $clone;
    }

    public function process(mixed $payload): mixed
    {
        return array_reduce(
            $this->stages,
            fn(mixed $carry, callable $stage) => $stage($carry),
            $payload
        );
    }

    public static function make(): static
    {
        return new static();
    }
}

const DEFAULT_TIMEOUT = 30;

function retry(callable $fn, int $times = 3, int $sleep = 0): mixed
{
    for ($attempt = 1; $attempt <= $times; $attempt++) {
        try {
            return $fn($attempt);
        } catch (\Throwable $e) {
            if ($attempt === $times) throw $e;
            if ($sleep > 0) usleep($sleep * 1000);
        }
    }
}
