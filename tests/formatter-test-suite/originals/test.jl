# ── CASE 1: Variables and basic types ────────────────────────────────────
name   = "Alice"
age    =  30
height = 1.72
is_active = true

# ── CASE 2: Struct and methods ────────────────────────────────────────────
struct User
    id::Int
    name::String
    email::String
    age::Int
end

function User(id::Int, name::String, email::String)
    User(id, name, email, 0)
end

function greet(u::User)
    "Hello, $(u.name)!"
end

function Base.show(io::IO, u::User)
    print(io, "User($(u.id), $(u.name))")
end

# ── CASE 3: Multiple dispatch ─────────────────────────────────────────────
function classify(n::Integer)
    if n < 0
        "negative"
    elseif n == 0
        "zero"
    elseif n < 10
        "small"
    else
        "large"
    end
end

function classify(x::Float64)
    if isnan(x)
        "NaN"
    elseif isinf(x)
        "Infinity"
    else
        classify(round(Int, x))
    end
end

# ── CASE 4: Abstract types and parametric types ────────────────────────────
abstract type Shape end

struct Circle <: Shape
    radius::Float64
end

struct Rectangle <: Shape
    width::Float64
    height::Float64
end

area(c::Circle)    = π * c.radius^2
area(r::Rectangle) = r.width * r.height

# ── CASE 5: Comprehensions and generators ─────────────────────────────────
squares   = [x^2 for x in 1:10]
evens     = [x for x in 1:20 if x % 2 == 0]
dict_comp = Dict(k => k^2 for k in 1:5)

# ── CASE 6: Macros ────────────────────────────────────────────────────────
@time begin
    result = sum(x^2 for x in 1:1_000_000)
    println("Sum: $result")
end

@assert area(Circle(1.0)) ≈ π "Circle area should be π"

# ── CASE 7: Long signature ────────────────────────────────────────────────
function processWithVeryLongFunctionNameExceedingLineWidth(input::Vector{User}, transform::Function, predicate::Function)::Vector{User}
    filter(predicate, map(transform, input))
end
