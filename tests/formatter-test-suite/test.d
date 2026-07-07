module myapp;

// ── CASE 1: Import declarations ────────────────────────────────────────────
import std.stdio;
import std.string;
import std.algorithm;
import std.range;
import std.conv;

// ── CASE 2: Struct with methods ────────────────────────────────────────────
struct User {
    int    id;
    string name;
    string email;
    int    age = 0;

    this ( int id , string name , string email ) {
        this.id    = id;
        this.name  = name;
        this.email = email;
    }

    string greet() const {
        return "Hello, " ~ name ~ "!";
    }

    override string toString() const {
        return format("User(%d, %s)", id, name);
    }
}

// ── CASE 3: Templates (generics) ──────────────────────────────────────────
T maxOf(T)(T a, T b) {
    return a > b ? a : b;
}

struct Stack(T) {
    private T[] items;

    void push(T item) { items ~= item; }
    T pop() {
        auto item = items[$-1];
        items.length--;
        return item;
    }
    bool empty() const { return items.length == 0; }
}

// ── CASE 4: Ranges and algorithms ─────────────────────────────────────────
void processNumbers() {
    auto numbers = iota(1, 11)
        .filter!(n => n % 2 == 0)
        .map!(n => n * n)
        .array;
    writeln(numbers);
}

// ── CASE 5: Classes and interfaces ────────────────────────────────────────
interface Describable {
    string describe() const;
}

class Shape : Describable {
    abstract double area() const;
    string describe() const { return format("Area: %f", area()); }
}

class Circle : Shape {
    double radius;
    this(double r) { radius = r; }
    override double area() const { return 3.14159 * radius * radius; }
}

// ── CASE 6: Long function signature ────────────────────────────────────────
void veryLongFunctionNameThatExceedsLineWidth(string paramOne, string paramTwo, int paramThree, double paramFour = 0.0) {
    writefln("%s %s %d %f", paramOne, paramTwo, paramThree, paramFour);
}

// ── CASE 7: Trailing whitespace ────────────────────────────────────────────
void main() {   
    auto u = User(1, "Alice", "alice@example.com");   
    writeln(u.greet());   
}
