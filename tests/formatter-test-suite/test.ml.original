-- ── CASE 1: Types and values ──────────────────────────────────────────────
type user = {
  id: int;
  name: string;
  email: string;
  age: int;
}

type shape =
  | Circle of float
  | Rectangle of float * float
  | Triangle of float * float * float

-- ── CASE 2: Functions and pattern matching ────────────────────────────────
let create_user id name email = { id; name; email; age = 0 }

let area = function
  | Circle r        -> Float.pi *. r *. r
  | Rectangle(w, h) -> w *. h
  | Triangle(a, b, c) ->
    let s = (a +. b +. c) /. 2.0 in
    sqrt (s *. (s-.a) *. (s-.b) *. (s-.c))

let classify n =
  if n < 0 then "negative"
  else if n = 0 then "zero"
  else if n < 10 then "small"
  else "large"

-- ── CASE 3: Module signatures ─────────────────────────────────────────────
module type COMPARABLE = sig
  type t
  val compare : t -> t -> int
  val equal : t -> t -> bool
end

module IntComparable : COMPARABLE with type t = int = struct
  type t = int
  let compare = Int.compare
  let equal = Int.equal
end

-- ── CASE 4: Higher-order functions ────────────────────────────────────────
let compose f g x = f (g x)
let apply_twice f x = f (f x)

let pipeline = List.filter (fun x -> x mod 2 = 0)
  |> List.map (fun x -> x * x)
  |> List.fold_left (+) 0

-- ── CASE 5: Error handling with result ────────────────────────────────────
let safe_divide a b =
  if b = 0 then Error "division by zero"
  else Ok (a / b)

let process n =
  match safe_divide 100 n with
  | Ok result  -> Printf.printf "Result: %d\n" result
  | Error msg  -> Printf.printf "Error: %s\n" msg
