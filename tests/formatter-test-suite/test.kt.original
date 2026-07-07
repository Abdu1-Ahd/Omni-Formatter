// ── CASE 1: Data class and properties ────────────────────────────────────
data class User(
    val id: Int,
    val name  :String,
    val email:String,
    val age: Int= 0
)

// ── CASE 2: Sealed class — mixed indentation ──────────────────────────────
sealed class Result<out T> {
  data class Success<T>(val data:T): Result<T>()
    data class Error(val message:String, val cause:Throwable?=null): Result<Nothing>()
  object Loading : Result<Nothing>()
}

// ── CASE 3: Extension functions ───────────────────────────────────────────
fun String.isValidEmail(): Boolean {
    val regex = Regex("[A-Z0-9a-z._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}")
    return regex.matches(this)
}

fun <T> List<T>.secondOrNull(): T? = if (size >= 2) this[1] else null

// ── CASE 4: Higher-order functions ───────────────────────────────────────
fun <T, R> List<T>.mapNotEmpty(transform:(T)->R): List<R> {
    return if (isEmpty()) emptyList() else map(transform)
}

// ── CASE 5: When expression ───────────────────────────────────────────────
fun classify(result: Result<*>): String = when (result) {
    is Result.Success -> "success: ${(result as Result.Success<*>).data}"
    is Result.Error   -> "error: ${result.message}"
    Result.Loading    -> "loading"
}

// ── CASE 6: Coroutines and suspend functions ──────────────────────────────
suspend fun fetchUser(id: Int): Result<User> {
    return try {
        val user = apiService.getUser(id)
        Result.Success(user)
    } catch (e: Exception) {
        Result.Error("Failed to fetch user",e)
    }
}

// ── CASE 7: Long function signature ──────────────────────────────────────
fun processDataWithVeryLongFunctionNameThatExceedsLineWidth(inputData: List<String>, transform: (String) -> String, predicate: (String) -> Boolean): List<String> {
    return inputData.filter(predicate).map(transform)
}

// ── CASE 8: Companion object ──────────────────────────────────────────────
class Repository private constructor() {
    companion object {
        @Volatile private var instance: Repository? = null

        fun getInstance(): Repository = instance ?: synchronized(this) {
            instance ?: Repository().also { instance = it }
        }
    }
}

// ── CASE 9: Trailing whitespace ───────────────────────────────────────────
fun main() {   
    val user = User(1,"Alice","alice@example.com",30)   
    println(classify(Result.Success(user)))   
}
