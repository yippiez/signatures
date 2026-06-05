package com.example.app

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

const val API_BASE_URL = "https://api.example.com/v1"
const val DEFAULT_TIMEOUT_MS = 5000L

data class User(
    val id: Long,
    val name: String,
    val email: String,
    val role: UserRole,
)

data class PagedResult<T>(
    val items: List<T>,
    val total: Int,
    val page: Int,
    val pageSize: Int,
)

enum class UserRole {
    ADMIN,
    EDITOR,
    VIEWER,
}

interface UserRepository {
    suspend fun findById(id: Long): User?
    suspend fun findAll(page: Int, pageSize: Int): PagedResult<User>
    suspend fun save(user: User): User
    suspend fun delete(id: Long): Boolean
    fun observeChanges(): Flow<User>
}

interface AuthService {
    suspend fun login(email: String, password: String): String
    suspend fun logout(token: String)
    suspend fun validateToken(token: String): Boolean
    fun currentUser(): User?
}

class UserRepositoryImpl(
    private val db: Database,
    private val cache: Cache,
) : UserRepository {

    override suspend fun findById(id: Long): User? {
        val cached = cache.get("user:$id")
        if (cached != null) return cached as User
        return db.query("SELECT * FROM users WHERE id = ?", id)
    }

    override suspend fun findAll(page: Int, pageSize: Int): PagedResult<User> {
        val offset = page * pageSize
        val items = db.queryList("SELECT * FROM users LIMIT ? OFFSET ?", pageSize, offset)
        val total = db.queryScalar("SELECT COUNT(*) FROM users") as Int
        return PagedResult(items, total, page, pageSize)
    }

    override suspend fun save(user: User): User {
        return db.upsert(user)
    }

    override suspend fun delete(id: Long): Boolean {
        cache.evict("user:$id")
        return db.execute("DELETE FROM users WHERE id = ?", id) > 0
    }

    override fun observeChanges(): Flow<User> {
        return db.watchTable("users").map { row -> row.toUser() }
    }

    private fun Row.toUser(): User {
        return User(
            id = getLong("id"),
            name = getString("name"),
            email = getString("email"),
            role = UserRole.valueOf(getString("role")),
        )
    }
}

object AppConfig {
    val instance: AppConfig get() = this
    fun load(path: String): Map<String, String> {
        return emptyMap()
    }
    fun get(key: String, default: String = ""): String {
        return default
    }
}

fun <T> retry(times: Int, block: suspend () -> T): T {
    var lastError: Throwable? = null
    repeat(times) {
        try {
            return block()
        } catch (e: Throwable) {
            lastError = e
        }
    }
    throw lastError ?: IllegalStateException("retry failed")
}

suspend fun fetchUserProfile(id: Long, repo: UserRepository): User {
    return repo.findById(id) ?: throw NoSuchElementException("User $id not found")
}
