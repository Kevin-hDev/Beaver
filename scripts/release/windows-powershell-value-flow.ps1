function Get-StaticStringAssignment(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [string]$VariableName,
    [int]$BeforeOffset
) {
    $assignments = @($Ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.AssignmentStatementAst] -and
            $node.Extent.StartOffset -lt $BeforeOffset -and
            $node.Left -is [Management.Automation.Language.VariableExpressionAst] -and
            $node.Left.VariablePath.UserPath -ieq $VariableName
    }, $true))
    if ($assignments.Count -eq 0 -or $assignments.Count -gt 64) { return $null }
    return @($assignments | Sort-Object { $_.Extent.StartOffset })[-1]
}

function Get-UniqueStaticStringAssignment(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [string]$VariableName,
    [int]$BeforeOffset
) {
    $assignments = @($Ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.AssignmentStatementAst] -and
            $node.Extent.StartOffset -lt $BeforeOffset -and
            $node.Left -is [Management.Automation.Language.VariableExpressionAst] -and
            $node.Left.VariablePath.UserPath -ieq $VariableName
    }, $true))
    if ($assignments.Count -ne 1) { return $null }
    return $assignments[0]
}

function Resolve-StaticStringExpression(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [Management.Automation.Language.Ast]$Expression,
    [int]$BeforeOffset,
    [int]$Depth = 0
) {
    if ($null -eq $Ast -or $null -eq $Expression -or $Depth -gt 8) { return $null }
    if ($Expression -is [Management.Automation.Language.CommandExpressionAst]) {
        return Resolve-StaticStringExpression `
            $Ast $Expression.Expression $BeforeOffset ($Depth + 1)
    }
    if ($Expression -is [Management.Automation.Language.StringConstantExpressionAst]) {
        return [string]$Expression.Value
    }
    if ($Expression -is [Management.Automation.Language.VariableExpressionAst]) {
        $assignment = Get-StaticStringAssignment `
            $Ast $Expression.VariablePath.UserPath $BeforeOffset
        if ($null -eq $assignment) { return $null }
        return Resolve-StaticStringExpression `
            $Ast $assignment.Right $assignment.Extent.StartOffset ($Depth + 1)
    }
    if ($Expression -is [Management.Automation.Language.ConvertExpressionAst]) {
        $value = Resolve-StaticStringExpression `
            $Ast $Expression.Child $BeforeOffset ($Depth + 1)
        if ($null -eq $value) { return $null }
        $typeName = [string]$Expression.Type.TypeName.FullName
        if ($typeName -iin @("string", "System.String")) { return [string]$value }
        if (
            $typeName -iin @("char", "System.Char") -and $value.Length -eq 1
        ) { return [string]$value[0] }
        return $null
    }
    if ($Expression -is [Management.Automation.Language.InvokeMemberExpressionAst]) {
        if (
            [string]$Expression.Member.Value -ine "Replace" -or
            $Expression.Arguments.Count -ne 2
        ) { return $null }
        $inputValue = Resolve-StaticStringExpression `
            $Ast $Expression.Expression $BeforeOffset ($Depth + 1)
        $oldValue = Resolve-StaticStringExpression `
            $Ast $Expression.Arguments[0] $BeforeOffset ($Depth + 1)
        $newValue = Resolve-StaticStringExpression `
            $Ast $Expression.Arguments[1] $BeforeOffset ($Depth + 1)
        if (
            $null -eq $inputValue -or [string]::IsNullOrEmpty($oldValue) -or
            $null -eq $newValue
        ) { return $null }
        $result = $inputValue.Replace($oldValue, $newValue)
        if ($result.Length -gt 65536) { return $null }
        return $result
    }
    if (
        $Expression -is [Management.Automation.Language.BinaryExpressionAst] -and
        $Expression.Operator -eq [Management.Automation.Language.TokenKind]::Plus
    ) {
        $left = Resolve-StaticStringExpression `
            $Ast $Expression.Left $BeforeOffset ($Depth + 1)
        $right = Resolve-StaticStringExpression `
            $Ast $Expression.Right $BeforeOffset ($Depth + 1)
        if ($null -eq $left -or $null -eq $right) { return $null }
        return $left + $right
    }
    if ($Expression -is [Management.Automation.Language.ParenExpressionAst]) {
        $pipeline = $Expression.Pipeline
        if (
            $pipeline.PipelineElements.Count -eq 1 -and
            $pipeline.PipelineElements[0] -is
                [Management.Automation.Language.CommandExpressionAst]
        ) {
            return Resolve-StaticStringExpression `
                $Ast $pipeline.PipelineElements[0].Expression $BeforeOffset ($Depth + 1)
        }
    }
    return $null
}

function Get-StaticStringCandidates(
    [Management.Automation.Language.ScriptBlockAst]$Ast,
    [Management.Automation.Language.Ast]$Expression,
    [int]$BeforeOffset,
    [int]$Depth = 0
) {
    if ($null -eq $Expression -or $Depth -gt 8) { return @() }
    if ($Expression -is [Management.Automation.Language.ConvertExpressionAst]) {
        $typeName = [string]$Expression.Type.TypeName.FullName
        if ($typeName -ieq "array" -or $typeName.EndsWith("[]")) {
            return @(Get-StaticStringCandidates `
                $Ast $Expression.Child $BeforeOffset ($Depth + 1))
        }
    }
    if ($Expression -is [Management.Automation.Language.CommandExpressionAst]) {
        return @(Get-StaticStringCandidates `
            $Ast $Expression.Expression $BeforeOffset ($Depth + 1))
    }
    if ($Expression -is [Management.Automation.Language.ParenExpressionAst]) {
        $pipeline = $Expression.Pipeline
        if (
            $pipeline.PipelineElements.Count -eq 1 -and
            $pipeline.PipelineElements[0] -is
                [Management.Automation.Language.CommandExpressionAst]
        ) {
            return @(Get-StaticStringCandidates `
                $Ast $pipeline.PipelineElements[0].Expression `
                $BeforeOffset ($Depth + 1))
        }
        return @()
    }
    if ($Expression -is [Management.Automation.Language.VariableExpressionAst]) {
        $assignment = Get-StaticStringAssignment `
            $Ast $Expression.VariablePath.UserPath $BeforeOffset
        if ($null -eq $assignment) { return @() }
        return @(Get-StaticStringCandidates `
            $Ast $assignment.Right $assignment.Extent.StartOffset ($Depth + 1))
    }
    if ($Expression -is [Management.Automation.Language.ArrayLiteralAst]) {
        if ($Expression.Elements.Count -gt 16) { return @() }
        $values = New-Object Collections.Generic.List[string]
        foreach ($element in $Expression.Elements) {
            foreach ($value in @(Get-StaticStringCandidates `
                $Ast $element $BeforeOffset ($Depth + 1))) {
                if ($values.Count -ge 16) { return @() }
                [void]$values.Add($value)
            }
        }
        return @($values)
    }
    if ($Expression -is [Management.Automation.Language.ArrayExpressionAst]) {
        $statements = @($Expression.SubExpression.Statements)
        if ($statements.Count -gt 16) { return @() }
        $values = New-Object Collections.Generic.List[string]
        foreach ($statement in $statements) {
            if (
                $statement -isnot [Management.Automation.Language.PipelineAst] -or
                $statement.PipelineElements.Count -ne 1 -or
                $statement.PipelineElements[0] -isnot
                    [Management.Automation.Language.CommandExpressionAst]
            ) { return @() }
            foreach ($value in @(Get-StaticStringCandidates `
                $Ast $statement.PipelineElements[0].Expression `
                $BeforeOffset ($Depth + 1))) {
                if ($values.Count -ge 16) { return @() }
                [void]$values.Add($value)
            }
        }
        return @($values)
    }
    $resolved = Resolve-StaticStringExpression $Ast $Expression $BeforeOffset $Depth
    if ($null -eq $resolved) { return @() }
    return @([string]$resolved)
}
