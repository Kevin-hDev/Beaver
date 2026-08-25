function Resolve-BoundedPackagePath([string]$Path, [string]$AllowedRoot = "") {
    if (
        [string]::IsNullOrWhiteSpace($Path) -or $Path.Length -gt 4096 -or
        $Path -match "[\x00-\x1f]" -or $AllowedRoot.Length -gt 4096 -or
        $AllowedRoot -match "[\x00-\x1f]"
    ) {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }
    try {
        $resolved = [IO.Path]::GetFullPath(
            $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
        )
        if (-not [string]::IsNullOrWhiteSpace($AllowedRoot)) {
            $allowed = [IO.Path]::GetFullPath(
                $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($AllowedRoot)
            )
            $allowedVolume = [IO.Path]::GetPathRoot($allowed)
            if ($allowed -cne $allowedVolume) {
                $allowed = $allowed.TrimEnd(
                    [IO.Path]::DirectorySeparatorChar,
                    [IO.Path]::AltDirectorySeparatorChar
                )
            }
            $allowedPrefix = if ($allowed.EndsWith([IO.Path]::DirectorySeparatorChar)) {
                $allowed
            } else {
                $allowed + [IO.Path]::DirectorySeparatorChar
            }
            if (-not $resolved.StartsWith($allowedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                throw (New-Object IO.InvalidDataException("Invalid package file."))
            }
        }

        # Inspect from the volume root so a junction above AllowedRoot cannot hide.
        $volumeRoot = [IO.Path]::GetPathRoot($resolved)
        if ([string]::IsNullOrWhiteSpace($volumeRoot)) {
            throw (New-Object IO.InvalidDataException("Invalid package file."))
        }
        $rootItem = Get-Item -LiteralPath $volumeRoot -Force -ErrorAction Stop
        if (
            -not $rootItem.PSIsContainer -or
            ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        ) {
            throw (New-Object IO.InvalidDataException("Invalid package file."))
        }
        $segments = @($resolved.Substring($volumeRoot.Length).Split(
            [char[]]@(
                [IO.Path]::DirectorySeparatorChar,
                [IO.Path]::AltDirectorySeparatorChar
            ),
            [StringSplitOptions]::RemoveEmptyEntries
        ))
        if ($segments.Count -le 0 -or $segments.Count -gt 256) {
            throw (New-Object IO.InvalidDataException("Invalid package file."))
        }
        $current = $volumeRoot
        for ($index = 0; $index -lt ($segments.Count - 1); $index += 1) {
            $current = Join-Path $current $segments[$index]
            $directory = Get-Item -LiteralPath $current -Force -ErrorAction Stop
            if (
                -not $directory.PSIsContainer -or
                ($directory.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
            ) {
                throw (New-Object IO.InvalidDataException("Invalid package file."))
            }
        }
        return $resolved
    } catch {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }
}

function Get-BoundedPackageFile(
    [string]$Path,
    [long]$MaxBytes,
    [string]$AllowedRoot = ""
) {
    if ($MaxBytes -le 0) {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }
    try {
        $resolved = Resolve-BoundedPackagePath $Path $AllowedRoot
        $item = Get-Item -LiteralPath $resolved -Force -ErrorAction Stop
        $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if ($item.PSIsContainer -or $isLink -or $item.Length -le 0 -or $item.Length -gt $MaxBytes) {
            throw (New-Object IO.InvalidDataException("Invalid package file."))
        }
        return $item
    } catch {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }
}

function Read-BoundedPackageBytes(
    [string]$Path,
    [long]$MaxBytes,
    [string]$AllowedRoot = ""
) {
    if ($MaxBytes -le 0 -or $MaxBytes -ge [int]::MaxValue) {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }
    $stream = $null
    $memory = $null
    try {
        $resolved = Resolve-BoundedPackagePath $Path $AllowedRoot
        $item = Get-Item -LiteralPath $resolved -Force -ErrorAction Stop
        if (
            $item.PSIsContainer -or
            ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        ) {
            throw (New-Object IO.InvalidDataException("Invalid package file."))
        }
        $stream = New-Object IO.FileStream(
            $resolved,
            [IO.FileMode]::Open,
            [IO.FileAccess]::Read,
            [IO.FileShare]::Read
        )
        $memory = New-Object IO.MemoryStream
        $buffer = New-Object byte[] 81920
        while ($memory.Length -le $MaxBytes) {
            $remaining = [int][Math]::Min($buffer.Length, ($MaxBytes + 1) - $memory.Length)
            $read = $stream.Read($buffer, 0, $remaining)
            if ($read -eq 0) { break }
            $memory.Write($buffer, 0, $read)
        }
        if ($memory.Length -le 0 -or $memory.Length -gt $MaxBytes) {
            throw (New-Object IO.InvalidDataException("Invalid package file."))
        }
        return ,$memory.ToArray()
    } catch {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    } finally {
        if ($null -ne $memory) { $memory.Dispose() }
        if ($null -ne $stream) { $stream.Dispose() }
    }
}

function Read-BoundedPackageText(
    [string]$Path,
    [long]$MaxBytes,
    [string]$AllowedRoot = ""
) {
    try {
        $bytes = [byte[]](Read-BoundedPackageBytes $Path $MaxBytes $AllowedRoot)
        $memory = New-Object IO.MemoryStream(,$bytes)
        try {
            $reader = New-Object IO.StreamReader($memory, [Text.Encoding]::UTF8, $true)
            try { return $reader.ReadToEnd() } finally { $reader.Dispose() }
        } finally {
            $memory.Dispose()
        }
    } catch {
        throw (New-Object IO.InvalidDataException("Invalid package file."))
    }
}

function Test-BoundedPackageFile(
    [string]$Path,
    [long]$MaxBytes,
    [string]$AllowedRoot = ""
) {
    try {
        [void](Get-BoundedPackageFile $Path $MaxBytes $AllowedRoot)
        return $true
    } catch {
        return $false
    }
}
