. (Join-Path $PSScriptRoot "windows-powershell-value-flow.ps1")

function Get-ReflectionMemberName(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [Management.Automation.Language.InvokeMemberExpressionAst]$Node
) {
    if ($Node.Member -is [Management.Automation.Language.StringConstantExpressionAst]) {
        return [string]$Node.Member.Value
    }
    return Resolve-StaticStringExpression `
        $Ast $Node.Member $Node.Extent.StartOffset
}

function Test-AllowedNativeIconReflection(
    [Management.Automation.Language.InvokeMemberExpressionAst]$Node,
    [string]$Member,
    [string]$OwnerPath
) {
    if ($OwnerPath -cne "scripts/release/windows-native-icon.ps1") { return $false }
    if (
        $Member -cne "GetMethod" -or
        $Node.Expression -isnot [Management.Automation.Language.VariableExpressionAst] -or
        $Node.Expression.VariablePath.UserPath -cne "script:NativeIconInteropType" -or
        $Node.Arguments.Count -ne 1 -or
        $Node.Arguments[0] -isnot [Management.Automation.Language.StringConstantExpressionAst]
    ) { return $false }
    return $Node.Arguments[0].Value -cin @("Extract", "Release")
}

function Test-ReflectionManagedDecoderNode(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [Management.Automation.Language.InvokeMemberExpressionAst]$Node,
    [string]$OwnerPath
) {
    $member = Get-ReflectionMemberName $Ast $Node
    if ($member -notin @(
        "CreateInstance", "GetConstructor", "GetConstructors", "GetMember",
        "GetMembers", "GetMethod", "GetMethods", "GetType", "GetTypes", "InvokeMember"
    )) { return $false }
    if ($member -ceq "GetType" -and $Node.Arguments.Count -eq 0) { return $false }
    return -not (Test-AllowedNativeIconReflection $Node $member $OwnerPath)
}
