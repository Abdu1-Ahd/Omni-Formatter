// ── CASE 1: Package and imports ────────────────────────────────────────────
package com.example.myapp;

import java.util.*;
import java.util.stream.*;

// ── CASE 2: Sealed class hierarchy (Scala 3 style) ────────────────────────
sealed trait Shape
case class Circle(radius: Double) extends Shape
case class Rectangle(width: Double, height: Double) extends Shape
case class Triangle(a: Double, b: Double, c: Double) extends Shape

// ── CASE 3: Object (singleton) and companion ──────────────────────────────
object ShapeUtils {
  def area(shape: Shape): Double = shape match {
    case Circle(r)        => math.Pi * r * r
    case Rectangle(w, h)  => w * h
    case Triangle(a,b,c)  =>
      val s = (a+b+c)/2
      math.sqrt(s*(s-a)*(s-b)*(s-c))
  }

  def classify(n: Int): String = n match {
    case 0                    => "zero"
    case n if n < 0           => "negative"
    case n if n < 10          => "small"
    case _                    => "large"
  }
}

// ── CASE 4: Case class with methods ────────────────────────────────────────
case class User(
  id: Int,
  name: String,
  email: String,
  age: Int = 0
) {
  def greet: String = s"Hello, $name!"
  def isAdult: Boolean = age >= 18
  def copy(newName: String): User = this.copy(name = newName)
}

// ── CASE 5: Higher-order functions and for-comprehension ──────────────────
object Processing {
  def processUsers(users: List[User]): List[String] =
    for {
      user  <- users
      if user.isAdult
      if user.email.contains("@")
    } yield user.name.toUpperCase

  def pipeline(numbers: List[Int]): Int =
    numbers
      .filter(_ % 2 == 0)
      .map(_ * _)
      .foldLeft(0)(_ + _)
}

// ── CASE 6: Traits and mixins ─────────────────────────────────────────────
trait Loggable {
  def log(msg: String): Unit = println(s"[LOG] $msg")
}

trait Validatable[T] {
  def validate(t: T): Either[String, T]
}

class UserService extends Loggable with Validatable[User] {
  def validate(user: User): Either[String, User] =
    if (user.name.isEmpty) Left("Name is required")
    else if (!user.email.contains("@")) Left("Invalid email")
    else Right(user)
}

// ── CASE 7: Long signature ────────────────────────────────────────────────
def processWithVeryLongFunctionNameExceedingLineWidth(inputList: List[User], transform: User => User, predicate: User => Boolean): List[User] =
  inputList.filter(predicate).map(transform)
