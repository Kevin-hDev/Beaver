function Get-BoundedPackageFile([string]$Path, [long]$MaxBytes) {
    if (
        [string]::IsNullOrWhiteSpace($Path) -or
        $Path.Length -gt 4096 -or
        $Path -match "[\x00-\x1f]" -or
        $MaxBytes -le 0
    ) {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }

    try {
        # PowerShell owns relative-path semantics; resolve once before every later check.
        $resolved = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
        $item = Get-Item -LiteralPath $resolved -Force
    } catch {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }

    $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
    if ($item.PSIsContainer -or $isLink -or $item.Length -le 0 -or $item.Length -gt $MaxBytes) {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }
    return $item
}

function Test-BoundedPackageFile([string]$Path, [long]$MaxBytes) {
    try {
        [void](Get-BoundedPackageFile $Path $MaxBytes)
        return $true
    } catch {
        return $false
    }
}
