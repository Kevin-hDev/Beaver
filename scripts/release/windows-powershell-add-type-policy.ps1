. (Join-Path $PSScriptRoot "windows-powershell-value-flow.ps1")
. (Join-Path $PSScriptRoot "windows-powershell-command-policy.ps1")

function Test-EmbeddedManagedDecoder([string]$Source) {
    if ([string]::IsNullOrWhiteSpace($Source) -or $Source.Length -gt 65536) {
        return $true
    }
    $patterns = @(
        '(?i)\bSystem\s*\.\s*Drawing\s*\.\s*(?:Icon|Image|Bitmap)\b',
        '(?i)\busing\s+System\s*\.\s*Drawing\b',
        '(?i)\bSystem\s*\.\s*Windows\s*\.\s*Media\s*\.\s*Imaging\b',
        '(?i)\busing\s+System\s*\.\s*Windows\s*\.\s*Media\s*\.\s*Imaging\b',
        '(?i)\bSystem\s*\.\s*Windows\s*\.\s*Markup\s*\.\s*XamlReader\b',
        '(?i)\bSystem\s*\.\s*Xaml\s*\.\s*XamlServices\b'
    )
    return $patterns.Where({ $Source -match $_ }).Count -ne 0
}

function Get-OwnerTypeDefinitionSource(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [string]$OwnerPath,
    [Management.Automation.Language.CommandAst]$Command,
    [string]$VariableName
) {
    $assignment = Get-UniqueStaticStringAssignment `
        $Ast $VariableName $Command.Extent.StartOffset
    if ($null -eq $assignment) { return $null }
    if (
        $OwnerPath -ceq "src-tauri/scripts/copy-windows-brand-resources.ps1" -and
        $VariableName -ceq "resourceApi"
    ) {
        $expression = $assignment.Right.Expression
        if (
            $expression -is [Management.Automation.Language.StringConstantExpressionAst] -and
            $expression.StringConstantType -eq "SingleQuotedHereString"
        ) { return [string]$expression.Value }
        return $null
    }
    if (
        $OwnerPath -cne "scripts/release/windows-native-icon.ps1" -or
        $VariableName -cne "nativeIconInteropSource"
    ) { return $null }
    $replace = $assignment.Right.Expression
    if (
        $replace -isnot [Management.Automation.Language.InvokeMemberExpressionAst] -or
        [string]$replace.Member.Value -cne "Replace" -or
        $replace.Expression -isnot [Management.Automation.Language.VariableExpressionAst] -or
        $replace.Expression.VariablePath.UserPath -cne "nativeIconInteropTemplate" -or
        $replace.Arguments.Count -ne 2 -or
        $replace.Arguments[0] -isnot [Management.Automation.Language.StringConstantExpressionAst] -or
        $replace.Arguments[0].Value -cne "__CLASS__" -or
        $replace.Arguments[1] -isnot [Management.Automation.Language.VariableExpressionAst] -or
        $replace.Arguments[1].VariablePath.UserPath -cne "nativeIconClassName"
    ) { return $null }
    $template = Get-UniqueStaticStringAssignment `
        $Ast "nativeIconInteropTemplate" $assignment.Extent.StartOffset
    if ($null -eq $template) { return $null }
    $templateExpression = $template.Right.Expression
    if (
        $templateExpression -isnot
            [Management.Automation.Language.StringConstantExpressionAst] -or
        $templateExpression.StringConstantType -ne "SingleQuotedHereString"
    ) { return $null }
    return [string]$templateExpression.Value
}

function Test-AllowedAddTypeCommand(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [Management.Automation.Language.CommandAst]$Node,
    [string]$OwnerPath
) {
    $elements = @($Node.CommandElements)
    if ($elements.Count -eq 3) {
        $parameter = $elements[1]
        $value = $elements[2]
        if (
            $parameter -isnot [Management.Automation.Language.CommandParameterAst] -or
            -not (Test-AddTypeParameterName $parameter.ParameterName "AssemblyName") -or
            $value -isnot [Management.Automation.Language.StringConstantExpressionAst]
        ) { return $false }
        if ($OwnerPath -ceq "install.ps1") { return $value.Value -ceq "System.Net.Http" }
        return $OwnerPath -ceq "scripts/release/windows-native-icon.ps1" -and
            $value.Value -ceq "System.Drawing"
    }
    $typeParameter = $elements.Where({
        $_ -is [Management.Automation.Language.CommandParameterAst] -and
        (Test-AddTypeParameterName $_.ParameterName "TypeDefinition")
    }, "First")
    if ($typeParameter.Count -ne 1) { return $false }
    $typeIndex = [Array]::IndexOf($elements, $typeParameter[0])
    if (
        $typeIndex -lt 0 -or $typeIndex + 1 -ge $elements.Count -or
        $elements[$typeIndex + 1] -isnot
            [Management.Automation.Language.VariableExpressionAst]
    ) { return $false }
    $variableName = $elements[$typeIndex + 1].VariablePath.UserPath
    $source = Get-OwnerTypeDefinitionSource $Ast $OwnerPath $Node $variableName
    if ($null -eq $source -or (Test-EmbeddedManagedDecoder $source)) { return $false }
    if ($OwnerPath -ceq "scripts/release/windows-native-icon.ps1") {
        return $elements.Count -eq 4 -and $elements[3] -is
            [Management.Automation.Language.CommandParameterAst] -and
            (Test-AddTypeParameterName $elements[3].ParameterName "PassThru")
    }
    return $OwnerPath -ceq "src-tauri/scripts/copy-windows-brand-resources.ps1" -and
        $elements.Count -eq 5 -and $elements[3] -is
            [Management.Automation.Language.CommandParameterAst] -and
        (Test-AddTypeParameterName $elements[3].ParameterName "Language") -and
        $elements[4] -is [Management.Automation.Language.StringConstantExpressionAst] -and
        $elements[4].Value -ceq "CSharp"
}

function Test-DynamicAddTypeNode(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [Management.Automation.Language.Ast]$Node
) {
    if ($Node -is [Management.Automation.Language.InvokeMemberExpressionAst]) {
        if (
            $Node.Expression -isnot [Management.Automation.Language.TypeExpressionAst] -or
            $Node.Expression.TypeName.FullName -ine "scriptblock" -or
            [string]$Node.Member.Value -ine "Create"
        ) { return $false }
        if ($Node.Arguments.Count -ne 1) { return $true }
        $resolved = Resolve-StaticStringExpression `
            $Ast $Node.Arguments[0] $Node.Extent.StartOffset
        if ($null -ne $resolved) { return $resolved -match "(?i)\bAdd-Type\b" }
        return $Node.Arguments[0].Extent.Text -match "(?i)\bAdd-Type\b"
    }
    if ($Node -isnot [Management.Automation.Language.CommandAst]) { return $false }
    $name = Get-AddTypeCommandName $Node
    if ($name -in @("Get-Command", "gcm")) {
        $elements = @($Node.CommandElements)
        for ($index = 1; $index -lt $elements.Count; $index += 1) {
            $target = $elements[$index]
            if ($target -is [Management.Automation.Language.CommandParameterAst]) {
                if ($null -eq $target.Argument) { continue }
                $target = $target.Argument
            }
            $candidates = @(Get-StaticStringCandidates `
                $Ast $target $Node.Extent.StartOffset)
            if ($candidates.Where({ Test-AddTypeNamePattern $_ }).Count -ne 0) {
                return $true
            }
        }
        return $false
    }
    if ($name -in @("Invoke-Expression", "iex")) {
        $argument = $Node.CommandElements[-1]
        $resolved = Resolve-StaticStringExpression `
            $Ast $argument $Node.Extent.StartOffset
        if ($null -ne $resolved) { return $resolved -match "(?i)\bAdd-Type\b" }
        return $argument.Extent.Text -match "(?i)\bAdd-Type\b"
    }
    if ($name -in @("Set-Alias", "New-Alias", "sal", "nal")) {
        $elements = @($Node.CommandElements)
        $target = $null
        for ($index = 1; $index -lt $elements.Count; $index += 1) {
            if (
                $elements[$index] -is [Management.Automation.Language.CommandParameterAst] -and
                (Test-AddTypeParameterName $elements[$index].ParameterName "Value") -and
                $index + 1 -lt $elements.Count
            ) { $target = $elements[$index + 1]; break }
        }
        if ($null -eq $target) {
            $positional = @($elements[1..($elements.Count - 1)] | Where-Object {
                $_ -isnot [Management.Automation.Language.CommandParameterAst]
            })
            if ($positional.Count -ge 2) { $target = $positional[1] }
        }
        if ($null -eq $target) { return $true }
        $resolved = Resolve-StaticStringExpression `
            $Ast $target $Node.Extent.StartOffset
        return $null -eq $resolved -or $resolved -ieq "Add-Type"
    }
    if (-not [string]::IsNullOrWhiteSpace($name)) { return $false }
    if ($Node.CommandElements.Count -eq 0) { return $true }
    $resolved = Resolve-StaticStringExpression `
        $Ast $Node.CommandElements[0] $Node.Extent.StartOffset
    return $resolved -ieq "Add-Type"
}

function Get-AddTypePolicyFailure(
    [string]$RelativePath,
    [string]$Source,
    [Management.Automation.Language.ScriptBlockAst]$Ast
) {
    if ($null -eq $Ast -or $Source.Length -gt 65536) { return "managed-icon-decoder" }
    $dynamic = $Ast.Find({ param($node) Test-DynamicAddTypeNode $Ast $node }, $true)
    if ($null -ne $dynamic) { return "managed-icon-decoder" }
    $commands = @($Ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.CommandAst] -and
            (Get-AddTypeCommandName $node) -ieq "Add-Type"
    }, $true))
    if ($commands.Count -eq 0) { return $null }
    if ($commands.Count -gt 8) { return "managed-icon-decoder" }
    $ownerPath = Get-AddTypeOwnerPath $RelativePath
    foreach ($command in $commands) {
        if (-not (Test-AllowedAddTypeCommand $Ast $command $ownerPath)) {
            return "managed-icon-decoder"
        }
    }
    return $null
}
