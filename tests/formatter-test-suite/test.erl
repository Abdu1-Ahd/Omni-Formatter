% ── CASE 1: Module and exports ───────────────────────────────────────────
-module(user).
-export([new/3, greet/1, classify/1]).

% ── CASE 2: Record definitions ────────────────────────────────────────────
-record(user, {
    id :: integer(),
    name :: binary(),
    email :: binary(),
    age = 0 :: non_neg_integer()
}).

% ── CASE 3: Function clauses and pattern matching ─────────────────────────
new(Id, Name, Email) ->
    #user{id = Id, name = Name, email = Email}.

greet(#user{name = Name}) ->
    io:format("Hello, ~s!~n", [Name]).

classify(N) when N < 0 ->
    negative;
classify(0) ->
    zero;
classify(N) when N < 10 ->
    small;
classify(_) ->
    large.

% ── CASE 4: List operations and guards ────────────────────────────────────
filter_adults(Users) ->
    [U || U = #user{age = A} <- Users, A >= 18].

sum_list([]) -> 0;
sum_list([H | T]) -> H + sum_list(T).

% ── CASE 5: Spawn and message passing ────────────────────────────────────
start_worker() ->
    spawn(fun() -> worker_loop() end).

worker_loop() ->
    receive
        {greet, Name} ->
            io:format("Hello ~s~n", [Name]),
            worker_loop();
        stop ->
            ok
    end.
