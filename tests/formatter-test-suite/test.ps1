# ── CASE 1: Variable declarations — spacing ───────────────────────────────
$Name = "Alice"
$Age  =  30
$LogFile = "C:\logs\app.log"
$IsAdmin=$false

# ── CASE 2: Functions — mixed indentation ─────────────────────────────────
function Get-UserGreeting {
    param(
        [string]$Name,
        [int]$Age = 0
    )

    if ($Age -gt 0) {
      return "Hello, $Name! You are $Age years old."
    }
    else {
        return "Hello, $Name!"
    }
}

function Write-Log {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory=$true)][string]$Message,
        [ValidateSet('INFO','WARN','ERROR')][string]$Level = 'INFO'
    )

    $timestamp = Get-Date -Format 'yyyy-MM-ddTHH:mm:ss'
    $entry = "[$timestamp] [$Level] $Message"
    Write-Output $entry
    Add-Content -Path $LogFile -Value $entry
}

# ── CASE 3: Pipeline operations ────────────────────────────────────────────
$processes = Get-Process |
    Where-Object { $_.CPU -gt 10 } |
    Sort-Object CPU -Descending |
        Select-Object -First 10 Name,CPU,WorkingSet

# ── CASE 4: Try/Catch/Finally ──────────────────────────────────────────────
try {
    $content = Get-Content -Path $LogFile -ErrorAction Stop
    $content | ForEach-Object { Write-Output $_ }
}
catch [System.IO.FileNotFoundException] {
    Write-Log -Message "File not found: $LogFile" -Level 'ERROR'
}
catch {
    Write-Log -Message "Unexpected error: $_" -Level 'ERROR'
}
finally {
    Write-Log -Message "Operation completed" -Level 'INFO'
}

# ── CASE 5: Switch statement ───────────────────────────────────────────────
switch ($Name) {
    'Alice'   { Write-Output "Found Alice" }
    'Bob'     { Write-Output "Found Bob" }
    default   { Write-Output "Unknown user" }
}

# ── CASE 6: Hashtable and splatting ───────────────────────────────────────
$params = @{
    Name    = 'Alice'
    Age     = 30
    IsAdmin = $true
}

# ── CASE 7: Long line ──────────────────────────────────────────────────────
$result = Get-ChildItem -Path "C:\Users" -Recurse -Filter "*.log" -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddDays(-7) }

# ── CASE 8: Trailing whitespace ────────────────────────────────────────────
Write-Output "hello"   
Write-Output "world"  
