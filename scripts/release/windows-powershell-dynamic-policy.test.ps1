$ErrorActionPreference = "Stop"

function Assert-DynamicPolicy([bool]$Value) {
    if (-not $Value) { throw "PowerShell dynamic policy contract failed." }
}

. (Join-Path $PSScriptRoot "windows-powershell-add-type-policy.ps1")

function Get-DynamicTestAst([string]$Source) {
    $tokens = $null
    $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseInput(
        $Source,
        [ref]$tokens,
        [ref]$errors
    )
    Assert-DynamicPolicy ($errors.Count -eq 0)
    return $ast
}

$sources = @(
    @'
$compile = Get-Command -Name @("Add-Type")
& $compile -TypeDefinition "public static class ArrayCompile {}"
'@,
    @'
$compile = Get-Command -Name ([string[]]@("Add-Type"))
& $compile -TypeDefinition "public static class TypedArrayCompile {}"
'@,
    @'
$compile = Get-Command -Name ([object[]]@("Add-Type"))
& $compile -TypeDefinition "public static class ObjectArrayCompile {}"
'@,
    @'
$pattern = "Add-Ty*"
$compile = Get-Command -Name $pattern
& $compile -TypeDefinition "public static class WildcardCompile {}"
'@,
    @'
$payload = "AddXType -AssemblyName System.Drawing".replace("X", "-")
Invoke-Expression $payload
'@,
    @'
$payload = "AddXType -AssemblyName System.Drawing".Replace(
    [System.Char]"X",
    [System.Char]"-"
)
Invoke-Expression $payload
'@
)
for ($index = 0; $index -lt $sources.Count; $index += 1) {
    $source = $sources[$index]
    if (
        (Get-AddTypePolicyFailure `
            "scripts/release/foreign.ps1" $source (Get-DynamicTestAst $source)) -cne
        "managed-icon-decoder"
    ) { throw "PowerShell dynamic policy fixture $index failed." }
}

Write-Host "PowerShell dynamic policy contracts OK"
