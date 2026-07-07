// ── CASE 1: Simple values and functions ──────────────────────────────────
let name = "Alice"
let age  =  30
let greeting = sprintf "Hello, %s! You are %d years old." name age

// ── CASE 2: Functions — pipeline and composition ──────────────────────────
let double x = x * 2
let isEven n = n % 2 = 0
let square x = x * x

let result =
    [1..10]
    |> List.filter isEven
    |> List.map square
    |> List.sum

// ── CASE 3: Discriminated unions ─────────────────────────────────────────
type Shape =
    | Circle of radius: float
    | Rectangle of width: float * height: float
    | Triangle of base: float * height: float

let area shape =
    match shape with
    | Circle r -> System.Math.PI * r * r
    | Rectangle(w, h) -> w * h
    | Triangle(b, h) -> 0.5 * b * h

// ── CASE 4: Record types ─────────────────────────────────────────────────
type Person = {
    Id: int
    Name: string
    Email: string
    Age: int
}

let alice = { Id=1; Name="Alice"; Email="alice@example.com"; Age=30 }

// ── CASE 5: Computation expressions (async) ───────────────────────────────
open System.Net.Http

let fetchAsync (url: string) = async {
    use client = new HttpClient()
    let! response = client.GetStringAsync(url) |> Async.AwaitTask
    return response
}

// ── CASE 6: Active patterns ───────────────────────────────────────────────
let (|Even|Odd|) n =
    if n % 2 = 0 then Even else Odd

let describe n =
    match n with
    | Even -> sprintf "%d is even" n
    | Odd  -> sprintf "%d is odd"  n

// ── CASE 7: Long line ─────────────────────────────────────────────────────
let veryLongFunctionNameThatExceedsLineWidth (parameterOne: string) (parameterTwo: int) (parameterThree: float) : string =
    sprintf "%s %d %f" parameterOne parameterTwo parameterThree

// ── CASE 8: Trailing whitespace ───────────────────────────────────────────
let main _ =   
    printfn "%s" greeting   
    printfn "Sum: %d" result   
    0
