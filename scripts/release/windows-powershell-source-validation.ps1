. (Join-Path $PSScriptRoot "windows-package-file.ps1")
. (Join-Path $PSScriptRoot "windows-powershell-managed-icon-policy.ps1")
. (Join-Path $PSScriptRoot "windows-powershell-add-type-policy.ps1")

function Get-PowerShellRelativePath([string]$Path) {
    $repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "../.."))
    $prefix = $repositoryRoot.TrimEnd("\") + "\"
    $fullPath = [IO.Path]::GetFullPath($Path)
    if ($fullPath.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        return $fullPath.Substring($prefix.Length).Replace("\", "/")
    }
    return "external.ps1"
}

function Get-PowerShellSourceFailure([string]$Path, [string]$AllowedRoot = "") {
    try {
        if ([string]::IsNullOrWhiteSpace($AllowedRoot)) {
            $source = Read-BoundedPackageText $Path 65536
        } else {
            $source = Read-BoundedPackageText $Path 65536 $AllowedRoot
        }
        $tokens = $null
        $errors = $null
        $ast = [Management.Automation.Language.Parser]::ParseInput(
            $source,
            [ref]$tokens,
            [ref]$errors
        )
        if ($errors.Count -ne 0) { return "syntax" }
        $addTypeFailure = Get-AddTypePolicyFailure `
            (Get-PowerShellRelativePath $Path) $source $ast
        if ($null -ne $addTypeFailure) { return $addTypeFailure }
        $importedNamespaces = @(
            $ast.UsingStatements |
                Where-Object { $_.UsingStatementKind -eq "Namespace" } |
                ForEach-Object { ([string]$_.Name).ToLowerInvariant() }
        )
        $ownerPath = Get-AddTypeOwnerPath (Get-PowerShellRelativePath $Path)
        $banned = $ast.Find({
            param($node)
            Test-BannedManagedIconNode $node $importedNamespaces $ownerPath $ast
        }, $true)
        if ($null -ne $banned) { return "managed-icon-decoder" }
        return $null
    } catch {
        return "read"
    }
}

function Get-RepositoryPowerShellFiles([string]$RepositoryRoot, [int]$MaxFiles) {
    if ($MaxFiles -le 0 -or $MaxFiles -gt 4096) {
        throw "PowerShell source discovery failed."
    }
    try {
        $root = [IO.Path]::GetFullPath($RepositoryRoot)
        $rootPrefix = $root.TrimEnd(
            [IO.Path]::DirectorySeparatorChar,
            [IO.Path]::AltDirectorySeparatorChar
        ) + [IO.Path]::DirectorySeparatorChar
        $paths = New-Object Collections.Generic.List[string]
        $seen = New-Object Collections.Generic.HashSet[string](
            [StringComparer]::OrdinalIgnoreCase
        )
        & git -c core.quotepath=false -C $root `
            ls-files --cached --others --exclude-standard -- `
            ":(icase,glob)*.ps1" ":(icase,glob)**/*.ps1" 2>$null |
            ForEach-Object {
                $relative = [string]$_
                if (
                    $paths.Count -ge $MaxFiles -or
                    [string]::IsNullOrWhiteSpace($relative) -or
                    $relative.Length -gt 4096 -or $relative -match "[\x00-\x1f]" -or
                    [IO.Path]::IsPathRooted($relative)
                ) {
                    throw "PowerShell source discovery failed."
                }
                $fullPath = [IO.Path]::GetFullPath((Join-Path $root $relative))
                if (
                    -not $fullPath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase) -or
                    [IO.Path]::GetExtension($fullPath) -ine ".ps1"
                ) {
                    throw "PowerShell source discovery failed."
                }
                if ($seen.Add($fullPath)) { [void]$paths.Add($fullPath) }
            }
        if ($LASTEXITCODE -ne 0 -or $paths.Count -le 0) {
            throw "PowerShell source discovery failed."
        }
        return @($paths | Sort-Object)
    } catch {
        throw "PowerShell source discovery failed."
    }
}

function Get-BoundedPowerShellTreeFiles(
    [string]$Root,
    [int]$MaxFiles,
    [int]$MaxEntries
) {
    if (
        $MaxFiles -le 0 -or $MaxFiles -gt 4096 -or
        $MaxEntries -lt $MaxFiles -or $MaxEntries -gt 100000
    ) { throw "PowerShell package discovery failed." }
    try {
        $rootItem = Get-Item -LiteralPath $Root -Force -ErrorAction Stop
        if (
            -not $rootItem.PSIsContainer -or
            ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        ) { throw "PowerShell package discovery failed." }
        $rootPath = [IO.Path]::GetFullPath($rootItem.FullName).TrimEnd("\")
        $prefix = $rootPath + "\"
        $files = New-Object Collections.Generic.List[string]
        $directories = New-Object Collections.Generic.Queue[object]
        $seenDirectories = New-Object Collections.Generic.HashSet[string](
            [StringComparer]::OrdinalIgnoreCase
        )
        [void]$seenDirectories.Add($rootPath)
        $directories.Enqueue([PSCustomObject]@{ Path = $rootPath; Depth = 0 })
        $entryCount = 0
        while ($directories.Count -gt 0) {
            $directory = $directories.Dequeue()
            foreach ($entry in [IO.Directory]::EnumerateFileSystemEntries($directory.Path)) {
                $entryCount += 1
                if ($entryCount -gt $MaxEntries) {
                    throw "PowerShell package discovery failed."
                }
                $item = Get-Item -LiteralPath $entry -Force -ErrorAction Stop
                $fullPath = [IO.Path]::GetFullPath($item.FullName)
                if (
                    -not $fullPath.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)
                ) { throw "PowerShell package discovery failed." }
                if ($item.PSIsContainer) {
                    if ($directory.Depth -ge 31) {
                        throw "PowerShell package discovery failed."
                    }
                    $nextPath = $fullPath
                    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                        # npm ships an internal junction; follow it only when its target stays owned.
                        $targets = @($item.Target)
                        if ($targets.Count -ne 1) {
                            throw "PowerShell package discovery failed."
                        }
                        $targetPath = [string]$targets[0]
                        if (-not [IO.Path]::IsPathRooted($targetPath)) {
                            $targetPath = Join-Path $item.Parent.FullName $targetPath
                        }
                        $targetItem = Get-Item -LiteralPath $targetPath -Force -ErrorAction Stop
                        $nextPath = [IO.Path]::GetFullPath($targetItem.FullName)
                        if (
                            -not $targetItem.PSIsContainer -or
                            ($targetItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
                            (
                                -not $nextPath.Equals(
                                    $rootPath,
                                    [StringComparison]::OrdinalIgnoreCase
                                ) -and
                                -not $nextPath.StartsWith(
                                    $prefix,
                                    [StringComparison]::OrdinalIgnoreCase
                                )
                            )
                        ) { throw "PowerShell package discovery failed." }
                    }
                    if ($seenDirectories.Add($nextPath)) {
                        $directories.Enqueue([PSCustomObject]@{
                            Path = $nextPath
                            Depth = $directory.Depth + 1
                        })
                    }
                } elseif ([IO.Path]::GetExtension($fullPath) -ieq ".ps1") {
                    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                        throw "PowerShell package discovery failed."
                    }
                    if ($files.Count -ge $MaxFiles) {
                        throw "PowerShell package discovery failed."
                    }
                    [void]$files.Add($fullPath)
                }
            }
        }
        return @($files | Sort-Object)
    } catch {
        throw "PowerShell package discovery failed."
    }
}

function Get-PowerShellTreeFailure(
    [string]$Root,
    [int]$MaxFiles,
    [int]$MaxEntries
) {
    try {
        $paths = @(Get-BoundedPowerShellTreeFiles $Root $MaxFiles $MaxEntries)
        foreach ($path in $paths) {
            $failure = Get-PowerShellSourceFailure $path $Root
            if ($null -ne $failure) { return $failure }
        }
        return $null
    } catch {
        return "read"
    }
}
