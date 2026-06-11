import os, sys, math
import datetime
import json,re
from collections import defaultdict,Counter
from itertools import chain


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
  def remove_item( self, product_id , qty ):
    if product_id in self.items:
      self.items[product_id]['qty'] -= qty
      if self.items[product_id]['qty']<=0:
        del self.items[product_id]
    else:
      print( 'Item not found' )
  def get_total_value(self):
    total=0.0
    for pid,data in self.items.items():
      total += data['product'].price * data['qty']
    return total


def generate_mock_database():
  db = [
        ProductData(1,"Laptop",999.99, "Electronics"), ProductData(2,"Mouse",19.99,"Electronics"),
        ProductData(3,"Keyboard", 49.99, "Electronics"), ProductData( 4, "Desk", 199.99, "Furniture"),
        ProductData(5,"Chair",89.99,"Furniture"),ProductData(6,"Monitor", 149.99, 'Electronics'),
        ProductData(7, "USB Cable", 9.99, "Accessories"), ProductData(8,"HDMI Cable",14.99,"Accessories"),
        ProductData(9, 'Webcam', 59.99, 'Electronics'),ProductData(10,"Microphone", 45.00, "Electronics"),
        ProductData(11,"Headphones", 89.50, "Electronics"),ProductData(12,"Speakers", 35.99, "Electronics"),
        ProductData(13,"External Hard Drive", 120.00, "Storage"),ProductData(14,"USB Flash Drive", 15.00, "Storage"),
        ProductData(15,"Smartphone", 799.00, "Electronics"),ProductData(16,"Tablet", 350.00, "Electronics"),
        ProductData(17,"Smartwatch", 199.00, "Electronics"),ProductData(18,"Fitness Tracker", 99.00, "Electronics"),
        ProductData(19,"Power Bank", 29.99, "Accessories"),ProductData(20,"Phone Case", 12.99, "Accessories"),
        ProductData(21,"Screen Protector", 8.99, "Accessories"),ProductData(22,"Laptop Sleeve", 24.99, "Accessories"),
        ProductData(23,"Backpack", 49.99, "Accessories"),ProductData(24,"Messenger Bag", 39.99, "Accessories"),
        ProductData(25,"Gaming Mouse", 59.99, "Electronics"),ProductData(26,"Gaming Keyboard", 89.99, "Electronics"),
        ProductData(27,"Gaming Headset", 79.99, "Electronics"),ProductData(28,"Mouse Pad", 14.99, "Accessories"),
        ProductData(29,"Router", 89.00, "Networking"),ProductData(30,"Modem", 69.00, "Networking"),
        ProductData(31,"Ethernet Cable", 11.99, "Networking"),ProductData(32,"Wi-Fi Extender", 45.00, "Networking"),
        ProductData(33,"Printer", 129.00, "Office"),ProductData(34,"Ink Cartridge", 35.00, "Office"),
        ProductData(35,"Paper", 8.50, "Office"),ProductData(36,"Pens", 5.99, "Office"),
        ProductData(37,"Notebook", 4.50, "Office"),ProductData(38,"Stapler", 9.50, "Office"),
        ProductData(39,"Paper Clips", 2.99, "Office"),ProductData(40,"Whiteboard", 45.00, "Office"),
        ProductData(41,"Markers", 12.50, "Office"),ProductData(42,"Eraser", 3.50, "Office"),
        ProductData(43,"Projector", 299.00, "Electronics"),ProductData(44,"Projector Screen", 89.00, "Accessories"),
        ProductData(45,"Camera", 450.00, "Electronics"),ProductData(46,"Tripod", 35.00, "Accessories"),
        ProductData(47,"Memory Card", 25.00, "Storage"),ProductData(48,"Camera Bag", 45.00, "Accessories"),
        ProductData(49,"Lens", 250.00, "Electronics"),ProductData(50,"Flash", 85.00, "Electronics"),
        ProductData(51,"Batteries", 12.00, "Accessories"),ProductData(52,"Battery Charger", 18.00, "Accessories"),
        ProductData(53,"Desk Lamp", 25.00, "Furniture"),ProductData(54,"Floor Lamp", 45.00, "Furniture"),
        ProductData(55,"Light Bulbs", 9.00, "Accessories"),ProductData(56,"Extension Cord", 15.00, "Accessories"),
        ProductData(57,"Surge Protector", 22.00, "Accessories"),ProductData(58,"Tool Kit", 55.00, "Hardware"),
        ProductData(59,"Screwdriver Set", 18.00, "Hardware"),ProductData(60,"Hammer", 12.00, "Hardware"),
        ProductData(61,"Wrench", 14.00, "Hardware"),ProductData(62,"Pliers", 10.00, "Hardware"),
        ProductData(63,"Tape Measure", 8.00, "Hardware"),ProductData(64,"Level", 15.00, "Hardware"),
        ProductData(65,"Utility Knife", 6.00, "Hardware"),ProductData(66,"Duct Tape", 5.00, "Hardware"),
        ProductData(67,"Glue", 4.00, "Hardware"),ProductData(68,"Screws", 3.00, "Hardware"),
        ProductData(69,"Nails", 3.00, "Hardware"),ProductData(70,"Hooks", 5.00, "Hardware"),
        ProductData(71,"Wall Anchors", 6.00, "Hardware"),ProductData(72,"Paint", 25.00, "Hardware"),
        ProductData(73,"Paint Brushes", 12.00, "Hardware"),ProductData(74,"Paint Roller", 15.00, "Hardware"),
        ProductData(75,"Drop Cloth", 8.00, "Hardware"),ProductData(76,"Sandpaper", 5.00, "Hardware"),
        ProductData(77,"Safety Glasses", 10.00, "Hardware"),ProductData(78,"Work Gloves", 12.00, "Hardware"),
        ProductData(79,"Dust Mask", 8.00, "Hardware"),ProductData(80,"Flashlight", 15.00, "Hardware"),
        ProductData(81,"Bicycle", 250.00, "Sports"),ProductData(82,"Helmet", 35.00, "Sports"),
        ProductData(83,"Bike Lock", 20.00, "Sports"),ProductData(84,"Bike Pump", 15.00, "Sports"),
        ProductData(85,"Water Bottle", 10.00, "Sports"),ProductData(86,"Yoga Mat", 25.00, "Sports"),
        ProductData(87,"Dumbbells", 45.00, "Sports"),ProductData(88,"Resistance Bands", 15.00, "Sports"),
        ProductData(89,"Jump Rope", 10.00, "Sports"),ProductData(90,"Treadmill", 800.00, "Sports"),
        ProductData(91,"Exercise Bike", 400.00, "Sports"),ProductData(92,"Rowing Machine", 500.00, "Sports"),
        ProductData(93,"Tennis Racket", 85.00, "Sports"),ProductData(94,"Tennis Balls", 12.00, "Sports"),
        ProductData(95,"Basketball", 25.00, "Sports"),ProductData(96,"Soccer Ball", 25.00, "Sports"),
        ProductData(97,"Football", 25.00, "Sports"),ProductData(98,"Baseball", 8.00, "Sports"),
        ProductData(99,"Baseball Bat", 45.00, "Sports"),ProductData(100,"Baseball Glove", 35.00, "Sports"),
        ProductData(101,"Golf Clubs", 350.00, "Sports"),ProductData(102,"Golf Balls", 20.00, "Sports"),
        ProductData(103,"Tees", 5.00, "Sports"),ProductData(104,"Golf Bag", 120.00, "Sports"),
        ProductData(105,"Running Shoes", 85.00, "Apparel"),ProductData(106,"Socks", 12.00, "Apparel"),
        ProductData(107,"T-Shirt", 15.00, "Apparel"),ProductData(108,"Shorts", 20.00, "Apparel"),
        ProductData(109,"Sweatpants", 25.00, "Apparel"),ProductData(110,"Hoodie", 35.00, "Apparel"),
        ProductData(111,"Jacket", 55.00, "Apparel"),ProductData(112,"Hat", 15.00, "Apparel"),
        ProductData(113,"Sunglasses", 45.00, "Apparel"),ProductData(114,"Watch", 120.00, "Apparel"),
        ProductData(115,"Belt", 20.00, "Apparel"),ProductData(116,"Jeans", 45.00, "Apparel"),
        ProductData(117,"Dress Shirt", 35.00, "Apparel"),ProductData(118,"Tie", 15.00, "Apparel"),
        ProductData(119,"Suit", 250.00, "Apparel"),ProductData(120,"Dress Shoes", 85.00, "Apparel"),
        ProductData(121,"Sneakers", 65.00, "Apparel"),ProductData(122,"Boots", 95.00, "Apparel"),
        ProductData(123,"Sandals", 25.00, "Apparel"),ProductData(124,"Swimsuit", 35.00, "Apparel"),
        ProductData(125,"Towel", 15.00, "Home"),ProductData(126,"Bed Sheets", 45.00, "Home"),
        ProductData(127,"Pillow", 20.00, "Home"),ProductData(128,"Blanket", 35.00, "Home"),
        ProductData(129,"Comforter", 65.00, "Home"),ProductData(130,"Curtains", 45.00, "Home"),
        ProductData(131,"Rug", 85.00, "Home"),ProductData(132,"Vase", 25.00, "Home"),
        ProductData(133,"Picture Frame", 15.00, "Home"),ProductData(134,"Candle", 12.00, "Home"),
        ProductData(135,"Clock", 25.00, "Home"),ProductData(136,"Mirror", 45.00, "Home"),
        ProductData(137,"Sofa", 600.00, "Furniture"),ProductData(138,"Coffee Table", 150.00, "Furniture"),
        ProductData(139,"End Table", 85.00, "Furniture"),ProductData(140,"Bookshelf", 120.00, "Furniture"),
        ProductData(141,"TV Stand", 180.00, "Furniture"),ProductData(142,"Dining Table", 400.00, "Furniture"),
        ProductData(143,"Dining Chair", 85.00, "Furniture"),ProductData(144,"Bed Frame", 300.00, "Furniture"),
        ProductData(145,"Mattress", 500.00, "Furniture"),ProductData(146,"Nightstand", 95.00, "Furniture"),
        ProductData(147,"Dresser", 250.00, "Furniture"),ProductData(148,"Wardrobe", 350.00, "Furniture"),
        ProductData(149,"Oven", 800.00, "Appliances"),ProductData(150,"Refrigerator", 1200.00, "Appliances"),
        ProductData(151,"Microwave", 150.00, "Appliances"),ProductData(152,"Dishwasher", 600.00, "Appliances"),
        ProductData(153,"Washing Machine", 700.00, "Appliances"),ProductData(154,"Dryer", 650.00, "Appliances"),
        ProductData(155,"Vacuum Cleaner", 200.00, "Appliances"),ProductData(156,"Blender", 45.00, "Appliances"),
        ProductData(157,"Toaster", 35.00, "Appliances"),ProductData(158,"Coffee Maker", 65.00, "Appliances"),
        ProductData(159,"Kettle", 25.00, "Appliances"),ProductData(160,"Food Processor", 85.00, "Appliances"),
        ProductData(161,"Mixer", 120.00, "Appliances"),ProductData(162,"Slow Cooker", 45.00, "Appliances"),
        ProductData(163,"Rice Cooker", 35.00, "Appliances"),ProductData(164,"Air Fryer", 85.00, "Appliances"),
        ProductData(165,"Pots and Pans Set", 150.00, "Kitchen"),ProductData(166,"Frying Pan", 35.00, "Kitchen"),
        ProductData(167,"Saucepan", 25.00, "Kitchen"),ProductData(168,"Baking Sheet", 15.00, "Kitchen"),
        ProductData(169,"Mixing Bowls", 25.00, "Kitchen"),ProductData(170,"Measuring Cups", 12.00, "Kitchen"),
        ProductData(171,"Cutting Board", 18.00, "Kitchen"),ProductData(172,"Knife Set", 85.00, "Kitchen"),
        ProductData(173,"Chef's Knife", 45.00, "Kitchen"),ProductData(174,"Spatula", 8.00, "Kitchen"),
        ProductData(175,"Tongs", 10.00, "Kitchen"),ProductData(176,"Whisk", 8.00, "Kitchen"),
        ProductData(177,"Peeler", 6.00, "Kitchen"),ProductData(178,"Can Opener", 12.00, "Kitchen"),
        ProductData(179,"Corkscrew", 15.00, "Kitchen"),ProductData(180,"Plates", 45.00, "Kitchen"),
        ProductData(181,"Bowls", 35.00, "Kitchen"),ProductData(182,"Cups", 25.00, "Kitchen"),
        ProductData(183,"Mugs", 20.00, "Kitchen"),ProductData(184,"Silverware Set", 55.00, "Kitchen"),
        ProductData(185,"Shampoo", 8.00, "Personal Care"),ProductData(186,"Conditioner", 8.00, "Personal Care"),
        ProductData(187,"Body Wash", 6.00, "Personal Care"),ProductData(188,"Soap", 4.00, "Personal Care"),
        ProductData(189,"Toothbrush", 3.00, "Personal Care"),ProductData(190,"Toothpaste", 4.00, "Personal Care"),
        ProductData(191,"Mouthwash", 6.00, "Personal Care"),ProductData(192,"Deodorant", 5.00, "Personal Care"),
        ProductData(193,"Lotion", 8.00, "Personal Care"),ProductData(194,"Sunscreen", 12.00, "Personal Care"),
        ProductData(195,"Razors", 15.00, "Personal Care"),ProductData(196,"Shaving Cream", 5.00, "Personal Care"),
        ProductData(197,"Hair Gel", 7.00, "Personal Care"),ProductData(198,"Hairspray", 6.00, "Personal Care"),
        ProductData(199,"Perfume", 65.00, "Personal Care"),ProductData(200,"Cologne", 55.00, "Personal Care")
    ]
  return db


class order_processor:
  def __init__(self, tax_rate=0.05):
    self.tax_rate=tax_rate
    self.orders=[]
  def create_order(self, customer_name,items_dict):
    order_id=len(self.orders)+1
    subtotal=0.0
    for prod,qty in items_dict.items():
      subtotal+=prod.price*qty
    tax=subtotal*self.tax_rate
    total=subtotal+tax
    order_data={'id':order_id,'customer':customer_name,'items':items_dict,'subtotal':subtotal,'tax':tax,'total':total,'status':'PENDING'}
    self.orders.append(order_data)
    return order_id
  def process_order( self, order_id ):
    for o in self.orders:
      if o['id']== order_id:
        o['status']='COMPLETED'
        return True
    return False


def calculate_complex_statistics( inventory ):
  categories=defaultdict(int)
  price_ranges={'low':0,'medium':0,'high':0}
  total_items=0
  for pid,data in inventory.items.items():
    cat = data['product'].category
    categories[cat]+=data['qty']
    total_items+=data['qty']
    prc=data['product'].price
    if prc<50:
      price_ranges['low']+=1
    elif prc<200:
      price_ranges['medium']+=1
    else:
      price_ranges['high']+=1
  return categories, price_ranges,total_items


class UserAccount:
  def __init__( self, username , email, role="customer"):
    self.username=username
    self.email= email
    self.role=role
    self.is_active= True
    self.login_attempts= 0
  def authenticate( self, password ):
    if password == "password123":
      self.login_attempts=0
      return True
    else:
      self.login_attempts+=1
      if self.login_attempts> 3:
        self.is_active=False
      return False
  def reset_password(self, old_pass,new_pass):
    if self.authenticate(old_pass):
      print("Password reset successful")
      return True
    return False


def run_simulation():
  print("Starting Simulation...")
  db=generate_mock_database()
  inv=inventory_manager()
  for i in range(0,len(db),2):
    inv.add_item(db[i], int(math.fmod(i*17, 50))+1 )
  print("Inventory initialized with",len(inv.items),"unique products.")
  proc=order_processor(tax_rate=0.08)
  cart1={db[0]:1,db[5]:2,db[10]:1}
  cart2={db[100]:1,db[150]:1}
  oid1=proc.create_order("Alice",cart1)
  oid2=proc.create_order("Bob",cart2)
  print("Order 1 Total:",proc.orders[0]['total'])
  print("Order 2 Total:",proc.orders[1]['total'])
  proc.process_order(oid1)
  cats,ranges,tot=calculate_complex_statistics(inv)
  print("Category distribution:")
  print(cats)
  print("Price ranges:")
  print(ranges)
  print("Total items in stock:",tot)


class ReportGenerator:
  def __init__(self, inventory):
    self.inventory = inventory
  def generate_html_report(self):
    html = "<html><head><title>Inventory Report</title></head><body>"
    html += "<h1>Current Inventory</h1>"
    html += "<table border='1'><tr><th>ID</th><th>Name</th><th>Category</th><th>Price</th><th>Quantity</th></tr>"
    for pid, data in self.inventory.items.items():
      p = data['product']
      q = data['qty']
      html += f"<tr><td>{p.id}</td><td>{p.name}</td><td>{p.category}</td><td>${p.price:.2f}</td><td>{q}</td></tr>"
    html += "</table></body></html>"
    return html
  def generate_csv_report(self):
    csv_data = "ID,Name,Category,Price,Quantity\n"
    for pid, data in self.inventory.items.items():
      p = data['product']
      q = data['qty']
      csv_data += f"{p.id},{p.name},{p.category},{p.price},{q}\n"
    return csv_data


def string_utility_test():
  test_str="   This is a   very poorly formatted    string.  "
  clean_str=re.sub(' +', ' ', test_str.strip())
  words=clean_str.split(' ')
  rev_words=words[::-1]
  return " ".join(rev_words)


class MatrixMathOperations:
  def __init__(self, matrix):
    self.matrix = matrix
  def transpose(self):
    result = [[self.matrix[j][i] for j in range(len(self.matrix))] for i in range(len(self.matrix[0]))]
    return result
  def flatten(self):
    return list(chain.from_iterable(self.matrix))
  def scalar_multiply(self, scalar):
    return [[cell * scalar for cell in row] for row in self.matrix]


def perform_heavy_math():
  mat = MatrixMathOperations([[1, 2, 3], [4, 5, 6], [7, 8, 9]])
  trans = mat.transpose()
  flat = mat.flatten()
  scaled = mat.scalar_multiply(5)
  return trans, flat, scaled


class DataValidator:
  @staticmethod
  def validate_email(email):
    pattern = r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$"
    return re.match(pattern, email) is not None
  @staticmethod
  def validate_phone(phone):
    pattern = r"^\+?\d{10,15}$"
    return re.match(pattern, phone) is not None
  @staticmethod
  def validate_age(age):
    try:
      a = int(age)
      return 0 <= a <= 120
    except:
      ValueError


def run_tests():
  print("Testing string util:", string_utility_test())
  t, f, s = perform_heavy_math()
  print("Matrix transpose:", t)
  print("Matrix flattened:", f)
  print("Matrix scaled:", s)
  v1 = DataValidator.validate_email("test@example.com")
  v2 = DataValidator.validate_phone("+12345678901")
  v3 = DataValidator.validate_age("25")
  print("Validations:", v1, v2, v3)


class Node:
  def __init__(self, data):
    self.data = data
    self.next = None
    self.prev = None


class DoublyLinkedList:
  def __init__(self):
    self.head = None
    self.tail = None
  def append(self, data):
    new_node = Node(data)
    if self.head is None:
      self.head = new_node
      self.tail = new_node
    else:
      self.tail.next = new_node
      new_node.prev = self.tail
      self.tail = new_node
  def prepend(self, data):
    new_node = Node(data)
    if self.head is None:
      self.head = new_node
      self.tail = new_node
    else:
      new_node.next = self.head
      self.head.prev = new_node
      self.head = new_node
  def delete(self, data):
    current = self.head
    while current:
      if current.data == data:
        if current.prev:
          current.prev.next = current.next
        else:
          self.head = current.next
        if current.next:
          current.next.prev = current.prev
        else:
          self.tail = current.prev
        return True
      current = current.next
    return False
  def display_forward(self):
    elements = []
    current = self.head
    while current:
      elements.append(current.data)
      current = current.next
    return elements
  def display_backward(self):
    elements = []
    current = self.tail
    while current:
      elements.append(current.data)
      current = current.prev
    return elements


def test_linked_list():
  dll = DoublyLinkedList()
  for i in range(1, 11):
    dll.append(i)
  dll.prepend(0)
  dll.delete(5)
  print("Forward:", dll.display_forward())
  print("Backward:", dll.display_backward())


class GenericTree:
  def __init__(self, root_data):
    self.root = {'data': root_data, 'children': []}
  def add_child(self, parent_data, child_data):
    parent_node = self._find_node(self.root, parent_data)
    if parent_node:
      parent_node['children'].append({'data': child_data, 'children': []})
      return True
    return False
  def _find_node(self, current_node, target_data):
    if current_node['data'] == target_data:
      return current_node
    for child in current_node['children']:
      result = self._find_node(child, target_data)
      if result:
        return result
    return None
  def traverse(self, node=None, depth=0):
    if node is None:
      node = self.root
    print("  " * depth + str(node['data']))
    for child in node['children']:
      self.traverse(child, depth + 1)


def test_tree():
  tree = GenericTree("CEO")
  tree.add_child("CEO", "CTO")
  tree.add_child("CEO", "CFO")
  tree.add_child("CTO", "Dev Manager")
  tree.add_child("CTO", "QA Manager")
  tree.add_child("CFO", "Accountant")
  tree.traverse()


class SimpleGraph:
  def __init__(self):
    self.graph = defaultdict(list)
  def add_edge(self, u, v):
    self.graph[u].append(v)
    self.graph[v].append(u)
  def bfs(self, start):
    visited = set()
    queue = [start]
    visited.add(start)
    result = []
    while queue:
      vertex = queue.pop(0)
      result.append(vertex)
      for neighbor in self.graph[vertex]:
        if neighbor not in visited:
          visited.add(neighbor)
          queue.append(neighbor)
    return result
  def dfs(self, start, visited=None):
    if visited is None:
      visited = set()
    visited.add(start)
    result = [start]
    for neighbor in self.graph[start]:
      if neighbor not in visited:
        result.extend(self.dfs(neighbor, visited))
    return result


def test_graph():
  g = SimpleGraph()
  edges = [(0, 1), (0, 2), (1, 2), (2, 0), (2, 3), (3, 3)]
  for u, v in edges:
    g.add_edge(u, v)
  print("BFS:", g.bfs(2))
  print("DFS:", g.dfs(2))


def complex_string_manipulation(text):
  vowels = "aeiouAEIOU"
  consonants = "bcdfghjklmnpqrstvwxyzBCDFGHJKLMNPQRSTVWXYZ"
  v_count = sum(1 for char in text if char in vowels)
  c_count = sum(1 for char in text if char in consonants)
  words = text.split()
  longest_word = max(words, key=len) if words else ""
  reversed_text = text[::-1]
  camel_case = "".join(word.capitalize() for word in words)
  snake_case = "_".join(word.lower() for word in words)
  kebab_case = "-".join(word.lower() for word in words)
  return {
    "vowels": v_count,
    "consonants": c_count,
    "longest": longest_word,
    "reversed": reversed_text,
    "camel": camel_case,
    "snake": snake_case,
    "kebab": kebab_case,
}


def test_string_manipulation():
  result = complex_string_manipulation("The quick brown fox jumps over the lazy dog")
  for k, v in result.items():
    print(f"{k}: {v}")


class ConfigurationManager:
  _instance = None
  def __new__(cls):
    if cls._instance is None:
      cls._instance = super(ConfigurationManager, cls).__new__(cls)
      cls._instance.config = {}
    return cls._instance
  def set(self, key, value):
    self.config[key] = value
  def get(self, key, default=None):
    return self.config.get(key, default)
  def load_from_json(self, json_string):
    try:
      data = json.loads(json_string)
      self.config.update(data)
      return True
    except:
      json.JSONDecodeError
  def export_to_json(self):
    return json.dumps(self.config, indent=4)


def test_config_manager():
  config1 = ConfigurationManager()
  config1.set("theme", "dark")
  config1.set("language", "en-US")
  config2 = ConfigurationManager()
  print("Config identical:", config1 is config2)
  print("Theme:", config2.get("theme"))
  json_str = '{"timeout": 30, "retries": 3}'
  config2.load_from_json(json_str)
  print("Exported JSON:\n", config1.export_to_json())


class EventDispatcher:
  def __init__(self):
    self.listeners = defaultdict(list)
  def subscribe(self, event_type, listener):
    self.listeners[event_type].append(listener)
  def unsubscribe(self, event_type, listener):
    if listener in self.listeners[event_type]:
      self.listeners[event_type].remove(listener)
  def dispatch(self, event_type, data):
    for listener in self.listeners[event_type]:
      listener(data)


def test_event_dispatcher():
  dispatcher = EventDispatcher()
  def on_user_login(data):
    print(f"User logged in: {data['username']}")
  def on_user_logout(data):
    print(f"User logged out: {data['username']}")
  def log_event(data):
    print(f"Event logged: {data}")
  dispatcher.subscribe("login", on_user_login)
  dispatcher.subscribe("login", log_event)
  dispatcher.subscribe("logout", on_user_logout)
  dispatcher.dispatch("login", {"username": "admin", "ip": "192.168.1.1"})
  dispatcher.dispatch("logout", {"username": "admin"})


def main():
  print("System Initialization")
  run_simulation()
  run_tests()
  test_linked_list()
  test_tree()
  test_graph()
  test_string_manipulation()
  test_config_manager()
  test_event_dispatcher()
  print("Execution Completed.")
if __name__== "__main__":
  main()
