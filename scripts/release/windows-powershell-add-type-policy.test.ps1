$ErrorActionPreference = "Stop"

function Assert-AddTypePolicy([bool]$Value) {
    if (-not $Value) { throw "PowerShell Add-Type policy contract failed." }
}

. (Join-Path $PSScriptRoot "windows-powershell-add-type-policy.ps1")

function Get-TestAst([string]$Source) {
    $tokens = $null
    $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseInput(
        $Source,
        [ref]$tokens,
        [ref]$errors
    )
    Assert-AddTypePolicy ($errors.Count -eq 0)
    return $ast
}

$safeNativeSource = [string]::Join("`n", @(
    "`$nativeIconInteropTemplate = @'",
    "using System;",
    "public static class __CLASS__ { public static void Release() {} }",
    "'@",
    "Add-Type -AssemblyName System.Drawing",
    "`$nativeIconClassName = 'NativeIcon_1'",
    "`$nativeIconInteropSource = `$nativeIconInteropTemplate.Replace(" +
        "'__CLASS__', `$nativeIconClassName)",
    "Add-Type -TypeDefinition `$nativeIconInteropSource -PassThru"
))
$safeNativeAst = Get-TestAst $safeNativeSource
Assert-AddTypePolicy (
    $null -eq (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" $safeNativeSource $safeNativeAst)
)
$abbreviatedNativeSource = $safeNativeSource.Replace("-AssemblyName", "-As")
Assert-AddTypePolicy (
    $null -eq (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" `
        $abbreviatedNativeSource `
        (Get-TestAst $abbreviatedNativeSource))
)
$documentedNativeSource = $safeNativeSource.Replace(
    "using System;",
    "using System; // Image resources stay native."
)
Assert-AddTypePolicy (
    $null -eq (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" `
        $documentedNativeSource `
        (Get-TestAst $documentedNativeSource))
)

$embeddedDecoderSource = $safeNativeSource.Replace(
    "public static void Release() {}",
    "public static System.Drawing.Icon Load(string path) { return new System.Drawing.Icon(path); }"
)
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" `
        $embeddedDecoderSource `
        (Get-TestAst $embeddedDecoderSource)) -ceq "managed-icon-decoder"
)

$ordinaryDecoderSource = @'
Add-Type -AssemblyName System.Drawing
$nativeIconInteropSource = "using System.Drawing; public static class Leak { public static Icon Load(string path) { return new Icon(path); } }"
Add-Type -TypeDefinition $nativeIconInteropSource -PassThru
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" `
        $ordinaryDecoderSource `
        (Get-TestAst $ordinaryDecoderSource)) -ceq "managed-icon-decoder"
)

$urlDecoderSource = $safeNativeSource.Replace(
    "using System;",
    "using System; string url = `"https://example`"; System.Drawing.Icon icon = null;"
)
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" `
        $urlDecoderSource `
        (Get-TestAst $urlDecoderSource)) -ceq "managed-icon-decoder"
)

$embeddedWpfSource = $safeNativeSource.Replace(
    "public static void Release() {}",
    "public static object Load() { return new System.Windows.Media.Imaging.BitmapImage(); }"
)
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" `
        $embeddedWpfSource `
        (Get-TestAst $embeddedWpfSource)) -ceq "managed-icon-decoder"
)

$foreignSource = 'Add-Type -AssemblyName System.Drawing'
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" $foreignSource (Get-TestAst $foreignSource)
    ) -ceq "managed-icon-decoder"
)

$dynamicCommandSource = @'
$compile = "Add-Type"
& $compile -TypeDefinition "public static class DynamicCompile {}"
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $dynamicCommandSource `
        (Get-TestAst $dynamicCommandSource)) -ceq "managed-icon-decoder"
)

$reassignedCommandSource = @'
$compile = "Write-Output"
$compile = "Add-Type"
& $compile -TypeDefinition "public static class ReassignedCompile {}"
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $reassignedCommandSource `
        (Get-TestAst $reassignedCommandSource)) -ceq "managed-icon-decoder"
)

$commandInfoSource = @'
$compile = Get-Command Add-Type
& $compile -TypeDefinition "public static class CommandInfoCompile {}"
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $commandInfoSource `
        (Get-TestAst $commandInfoSource)) -ceq "managed-icon-decoder"
)

$reorderedCommandInfoSource = @'
$compile = Get-Command -CommandType Cmdlet -Name:Add-Type
& $compile -TypeDefinition "public static class ReorderedCommandInfo {}"
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $reorderedCommandInfoSource `
        (Get-TestAst $reorderedCommandInfoSource)) -ceq "managed-icon-decoder"
)

$dynamicAliasSource = @'
$name = "Add-Type"
Set-Alias compile $name
compile -TypeDefinition "public static class DynamicAlias {}"
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $dynamicAliasSource `
        (Get-TestAst $dynamicAliasSource)) -ceq "managed-icon-decoder"
)

$dynamicScopedAliasSource = @'
$name = "Add-Type"
Set-Alias -Name compile -Value $name -Scope Script
compile -TypeDefinition "public static class DynamicScopedAlias {}"
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $dynamicScopedAliasSource `
        (Get-TestAst $dynamicScopedAliasSource)) -ceq "managed-icon-decoder"
)

$safeDynamicExecutionSource = @'
$command = "& `"node.exe`" `"npm-cli.js`" --version"
Invoke-Expression $command
'@
Assert-AddTypePolicy (
    $null -eq (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $safeDynamicExecutionSource `
        (Get-TestAst $safeDynamicExecutionSource))
)

$dynamicExpressionSource = @'
$payload = "Add-Type -AssemblyName System.Drawing"
Invoke-Expression $payload
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $dynamicExpressionSource `
        (Get-TestAst $dynamicExpressionSource)) -ceq "managed-icon-decoder"
)

$replacedExpressionSource = @'
$payload = "AddXType -AssemblyName System.Drawing".Replace("X", "-")
Invoke-Expression $payload
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $replacedExpressionSource `
        (Get-TestAst $replacedExpressionSource)) -ceq "managed-icon-decoder"
)

$charReplacedExpressionSource = @'
$payload = "AddXType -AssemblyName System.Drawing".Replace([char]"X", [char]"-")
Invoke-Expression $payload
'@
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/foreign.ps1" `
        $charReplacedExpressionSource `
        (Get-TestAst $charReplacedExpressionSource)) -ceq "managed-icon-decoder"
)

$extraOwnerCommand = "$safeNativeSource`nAdd-Type -AssemblyName PresentationCore"
Assert-AddTypePolicy (
    (Get-AddTypePolicyFailure `
        "scripts/release/windows-native-icon.ps1" `
        $extraOwnerCommand `
        (Get-TestAst $extraOwnerCommand)) -ceq "managed-icon-decoder"
)

Write-Host "PowerShell Add-Type policy contracts OK"
