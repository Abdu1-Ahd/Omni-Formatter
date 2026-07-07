defmodule MyApp.User do
  # ── CASE 1: Struct definition ──────────────────────────────────────────────
  defstruct [:id, :name, :email, age: 0, role: :user]

  @type t :: %__MODULE__{
    id: integer(),
    name: String.t(),
    email: String.t(),
    age: non_neg_integer(),
    role: atom()
  }

  # ── CASE 2: Functions — guards and pattern matching ─────────────────────
  def new(id, name, email) do
    %__MODULE__{id: id, name: name, email: email}
  end

  def greet(%__MODULE__{name: name}) when is_binary(name) do
    "Hello, #{name}!"
  end

  def greet(_), do: "Hello, stranger!"

  # ── CASE 3: Pipeline operator ────────────────────────────────────────────
  def process_name(name) do
    name
    |> String.trim()
    |> String.downcase()
    |> String.split(" ")
    |> Enum.map(&String.capitalize/1)
    |> Enum.join(" ")
  end

  # ── CASE 4: Comprehension ────────────────────────────────────────────────
  def valid_emails(users) do
    for user <- users,
        String.contains?(user.email, "@"),
        user.role != :banned,
        do: user.email
  end

  # ── CASE 5: with macro ────────────────────────────────────────────────────
  def fetch_and_validate(id) do
    with {:ok, user} <- fetch_user(id),
         {:ok, _} <- validate(user),
         :ok <- authorize(user) do
      {:ok, user}
    else
      {:error, :not_found} -> {:error, "User not found"}
      {:error, reason}     -> {:error, reason}
    end
  end
end

defmodule MyApp.UserTest do
  use ExUnit.Case

  # ── CASE 6: ExUnit tests ──────────────────────────────────────────────────
  test "new/3 creates a user struct" do
    user = MyApp.User.new(1,"Alice","alice@example.com")
    assert user.name == "Alice"
    assert user.email == "alice@example.com"
  end

  test "greet/1 returns greeting string" do
    user = %MyApp.User{id: 1, name: "Bob", email: "bob@example.com"}
    assert MyApp.User.greet(user) == "Hello, Bob!"
  end
end
