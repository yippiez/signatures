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
