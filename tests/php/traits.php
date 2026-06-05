<?php

namespace App\Concerns;

trait Timestamps
{
    public const DATE_FORMAT = 'Y-m-d H:i:s';

    private ?\DateTimeImmutable $createdAt = null;
    private ?\DateTimeImmutable $updatedAt = null;

    public function getCreatedAt(): ?\DateTimeImmutable
    {
        return $this->createdAt;
    }

    public function getUpdatedAt(): ?\DateTimeImmutable
    {
        return $this->updatedAt;
    }

    public function touch(): void
    {
        $this->updatedAt = new \DateTimeImmutable();
    }

    protected function initTimestamps(): void
    {
        $now = new \DateTimeImmutable();
        $this->createdAt = $now;
        $this->updatedAt = $now;
    }
}

trait SoftDeletes
{
    private bool $deleted = false;
    private ?\DateTimeImmutable $deletedAt = null;

    public function delete(): void
    {
        $this->deleted = true;
        $this->deletedAt = new \DateTimeImmutable();
    }

    public function restore(): void
    {
        $this->deleted = false;
        $this->deletedAt = null;
    }

    public function isDeleted(): bool
    {
        return $this->deleted;
    }

    public static function withTrashed(): array
    {
        return [];
    }
}

trait Serializable
{
    abstract public function toArray(): array;

    public function toJson(int $flags = 0): string
    {
        return json_encode($this->toArray(), $flags);
    }

    public static function fromArray(array $data): static
    {
        $instance = new static();
        foreach ($data as $key => $value) {
            $instance->$key = $value;
        }
        return $instance;
    }
}

class Model
{
    use Timestamps, SoftDeletes, Serializable;

    public const TABLE = 'models';

    protected int $id;
    protected string $name;

    public function __construct(int $id, string $name)
    {
        $this->id   = $id;
        $this->name = $name;
        $this->initTimestamps();
    }

    public function toArray(): array
    {
        return ['id' => $this->id, 'name' => $this->name];
    }

    public function getId(): int
    {
        return $this->id;
    }
}

interface Persistable
{
    public function save(): bool;
    public function delete(): void;
}
