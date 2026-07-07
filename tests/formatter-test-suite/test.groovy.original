// ── CASE 1: Class — mixed indentation ─────────────────────────────────────
class User {
    int id
    String   name
    String email
    int age = 0

    User ( int id , String name , String email ) {
        this.id    = id
        this.name  = name
        this.email = email
    }

    String greet() {
        "Hello, ${name}!"
    }

    @Override
    String toString() {
        "User(${id}, ${name})"
    }
}

// ── CASE 2: Closures ──────────────────────────────────────────────────────
def double  = { x -> x * 2 }
def isEven  = { n -> n % 2 == 0 }
def classify = { n ->
    if (n < 0)      "negative"
    else if (n == 0) "zero"
    else if (n < 10) "small"
    else            "large"
}

// ── CASE 3: Collection operations ─────────────────────────────────────────
def numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

def evenSquares = numbers
    .findAll { it % 2 == 0 }
    .collect { it ** 2 }
    .sum()

def grouped = numbers.groupBy { it % 3 }

// ── CASE 4: Maps and GStrings ─────────────────────────────────────────────
def config = [
    host:   'localhost',
    port:   5432,
    name:   'mydb',
    pool:   5,
]

def url = "jdbc:postgresql://${config.host}:${config.port}/${config.name}"

// ── CASE 5: Metaprogramming and annotations ────────────────────────────────
@groovy.transform.Canonical
@groovy.transform.Immutable
class Point {
    double x
    double y

    double distanceTo(Point other) {
        Math.sqrt((x - other.x) ** 2 + (y - other.y) ** 2)
    }
}

// ── CASE 6: Long closure ──────────────────────────────────────────────────
def processUsersWithVeryLongClosureNameThatExceedsLineWidth(users, transform, predicate) {
    users.findAll(predicate).collect(transform)
}

// ── CASE 7: Trailing whitespace ────────────────────────────────────────────
def main() {   
    def user = new User(1, 'Alice', 'alice@example.com')   
    println user.greet()   
}
