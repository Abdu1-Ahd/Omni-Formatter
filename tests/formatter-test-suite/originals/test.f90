! ── CASE 1: Module declaration ────────────────────────────────────────────
MODULE test_module
  IMPLICIT NONE

  ! ── CASE 2: Parameter and type declarations ──────────────────────────
  INTEGER, PARAMETER :: dp = KIND(1.0d0)
  INTEGER, PARAMETER :: MAX_SIZE = 1000
  REAL(dp), PARAMETER :: PI = 3.14159265358979323846_dp

  ! ── CASE 3: Derived type ──────────────────────────────────────────────
  TYPE :: User
    INTEGER :: id
    CHARACTER(LEN=64) :: name
    CHARACTER(LEN=128) :: email
    INTEGER :: age = 0
  END TYPE User

CONTAINS

  ! ── CASE 4: Functions ─────────────────────────────────────────────────
  FUNCTION greet(name) RESULT(msg)
    CHARACTER(LEN=*), INTENT(IN) :: name
    CHARACTER(LEN=100) :: msg
    msg = "Hello, " // TRIM(name) // "!"
  END FUNCTION greet

  FUNCTION area_circle(radius) RESULT(a)
    REAL(dp), INTENT(IN) :: radius
    REAL(dp) :: a
    a = PI * radius * radius
  END FUNCTION area_circle

  ! ── CASE 5: Subroutine ────────────────────────────────────────────────
  SUBROUTINE process_array(arr, n, result)
    INTEGER, INTENT(IN) :: n
    REAL(dp), INTENT(IN) :: arr(n)
    REAL(dp), INTENT(OUT) :: result
    INTEGER :: i
    result = 0.0_dp
    DO i = 1, n
      result = result + arr(i)
    END DO
    result = result / REAL(n, dp)
  END SUBROUTINE process_array

END MODULE test_module

! ── CASE 6: Main program ──────────────────────────────────────────────────
PROGRAM main
  USE test_module
  IMPLICIT NONE

  TYPE(User) :: u
  REAL(dp) :: data(5) = [1.0_dp, 2.0_dp, 3.0_dp, 4.0_dp, 5.0_dp]
  REAL(dp) :: avg
  CHARACTER(LEN=100) :: msg

  u%id = 1
  u%name = "Alice"
  u%email = "alice@example.com"
  u%age = 30

  msg = greet(u%name)
  WRITE(*,*) TRIM(msg)

  CALL process_array(data, 5, avg)
  WRITE(*,'(A,F6.2)') "Average: ", avg

END PROGRAM main
