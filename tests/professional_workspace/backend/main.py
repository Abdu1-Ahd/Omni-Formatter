from fastapi import FastAPI
from pydantic import BaseModel
app = FastAPI()


class Item(BaseModel):
  name: str
  description: str = None
  price: float
  tax: float = None
# Messy spacing
@app.post("/items/")


def create_item(item: Item):
  return item
@app.get("/items/{item_id}")


def read_item(item_id: int, q: str = None):
  return {"item_id": item_id, "q": q}
@app.get("/users/")


def get_users():
  return [{"username": "john"}, {"username": "jane"}]
# long line for black to wrap


def very_long_function_name(
  argument_one: str,
  argument_two: int,
  argument_three: float,
  ) -> str:,
)
  pass
