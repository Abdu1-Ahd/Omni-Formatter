# frozen_string_literal: true

require 'json'
require 'date'

# ── CASE 1: Class definition — mixed indentation ──────────────────────────
class User
  attr_accessor :id , :name , :email
  attr_reader   :created_at

  def initialize ( id , name , email )
    @id         = id
    @name       = name
    @email      = email
    @created_at = Date.today
  end

  def to_s
    "User(#{@id}, #{@name}, #{@email})"
  end

  def valid?
    !@name.nil? && !@email.nil? && @email.include?('@')
  end
end

# ── CASE 2: Module with methods ───────────────────────────────────────────
module Greetable
  def greet
    "Hello, I'm #{name}!"
  end

  def farewell
    "Goodbye from #{name}!"
  end
end

class Employee < User
  include Greetable

  attr_accessor :department , :salary

  def initialize(id, name, email, department, salary)
    super(id, name, email)
    @department = department
    @salary     = salary
  end
end

# ── CASE 3: Symbols and hashes ────────────────────────────────────────────
config = {
  host:   'localhost',
  port:      5432,
  database: 'mydb',
  pool:  5,
}

old_hash = {
  :host => 'localhost',
  :port => 5432,
}

# ── CASE 4: Blocks, procs, lambdas ────────────────────────────────────────
numbers = [1,2,3,4,5,6,7,8,9,10]
evens   = numbers.select{|n| n.even?}
doubled = numbers.map { |n| n * 2 }
sum     = numbers.reduce(0) { |acc, n| acc + n }

double  = ->(x) { x * 2 }
square  = proc { |x| x ** 2 }

# ── CASE 5: String operations ─────────────────────────────────────────────
name = 'world'
puts "Hello, #{name}!"
puts 'Single quote: no interpolation'
puts name.upcase.reverse.split('').join('-')

# ── CASE 6: Exceptions ────────────────────────────────────────────────────
def safe_divide(a, b)
  raise ArgumentError, 'Division by zero' if b.zero?
  a / b
rescue ZeroDivisionError => e
  puts "Error: #{e.message}"
  nil
ensure
  puts 'Division attempted'
end

# ── CASE 7: Long line ─────────────────────────────────────────────────────
def very_long_method_name_that_exceeds_line_width(parameter_one, parameter_two, parameter_three, parameter_four = nil)
  [parameter_one, parameter_two, parameter_three, parameter_four].compact.join(', ')
end

# ── CASE 8: Trailing whitespace ───────────────────────────────────────────
def trailing_example   
  x = 1   
  y = 2  
  x + y
end
