. (Join-Path $PSScriptRoot "windows-package-file.ps1")
. (Join-Path $PSScriptRoot "windows-powershell-managed-icon-policy.ps1")

function Get-PowerShellSourceFailure([string]$Path) {
    try {
        $source = Read-BoundedPackageText $Path 65536
        $tokens = $null
        $errors = $null
        $ast = [Management.Automation.Language.Parser]::ParseInput(
            $source,
            [ref]$tokens,
            [ref]$errors
        )
        if ($errors.Count -ne 0) { return "syntax" }
        $importedNamespaces = @(
            $ast.UsingStatements |
                Where-Object { $_.UsingStatementKind -eq "Namespace" } |
                ForEach-Object { ([string]$_.Name).ToLowerInvariant() }
        )
        $banned = $ast.Find({
            param($node)
            Test-BannedManagedIconNode $node $importedNamespaces
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
        & git -C $root ls-files --cached --others --exclude-standard -- "*.ps1" 2>$null |
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
