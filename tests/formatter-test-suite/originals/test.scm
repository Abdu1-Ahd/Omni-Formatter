; ── CASE 1: Basic Scheme ──────────────────────────────────────────────────
(define (square x) (* x x))
(define (cube   x) (* x x x))
(define (average a b) (/ (+ a b) 2))

; ── CASE 2: Recursion and tail-call ───────────────────────────────────────
(define (factorial n)
  (define (iter acc n)
    (if (= n 0)
        acc
        (iter (* acc n) (- n 1))))
  (iter 1 n))

(define (fibonacci n)
  (cond ((= n 0) 0)
        ((= n 1) 1)
        (else (+ (fibonacci (- n 1))
                 (fibonacci (- n 2))))))

; ── CASE 3: Higher-order functions ────────────────────────────────────────
(define (my-map f lst)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (my-map f (cdr lst)))))

(define (my-filter pred lst)
  (cond ((null? lst) '())
        ((pred (car lst)) (cons (car lst) (my-filter pred (cdr lst))))
        (else (my-filter pred (cdr lst)))))

(define (my-fold-left f init lst)
  (if (null? lst)
      init
      (my-fold-left f (f init (car lst)) (cdr lst))))

; ── CASE 4: Let and lambda ────────────────────────────────────────────────
(define (make-adder n)
  (lambda (x) (+ x n)))

(define (compute x y)
  (let* ((sum  (+ x y))
         (diff (- x y))
         (prod (* x y)))
    (list sum diff prod)))

; ── CASE 5: Continuations ────────────────────────────────────────────────
(define (find-first pred lst)
  (call/cc
   (lambda (return)
     (for-each (lambda (x)
                 (when (pred x) (return x)))
               lst)
     #f)))

; ── CASE 6: Quasiquote ───────────────────────────────────────────────────
(define (make-greet name)
  `(lambda () (display ,(string-append "Hello, " name "!"))))

; ── CASE 7: Main ─────────────────────────────────────────────────────────
(display (factorial 10))
(newline)
(display (my-map square '(1 2 3 4 5)))
(newline)
