-- ── CASE 1: Simple Haskell module ────────────────────────────────────────
module Main where

import Data.List (sort, nub, group)
import Data.Char (toUpper, toLower, isAlpha)
import Data.Maybe (fromMaybe, mapMaybe)

-- ── CASE 2: Type declarations and data types ──────────────────────────────
data Shape
    = Circle Double
    | Rectangle Double Double
    | Triangle Double Double Double
    deriving (Show, Eq)

data Maybe' a = Nothing' | Just' a deriving (Show)

-- ── CASE 3: Type class instances ──────────────────────────────────────────
class Describable a where
    describe :: a -> String

instance Describable Shape where
    describe (Circle r)        = "Circle with radius " ++ show r
    describe (Rectangle w h)   = "Rectangle " ++ show w ++ "x" ++ show h
    describe (Triangle a b c)  = "Triangle " ++ show a ++ " " ++ show b ++ " " ++ show c

-- ── CASE 4: Functions — pattern matching and guards ───────────────────────
area :: Shape -> Double
area (Circle r)        = pi * r * r
area (Rectangle w h)   = w * h
area (Triangle a b c) = let s = (a + b + c) / 2
                          in sqrt (s * (s-a) * (s-b) * (s-c))

classify :: Int -> String
classify n
    | n < 0     = "negative"
    | n == 0    = "zero"
    | n < 10    = "single digit"
    | n < 100   = "double digit"
    | otherwise = "large"

-- ── CASE 5: Higher-order functions ────────────────────────────────────────
applyTwice :: (a -> a) -> a -> a
applyTwice f = f . f

myMap :: (a -> b) -> [a] -> [b]
myMap _ []     = []
myMap f (x:xs) = f x : myMap f xs

-- ── CASE 6: List comprehension ────────────────────────────────────────────
pythagorean :: Int -> [(Int, Int, Int)]
pythagorean n = [(a, b, c) | c <- [1..n], b <- [1..c], a <- [1..b], a^2 + b^2 == c^2]

-- ── CASE 7: main ──────────────────────────────────────────────────────────
main :: IO ()
main = do
    let shapes = [Circle 5, Rectangle 4 6, Triangle 3 4 5]
    mapM_ (putStrLn . describe) shapes
    putStrLn $ "Classify 42: " ++ classify 42
