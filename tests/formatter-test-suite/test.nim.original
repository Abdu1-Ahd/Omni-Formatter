# ── CASE 1: Types and variables ───────────────────────────────────────────
type
  UserId = distinct int
  UserRole = enum
    urAdmin, urUser, urGuest

  User = object
    id*:    UserId
    name*:  string
    email*: string
    role*:  UserRole
    age:    int

# ── CASE 2: Procedures ────────────────────────────────────────────────────
proc newUser*(id: UserId, name, email: string): User =
  User(id: id, name: name, email: email, role: urUser)

proc greet*(u: User): string =
  "Hello, " & u.name & "!"

proc `$`*(u: User): string =
  "User(" & $u.id.int & ", " & u.name & ")"

# ── CASE 3: Templates and generics ────────────────────────────────────────
template check*(cond: bool, msg: string) =
  if not cond:
    raise newException(ValueError, msg)

proc maxVal*[T: Ordinal](a, b: T): T =
  if a > b: a else: b

# ── CASE 4: Iterator and closure ──────────────────────────────────────────
iterator evens*(n: int): int =
  for i in 0 ..< n:
    if i mod 2 == 0:
      yield i

proc makeAdder*(x: int): proc(y: int): int =
  result = proc(y: int): int = x + y

# ── CASE 5: Exception handling ────────────────────────────────────────────
proc safeDivide*(a, b: int): int =
  if b == 0:
    raise newException(DivByZeroDefect, "division by zero")
  a div b

proc processNumber(n: int) =
  try:
    let result = safeDivide(100, n)
    echo "Result: ", result
  except DivByZeroDefect as e:
    echo "Error: ", e.msg
  finally:
    echo "Done"

# ── CASE 6: Long line ─────────────────────────────────────────────────────
proc veryLongProcedureNameThatExceedsLineWidth*(paramOne: string, paramTwo: int, paramThree: float, paramFour: bool = false): string =
  $paramOne & " " & $paramTwo & " " & $paramThree

# ── CASE 7: Trailing whitespace ───────────────────────────────────────────
proc trailingExample() =   
  let x = 1   
  let y = 2  
  echo x + y
