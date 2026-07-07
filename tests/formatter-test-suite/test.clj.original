; ── CASE 1: Basic Clojure — namespaces and requires ──────────────────────
(ns myapp.core
  (:require [clojure.string :as str]
            [clojure.set :as set]
            [clojure.core.async :refer [chan go <!]]))

; ── CASE 2: Function definitions ─────────────────────────────────────────
(defn greet [name]
  (str "Hello, " name "!"))

(defn classify [n]
  (cond
    (neg? n)    "negative"
    (zero? n)   "zero"
    (< n 10)    "small"
    :else        "large"))

; ── CASE 3: Data structures ───────────────────────────────────────────────
(def user {:id 1 :name "Alice" :email "alice@example.com" :age 30})
(def users [{:id 1 :name "Alice"}
            {:id 2 :name "Bob"}
            {:id 3 :name "Carol"}])

; ── CASE 4: Higher-order functions ───────────────────────────────────────
(defn process-users [users]
  (->> users
       (filter #(> (count (:name %)) 3))
       (map #(update % :name str/upper-case))
       (sort-by :id)))

; ── CASE 5: Macros and let ───────────────────────────────────────────────
(defmacro when-valid [expr & body]
  `(when (valid? ~expr)
     ~@body))

(defn compute [x y]
  (let [sum  (+ x y)
        diff (- x y)
        prod (* x y)]
    {:sum sum :diff diff :product prod}))

; ── CASE 6: Multimethods ─────────────────────────────────────────────────
(defmulti area :type)

(defmethod area :circle [shape]
  (* Math/PI (:radius shape) (:radius shape)))

(defmethod area :rectangle [shape]
  (* (:width shape) (:height shape)))

; ── CASE 7: Core.async ───────────────────────────────────────────────────
(defn async-example []
  (let [c (chan)]
    (go (>! c "hello"))
    (go (println (<! c)))))
