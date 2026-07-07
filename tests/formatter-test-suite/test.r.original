# ── CASE 1: Variable assignment — spacing ─────────────────────────────────
name <- "Alice"
age  <-  30
height<-1.72
is_active =TRUE

# ── CASE 2: Functions ─────────────────────────────────────────────────────
greet <- function(name, greeting = "Hello") {
    paste(greeting, name, sep = ", ")
}

classify <- function(n) {
    if (n < 0) {
        "negative"
    } else if (n == 0) {
        "zero"
    } else if (n < 10) {
        "small"
    } else {
        "large"
    }
}

# ── CASE 3: Vectors and lists ──────────────────────────────────────────────
numbers <- c(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
evens   <- numbers[numbers %% 2 == 0]
squared <- evens^2

user <- list(
    id    = 1L,
    name  = "Alice",
    email = "alice@example.com",
    age   = 30L
)

# ── CASE 4: Data frame ─────────────────────────────────────────────────────
df <- data.frame(
    name  = c("Alice", "Bob",   "Carol"),
    age   = c(30L,     25L,     35L),
    score = c(9.5,     8.2,     9.1),
    stringsAsFactors = FALSE
)

# Filtering and transformation
adults  <- df[df$age >= 18, ]
top     <- adults[order(-adults$score), ][1:2, ]

# ── CASE 5: Apply family ───────────────────────────────────────────────────
results <- sapply(1:10, function(x) x^2 + 2*x + 1)
mapped  <- lapply(df$name, toupper)

# ── CASE 6: Pipe operator (R 4.1+ native) ────────────────────────────────
library(dplyr)
processed <- df |>
    filter(age >= 25) |>
    mutate(score_rank = rank(-score)) |>
    select(name, age, score, score_rank) |>
    arrange(score_rank)

# ── CASE 7: Long line ─────────────────────────────────────────────────────
very_long_variable_name_that_exceeds_line_width <- paste("Hello", "World", "from", "R", sep = " - ")

# ── CASE 8: Trailing whitespace ────────────────────────────────────────────
cat(greet(name))
cat("\n")
