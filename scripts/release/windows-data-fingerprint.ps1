[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Source,
    [string]$BackupDirectory = "",
    [ValidateRange(1, 20000)]
    [int]$MaxFiles = 20000,
    [ValidateRange(1, 1073741824)]
    [long]$MaxBytes = 1073741824
)

$ErrorActionPreference = "Stop"
$MaxEntries = 40000
$MaxPathLength = 4096

. (Join-Path $PSScriptRoot "windows-artifact-helpers.ps1")

function Stop-DataValidation {
    throw "Data validation failed."
}

function Get-DirectoryPrefix([string]$Path) {
    if ($Path.EndsWith("\")) { return $Path }
    return $Path + "\"
}

function Resolve-DataDirectory([string]$Path, [bool]$Create) {
    if (
        $Path.Length -gt $MaxPathLength -or
        -not (Test-FullyQualifiedWindowsPath $Path)
    ) {
        Stop-DataValidation
    }
    try {
        $full = [IO.Path]::GetFullPath($Path)
        if ($Create) { [void][IO.Directory]::CreateDirectory($full) }
        $item = Get-Item -LiteralPath $full -Force
        $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if (-not $item.PSIsContainer -or $isLink) { Stop-DataValidation }
        $root = [IO.Path]::GetPathRoot($item.FullName)
        if ($item.FullName.Length -le $root.Length) { return $root }
        return $item.FullName.TrimEnd("\")
    } catch {
        Stop-DataValidation
    }
}

function Get-VerifiedContentHash([string]$Path, [long]$ExpectedLength) {
    $stream = $null
    $sha = $null
    try {
        $item = Get-Item -LiteralPath $Path -Force
        $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if ($item.PSIsContainer -or $isLink -or $item.Length -ne $ExpectedLength) {
            Stop-DataValidation
        }
        $stream = [IO.File]::Open(
            $Path,
            [IO.FileMode]::Open,
            [IO.FileAccess]::Read,
            [IO.FileShare]::Read
        )
        if ($stream.Length -ne $ExpectedLength) { Stop-DataValidation }
        $sha = [Security.Cryptography.SHA256]::Create()
        $digest = $sha.ComputeHash($stream)
        return -join @($digest | ForEach-Object { $_.ToString("X2") })
    } catch {
        Stop-DataValidation
    } finally {
        if ($null -ne $sha) { $sha.Dispose() }
        if ($null -ne $stream) { $stream.Dispose() }
    }
}

function Copy-VerifiedDataFile($Entry, [string]$BackupRoot) {
    try {
        $segments = @($Entry.Relative.Split("/"))
        if ($segments.Count -eq 0 -or $segments.Count -gt 256) { Stop-DataValidation }
        foreach ($segment in $segments) {
            if (
                [string]::IsNullOrWhiteSpace($segment) -or
                $segment -eq "." -or
                $segment -eq ".." -or
                $segment.Length -gt 255
            ) {
                Stop-DataValidation
            }
        }
        $parent = $BackupRoot
        for ($index = 0; $index -lt $segments.Count - 1; $index += 1) {
            $parent = [IO.Path]::GetFullPath((Join-Path $parent $segments[$index]))
            [void][IO.Directory]::CreateDirectory($parent)
            $parentItem = Get-Item -LiteralPath $parent -Force
            $isLink = ($parentItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
            if (-not $parentItem.PSIsContainer -or $isLink) { Stop-DataValidation }
        }
        $destination = [IO.Path]::GetFullPath((Join-Path $parent $segments[-1]))
        $backupPrefix = Get-DirectoryPrefix $BackupRoot
        if (-not $destination.StartsWith($backupPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            Stop-DataValidation
        }
        if (Test-Path -LiteralPath $destination) {
            $existing = Get-Item -LiteralPath $destination -Force
            $isLink = ($existing.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
            if ($existing.PSIsContainer -or $isLink) { Stop-DataValidation }
        }
        [IO.File]::Copy($Entry.Full, $destination, $true)
        $sourceHash = Get-VerifiedContentHash $Entry.Full $Entry.Length
        $backupHash = Get-VerifiedContentHash $destination $Entry.Length
        if ($sourceHash -cne $Entry.Hash -or $backupHash -cne $Entry.Hash) {
            Stop-DataValidation
        }
    } catch {
        Stop-DataValidation
    }
}

function Get-BoundedDataFingerprint(
    [string]$Source,
    [string]$BackupDirectory,
    [int]$MaxFiles,
    [long]$MaxBytes
) {
    try {
        $root = Resolve-DataDirectory $Source $false
        $prefix = Get-DirectoryPrefix $root
        $backupRoot = ""
        if (-not [string]::IsNullOrWhiteSpace($BackupDirectory)) {
            $backupRoot = Resolve-DataDirectory $BackupDirectory $true
            $backupPrefix = Get-DirectoryPrefix $backupRoot
            if (
                $backupRoot.Equals($root, [StringComparison]::OrdinalIgnoreCase) -or
                $backupRoot.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase) -or
                $root.StartsWith($backupPrefix, [StringComparison]::OrdinalIgnoreCase)
            ) {
                Stop-DataValidation
            }
        }

        $files = New-Object System.Collections.Generic.List[object]
        $directories = New-Object System.Collections.Generic.Stack[string]
        $directories.Push($root)
        $totalBytes = 0L
        $entries = 0
        while ($directories.Count -gt 0) {
            $directory = $directories.Pop()
            foreach ($path in [IO.Directory]::EnumerateFileSystemEntries($directory)) {
                $entries += 1
                if ($entries -gt $MaxEntries) { Stop-DataValidation }
                $full = [IO.Path]::GetFullPath($path)
                if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
                    Stop-DataValidation
                }
                $item = Get-Item -LiteralPath $full -Force
                $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
                if ($isLink) { Stop-DataValidation }
                if ($item.PSIsContainer) {
                    $directories.Push($full)
                    continue
                }
                if ($files.Count -ge $MaxFiles) { Stop-DataValidation }
                if ($item.Length -gt $MaxBytes - $totalBytes) { Stop-DataValidation }
                $totalBytes += $item.Length
                $relative = $full.Substring($prefix.Length).Replace("\", "/")
                $files.Add([pscustomobject]@{
                    Full = $full
                    Relative = $relative
                    Length = [long]$item.Length
                    Hash = ""
                })
            }
        }

        $comparison = [System.Comparison[object]]{
            param($left, $right)
            [StringComparer]::Ordinal.Compare([string]$left.Relative, [string]$right.Relative)
        }
        $files.Sort($comparison)
        $sha = [Security.Cryptography.SHA256]::Create()
        try {
            foreach ($entry in $files) {
                $pathBytes = [Text.Encoding]::UTF8.GetBytes($entry.Relative)
                [void]$sha.TransformBlock($pathBytes, 0, $pathBytes.Length, $pathBytes, 0)
                [Array]::Clear($pathBytes, 0, $pathBytes.Length)
                $entry.Hash = Get-VerifiedContentHash $entry.Full $entry.Length
                $contentBytes = [Text.Encoding]::ASCII.GetBytes($entry.Hash)
                [void]$sha.TransformBlock($contentBytes, 0, $contentBytes.Length, $contentBytes, 0)
                [Array]::Clear($contentBytes, 0, $contentBytes.Length)
                if ($backupRoot) { Copy-VerifiedDataFile $entry $backupRoot }
            }
            [void]$sha.TransformFinalBlock([byte[]]@(), 0, 0)
            return -join @($sha.Hash | ForEach-Object { $_.ToString("X2") })
        } finally {
            $sha.Dispose()
        }
    } catch {
        Stop-DataValidation
    }
}

Get-BoundedDataFingerprint $Source $BackupDirectory $MaxFiles $MaxBytes
