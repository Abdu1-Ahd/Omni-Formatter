       IDENTIFICATION DIVISION.
       PROGRAM-ID. TEST-PROGRAM.
       AUTHOR. ALICE.
       DATE-WRITTEN. 2024-01-01.

      * ── CASE 1: DATA DIVISION — mixed indentation ────────────────────
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-PC.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-EMPLOYEE.
           05 WS-EMP-ID       PIC 9(5).
           05 WS-EMP-NAME     PIC X(30).
           05 WS-EMP-SALARY   PIC 9(7)V99.
           05 WS-EMP-DEPT     PIC X(20).
       01 WS-COUNTER          PIC 9(3)  VALUE 0.
       01 WS-TOTAL-SALARY     PIC 9(9)V99 VALUE 0.
       01 WS-MSG              PIC X(80).

      * ── CASE 2: FILE SECTION ───────────────────────────────────────
       FILE SECTION.
       FD EMPLOYEE-FILE.
       01 EMPLOYEE-RECORD.
           05 EMP-ID          PIC 9(5).
           05 EMP-NAME        PIC X(30).
           05 EMP-SALARY      PIC 9(7)V99.

      * ── CASE 3: PROCEDURE DIVISION ─────────────────────────────────
       PROCEDURE DIVISION.
       MAIN-PARAGRAPH.
           PERFORM INITIALIZE-DATA
           PERFORM PROCESS-EMPLOYEES UNTIL WS-COUNTER > 100
           PERFORM DISPLAY-SUMMARY
           STOP RUN.

       INITIALIZE-DATA.
           MOVE 0 TO WS-COUNTER
           MOVE 0 TO WS-TOTAL-SALARY
           MOVE SPACES TO WS-MSG.

       PROCESS-EMPLOYEES.
           ADD 1 TO WS-COUNTER
           READ EMPLOYEE-FILE INTO WS-EMPLOYEE
               AT END
                   MOVE 'EOF' TO WS-MSG
               NOT AT END
                   ADD WS-EMP-SALARY TO WS-TOTAL-SALARY
           END-READ.

       DISPLAY-SUMMARY.
           DISPLAY "Total employees: " WS-COUNTER
           DISPLAY "Total salary: " WS-TOTAL-SALARY.
