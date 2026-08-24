function Get-AddTypeOwnerPath([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path) -or $Path.Length -gt 4096) { return "" }
    return $Path.Replace("\", "/").TrimStart("/").ToLowerInvariant()
}

function Test-AddTypeParameterName([string]$Actual, [string]$Expected) {
    return -not [string]::IsNullOrWhiteSpace($Actual) -and
        $Expected.StartsWith($Actual, [StringComparison]::OrdinalIgnoreCase)
}

function Get-AddTypeCommandName([Management.Automation.Language.CommandAst]$Node) {
    $name = [string]$Node.GetCommandName()
    if ($name.LastIndexOf([char]92) -ge 0) {
        $name = $name.Substring($name.LastIndexOf([char]92) + 1)
    }
    return $name
}

function Test-AddTypeNamePattern([string]$Value) {
    if ([string]::IsNullOrWhiteSpace($Value) -or $Value.Length -gt 256) {
        return $false
    }
    try {
        $pattern = New-Object Management.Automation.WildcardPattern(
            $Value,
            [Management.Automation.WildcardOptions]::IgnoreCase
        )
        return $pattern.IsMatch("Add-Type")
    } catch {
        return $false
    }
}
