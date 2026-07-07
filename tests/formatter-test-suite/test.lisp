; ── CASE 1: Basic Common Lisp definitions ────────────────────────────────
(defpackage :myapp
  (:use :cl)
  (:export :user :make-user :user-name :greet))

(in-package :myapp)

; ── CASE 2: Struct and constructor ────────────────────────────────────────
(defstruct user
  (id 0 :type integer)
  (name "" :type string)
  (email "" :type string)
  (age 0 :type (integer 0)))

(defun make-person (id name email &optional (age 0))
  (make-user :id id :name name :email email :age age))

; ── CASE 3: Generic functions ──────────────────────────────────────────────
(defgeneric greet (obj)
  (:documentation "Returns a greeting string for obj"))

(defmethod greet ((u user))
  (format nil "Hello, ~a!" (user-name u)))

(defmethod greet ((name string))
  (format nil "Hello, ~a!" name))

; ── CASE 4: Higher-order functions ────────────────────────────────────────
(defun my-map (f lst)
  (if (null lst)
      nil
      (cons (funcall f (car lst))
            (my-map f (cdr lst)))))

(defun compose (&rest fns)
  (lambda (x)
    (reduce #'funcall fns :initial-value x :from-end t)))

; ── CASE 5: Macros ────────────────────────────────────────────────────────
(defmacro when-valid (expr &body body)
  `(when (valid-p ,expr)
     ,@body))

(defmacro with-logging (level &body forms)
  `(progn
     (format t "[~a] Starting~%" ,level)
     ,@forms
     (format t "[~a] Done~%" ,level)))

; ── CASE 6: Let forms ─────────────────────────────────────────────────────
(defun compute (x y)
  (let* ((sum  (+ x y))
         (diff (- x y))
         (prod (* x y))
         (quot (if (zerop y) nil (/ x y))))
    (list :sum sum :diff diff :product prod :quotient quot)))

; ── CASE 7: Main entry ────────────────────────────────────────────────────
(defun main ()
  (let ((u (make-person 1 "Alice" "alice@example.com" 30)))
    (format t "~a~%" (greet u))
    (format t "~a~%" (compute 10 3))))
