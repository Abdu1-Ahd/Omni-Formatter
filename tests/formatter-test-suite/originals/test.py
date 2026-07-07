import os, sys, math
import datetime
import json,re
from collections import defaultdict,Counter

# ── CASE 1: Class definition — wrong indentation ───────────────────────────
class ProductData:
  def __init__(self,id,name, price,category):
    self.id= id
    self.name=name
    self.price = price
    self.category= category
  def get_discounted_price( self , discount_rate ):
    return self.price - ( self.price* discount_rate)
  def __str__(self):
    return "Product: " + str(self.id) + " - " + self.name


# ── CASE 2: Class with inconsistent indent (tabs vs spaces) ────────────────
class inventory_manager:
  def __init__(self ):
    self.items={}
    self.categories=set()
  def add_item(self,product,qty):
    if product.id in self.items:
      self.items[product.id]['qty']+=qty
    else:
      self.items[product.id]={'product':product,'qty':qty}
      self.categories.add(product.category)


# ── CASE 3: Function with mixed quote styles ────────────────────────────────
def process_record(record):
    name = record['name']
    status = record["status"]
    tag = record['tag'] if record['tag'] else "default"
    return {'name': name, "status": status, 'tag': tag}


# ── CASE 4: Long lines — should wrap ───────────────────────────────────────
def very_long_function_name_that_exceeds_the_line_width(argument_one, argument_two, argument_three, argument_four, argument_five):
    return argument_one + argument_two + argument_three + argument_four + argument_five


# ── CASE 5: Nested blocks — correct base indentation + extra ───────────────
def nested_logic(data):
  for item in data:
    if item > 0:
      for sub in range(item):
        if sub % 2 == 0:
          print(sub)
        else:
          continue


# ── CASE 6: Trailing whitespace ────────────────────────────────────────────
def with_trailing():   
    x = 1   
    y = 2  
    return x + y  


# ── CASE 7: Magic trailing comma — should force multi-line expansion ────────
result = [
    item for item in some_list
    if item.is_valid()
]
compact = [1, 2, 3,]
forced_multiline = (
    value_one,
    value_two,
    value_three,
)


# ── CASE 8: Dictionary with mixed alignment ────────────────────────────────
config = {
    'host':    'localhost',
    'port':    5432,
    'database':   'mydb',
    'user':'admin',
    'password':    'secret',
}


# ── CASE 9: f-strings and string concatenation ────────────────────────────
name = 'World'
greeting = f'Hello, {name}!'
old_style = 'Hello, ' + name + '!'
formatted = "Result: %s" % name


# ── CASE 10: Decorator syntax ─────────────────────────────────────────────
@property
def value(self):
    return self._value

@value.setter
def value(self, val):
    self._value = val

@staticmethod
def create():
    return MyClass()


# ── CASE 11: Lambda and comprehension ─────────────────────────────────────
square = lambda x: x**2
evens = [x for x in range(100) if x % 2 == 0]
pairs = {k: v for k, v in zip(keys, values) if k is not None}


# ── CASE 12: try/except/finally ───────────────────────────────────────────
def safe_divide(a, b):
  try:
    result = a / b
  except ZeroDivisionError as e:
    print(f'Error: {e}')
    result = None
  except (TypeError, ValueError):
    result = 0
  else:
    print('Success')
  finally:
    return result


# ── CASE 13: Type annotations — PEP 526 and return types ──────────────────
def greet(name: str, times: int = 1) -> str:
    return (name + " ") * times

items: list[int] = []
mapping: dict[str, list[int]] = {}
