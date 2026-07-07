; ── CASE 1: Variables and constants ──────────────────────────────────────
#NoEnv
#SingleInstance Force
#Persistent

global AppName  := "TestApp"
global Version  :=  "1.0.0"
global IsRunning :=  false
LogFile := A_ScriptDir . "\app.log"

; ── CASE 2: Hotkeys ───────────────────────────────────────────────────────
^!r::
    Reload
Return

^!q::
    ExitApp
Return

; ── CASE 3: Functions ────────────────────────────────────────────────────
Greet(name, greeting := "Hello") {
    return greeting . ", " . name . "!"
}

LogMessage(message, level := "INFO") {
    global LogFile
    timestamp := A_Now
    FormatTime, ts, %timestamp%, yyyy-MM-dd HH:mm:ss
    entry := "[" . ts . "] [" . level . "] " . message
    FileAppend, %entry%`n, %LogFile%
}

; ── CASE 4: Labels and GoSub ─────────────────────────────────────────────
InitApp:
    LogMessage("Application starting...")
    IsRunning := true
Return

CleanUp:
    LogMessage("Cleaning up...")
    IsRunning := false
Return

; ── CASE 5: Loops ────────────────────────────────────────────────────────
ProcessFiles:
    Loop, Files, %A_ScriptDir%\*.log
    {
        LogMessage("Processing: " . A_LoopFilePath)
        FileRead, content, %A_LoopFilePath%
        StringLen, len, content
        LogMessage("Length: " . len)
    }
Return

; ── CASE 6: String operations ────────────────────────────────────────────
FormatOutput:
    name := "alice"
    StringUpper, upper, name
    StringLower, lower, name
    StringLen, length, name
    msg := upper . " (" . length . " chars)"
    MsgBox, %msg%
Return

; ── CASE 7: GUI creation ──────────────────────────────────────────────────
ShowGui:
    Gui, New, , %AppName%
    Gui, Add, Text,, Enter your name:
    Gui, Add, Edit, vUserName w200
    Gui, Add, Button, Default, Submit
    Gui, Show
Return

ButtonSubmit:
    Gui, Submit
    MsgBox, % Greet(UserName)
Return
