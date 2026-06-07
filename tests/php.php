@@CASE@@ comments_strings
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
@@CASE@@ edge
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
@@CASE@@ modern
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
@@CASE@@ nested
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
@@CASE@@ realworld
<?php

namespace App\Http\Controllers;

use App\Models\User;
use App\Services\AuthService;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;

const API_VERSION = 'v1';

define('MAX_LOGIN_ATTEMPTS', 5);

class UserController extends BaseController
{
    public const CACHE_TTL = 3600;
    private const SECRET_KEY = 'hidden';

    private AuthService $authService;

    public function __construct(AuthService $authService)
    {
        $this->authService = $authService;
        parent::__construct();
    }

    public function index(Request $request): JsonResponse
    {
        $users = User::query()
            ->when($request->has('active'), fn($q) => $q->where('active', true))
            ->paginate(20);

        return response()->json(['data' => $users]);
    }

    public function show(int $id): JsonResponse
    {
        $user = User::findOrFail($id);
        return response()->json($user);
    }

    public function store(Request $request): JsonResponse
    {
        $validated = $request->validate([
            'name'  => 'required|string|max:255',
            'email' => 'required|email|unique:users',
        ]);

        $user = User::create($validated);
        return response()->json($user, 201);
    }

    public function update(Request $request, int $id): JsonResponse
    {
        $user = User::findOrFail($id);
        $user->update($request->validated());
        return response()->json($user);
    }

    public function destroy(int $id): JsonResponse
    {
        User::findOrFail($id)->delete();
        return response()->json(null, 204);
    }

    protected static function resolveMiddleware(): array
    {
        return ['auth:api', 'throttle:60,1'];
    }

    private function authorize(string $ability): void
    {
        if (!$this->authService->can($ability)) {
            abort(403);
        }
    }
}

interface ResourceController
{
    public function index(Request $request): JsonResponse;
    public function show(int $id): JsonResponse;
    public function store(Request $request): JsonResponse;
    public function update(Request $request, int $id): JsonResponse;
    public function destroy(int $id): JsonResponse;
}

function sanitizeInput(string $input): string
{
    return htmlspecialchars(strip_tags(trim($input)), ENT_QUOTES, 'UTF-8');
}

function paginateArray(array $items, int $perPage = 15, int $page = 1): array
{
    $offset = ($page - 1) * $perPage;
    return array_slice($items, $offset, $perPage);
}
@@CASE@@ sample
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
@@CASE@@ traits
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
@@CASE@@ heredoc_nowdoc
<?php
function f(): string {
    return <<<EOT
    function fakeHeredoc() {}
    EOT;
}
class C {
    public function m(): string {
        return <<<'EOT'
        }}} class FakeNowdoc {}
        EOT;
    }
    public function after(): int { return 42; }
}
function g(): void {}
@@CASE@@ attribute_inline_and_line
<?php
#[Route("/api")]
class Api {
    public function handle(#[Inject] string $svc, int $id): void {}
}
