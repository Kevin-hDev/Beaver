function Test-AssociatedIcon([string]$Path) {
    Add-Type -AssemblyName System.Drawing
    $icon = [Drawing.Icon]::ExtractAssociatedIcon($Path)
    if ($null -eq $icon) {
        Stop-Validation
    }
    try {
        if ($icon.Width -le 0 -or $icon.Height -le 0) {
            Stop-Validation
        }
    } finally {
        $icon.Dispose()
    }
}

function Test-ShortcutTarget([string]$Path, [string]$ExpectedTarget) {
    $shell = New-Object -ComObject WScript.Shell
    try {
        $target = $shell.CreateShortcut($Path).TargetPath
        return $target.Equals($ExpectedTarget, [StringComparison]::OrdinalIgnoreCase)
    } finally {
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell)
    }
}
