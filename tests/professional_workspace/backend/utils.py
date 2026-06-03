def calculate_tax(amount: float, rate: float) -> float:
  return amount * rate
# fmt: off
MATRIX = [
    1, 0, 0,
    0, 1, 0,
    0, 0, 1
]
# fmt: on


def apply_discount(amount: float) -> float:
  return amount * 0.9
