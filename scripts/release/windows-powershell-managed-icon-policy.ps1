function Test-ManagedIconType(
    [string]$TypeName,
    [string[]]$ExpectedNames,
    [string[]]$ImportedNamespaces
) {
    if ([string]::IsNullOrWhiteSpace($TypeName)) { return $false }
    $normalized = @($TypeName.Split(",", 2))[0].Trim().ToLowerInvariant()
    if ($normalized -in $ExpectedNames) { return $true }
    if (-not $normalized.StartsWith("system.") -and "system.$normalized" -in $ExpectedNames) {
        return $true
    }
    if ($normalized -notmatch "\.") {
        foreach ($namespace in $ImportedNamespaces) {
            if ("$namespace.$normalized" -in $ExpectedNames) { return $true }
        }
    }
    return $false
}

function Get-ManagedIconTypeKind([string]$TypeName, [string[]]$ImportedNamespaces) {
    if (Test-ManagedIconType $TypeName @("drawing.icon", "system.drawing.icon") $ImportedNamespaces) {
        return "icon"
    }
    if (Test-ManagedIconType $TypeName @("drawing.bitmap", "system.drawing.bitmap") $ImportedNamespaces) {
        return "bitmap"
    }
    if (Test-ManagedIconType $TypeName @("drawing.image", "system.drawing.image") $ImportedNamespaces) {
        return "image"
    }
    if (Test-ManagedIconType $TypeName @(
        "system.windows.media.imaging.bitmapdecoder",
        "system.windows.media.imaging.bitmapframe",
        "system.windows.media.imaging.bitmapimage",
        "system.windows.media.imaging.bmpbitmapdecoder",
        "system.windows.media.imaging.gifbitmapdecoder",
        "system.windows.media.imaging.iconbitmapdecoder",
        "system.windows.media.imaging.jpegbitmapdecoder",
        "system.windows.media.imaging.pngbitmapdecoder",
        "system.windows.media.imaging.tiffbitmapdecoder",
        "system.windows.media.imaging.wmpbitmapdecoder"
    ) $ImportedNamespaces) {
        return "wpf-decoder"
    }
    if (Test-ManagedIconType $TypeName @(
        "system.windows.markup.xamlreader",
        "system.xaml.xamlservices"
    ) $ImportedNamespaces) {
        return "xaml-loader"
    }
    return $null
}

. (Join-Path $PSScriptRoot "windows-powershell-reflection-policy.ps1")

function Test-ParameterName([string]$Actual, [string]$Expected) {
    return -not [string]::IsNullOrWhiteSpace($Actual) -and
        $Expected.StartsWith($Actual, [StringComparison]::OrdinalIgnoreCase)
}

function Test-BitmapDimensionNode(
    [Management.Automation.Language.ExpressionAst]$Node,
    [string]$ExpectedMember
) {
    if ($Node -is [Management.Automation.Language.ConstantExpressionAst]) {
        return $Node.Value -is [ValueType] -and [int64]$Node.Value -gt 0
    }
    return $Node -is [Management.Automation.Language.MemberExpressionAst] -and
        [string]$Node.Member.Value -ceq $ExpectedMember
}

function Test-BitmapPixelFormatNode(
    [Management.Automation.Language.ExpressionAst]$Node,
    [string[]]$ImportedNamespaces
) {
    if (
        $Node -isnot [Management.Automation.Language.MemberExpressionAst] -or
        $Node.Expression -isnot [Management.Automation.Language.TypeExpressionAst]
    ) { return $false }
    return (
        Test-ManagedIconType $Node.Expression.TypeName.FullName @(
            "drawing.imaging.pixelformat",
            "system.drawing.imaging.pixelformat"
        ) $ImportedNamespaces
    ) -and [string]$Node.Member.Value -match "^Format[0-9]+bpp"
}

function Test-AllowedBitmapInvocation(
    [Management.Automation.Language.InvokeMemberExpressionAst]$Node,
    [string[]]$ImportedNamespaces
) {
    return $Node.Arguments.Count -in @(2, 3) -and
        (Test-BitmapDimensionNode $Node.Arguments[0] "Width") -and
        (Test-BitmapDimensionNode $Node.Arguments[1] "Height") -and
        (
            $Node.Arguments.Count -eq 2 -or
            (Test-BitmapPixelFormatNode $Node.Arguments[2] $ImportedNamespaces)
        )
}

function Test-AllowedManagedTypeNode(
    [Management.Automation.Language.Ast]$Node,
    [string]$Kind,
    [string[]]$ImportedNamespaces
) {
    $parent = $Node.Parent
    if ($Node -is [Management.Automation.Language.TypeConstraintAst]) {
        if ($parent -is [Management.Automation.Language.ParameterAst]) { return $true }
        return $Kind -eq "icon" -and
            $parent -is [Management.Automation.Language.ConvertExpressionAst] -and
            $parent.Child -is [Management.Automation.Language.InvokeMemberExpressionAst] -and
            [string]$parent.Child.Member.Value -ceq "Clone"
    }
    if ($parent -isnot [Management.Automation.Language.InvokeMemberExpressionAst]) {
        return $false
    }
    if (-not [object]::ReferenceEquals($parent.Expression, $Node)) { return $false }
    $member = [string]$parent.Member.Value
    if ($Kind -eq "icon" -and $member -ceq "FromHandle") { return $true }
    return $Kind -eq "bitmap" -and $member -ceq "new" -and
        (Test-AllowedBitmapInvocation $parent $ImportedNamespaces)
}

function Get-NewObjectTypeIndex([Management.Automation.Language.CommandAst]$Node) {
    for ($index = 1; $index -lt $Node.CommandElements.Count; $index += 1) {
        $element = $Node.CommandElements[$index]
        if (
            $element -is [Management.Automation.Language.CommandParameterAst] -and
            (Test-ParameterName $element.ParameterName "TypeName")
        ) { return $index + 1 }
    }
    if (
        $Node.CommandElements.Count -gt 1 -and
        $Node.CommandElements[1] -isnot [Management.Automation.Language.CommandParameterAst]
    ) { return 1 }
    return -1
}

function Test-AllowedNewBitmap(
    [Management.Automation.Language.CommandAst]$Node,
    [int]$TypeIndex
) {
    if ($TypeIndex -ge ($Node.CommandElements.Count - 1)) { return $false }
    $arguments = @(
        $Node.CommandElements[($TypeIndex + 1)..($Node.CommandElements.Count - 1)] |
            Where-Object { $_ -isnot [Management.Automation.Language.CommandParameterAst] }
    )
    return $arguments.Count -eq 1 -and
        $arguments[0] -is [Management.Automation.Language.ArrayLiteralAst] -and
        $arguments[0].Elements.Count -eq 2 -and
        (Test-BitmapDimensionNode $arguments[0].Elements[0] "Width") -and
        (Test-BitmapDimensionNode $arguments[0].Elements[1] "Height")
}

function Test-BannedManagedIconNode(
    [Management.Automation.Language.Ast]$Node,
    [string[]]$ImportedNamespaces,
    [string]$OwnerPath,
    [Management.Automation.Language.ScriptBlockAst]$Ast
) {
    if (
        $Node -is [Management.Automation.Language.TypeExpressionAst] -or
        $Node -is [Management.Automation.Language.TypeConstraintAst]
    ) {
        $kind = Get-ManagedIconTypeKind $Node.TypeName.FullName $ImportedNamespaces
        return $null -ne $kind -and
            -not (Test-AllowedManagedTypeNode $Node $kind $ImportedNamespaces)
    }
    if ($Node -is [Management.Automation.Language.InvokeMemberExpressionAst]) {
        $member = [string]$Node.Member.Value
        if (Test-ReflectionManagedDecoderNode $Ast $Node $OwnerPath) { return $true }
        if ($Node.Expression -isnot [Management.Automation.Language.TypeExpressionAst]) {
            return $Node.Static -and $member -in @(
                "new", "ExtractAssociatedIcon", "FromFile", "FromStream"
            )
        }
        $kind = Get-ManagedIconTypeKind $Node.Expression.TypeName.FullName $ImportedNamespaces
        if ($null -eq $kind) { return $false }
        return -not (Test-AllowedManagedTypeNode $Node.Expression $kind $ImportedNamespaces)
    }
    if ($Node -isnot [Management.Automation.Language.CommandAst]) { return $false }
    $commandName = [string]$Node.GetCommandName()
    if ($commandName.LastIndexOf([char]92) -ge 0) {
        $commandName = $commandName.Substring($commandName.LastIndexOf([char]92) + 1)
    }
    if ($commandName -ine "New-Object") { return $false }
    if ($Node.CommandElements.Where({
        $_ -is [Management.Automation.Language.CommandParameterAst] -and
        (Test-ParameterName $_.ParameterName "ComObject")
    }).Count -ne 0) { return $false }
    $typeIndex = Get-NewObjectTypeIndex $Node
    if ($typeIndex -lt 0 -or $typeIndex -ge $Node.CommandElements.Count) { return $true }
    $typeElement = $Node.CommandElements[$typeIndex]
    if ($typeElement -isnot [Management.Automation.Language.StringConstantExpressionAst]) {
        return $true
    }
    $kind = Get-ManagedIconTypeKind $typeElement.Value $ImportedNamespaces
    if ($null -eq $kind) { return $false }
    return $kind -ne "bitmap" -or -not (Test-AllowedNewBitmap $Node $typeIndex)
}
