$ErrorActionPreference = "Stop"

function Assert-SourceTrue([bool]$Value) {
    if (-not $Value) { throw "PowerShell source validation contract failed." }
}

. (Join-Path $PSScriptRoot "windows-powershell-source-validation.ps1")

$randomBytes = New-Object byte[] 16
$random = [Security.Cryptography.RandomNumberGenerator]::Create()
try {
    $random.GetBytes($randomBytes)
} finally {
    $random.Dispose()
}
$directoryName = -join @($randomBytes | ForEach-Object { $_.ToString("x2") })
$temporaryBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd("\")
$temporaryRoot = [IO.Path]::GetFullPath((Join-Path $temporaryBase $directoryName))

try {
    [void](New-Item -ItemType Directory -Path $temporaryRoot)
    $bannedSources = @(
        '[Drawing.Icon]::new("icon.ico")',
        '[System.Drawing.Icon]::new("icon.ico")',
        'New-Object Drawing.Icon "icon.ico"',
        'New-Object -TypeName System.Drawing.Icon -ArgumentList "icon.ico"',
        '[Drawing.Image]::FromFile("icon.ico")',
        '[Drawing.Bitmap]::new("icon.ico")',
        '[Drawing.Bitmap]::new("icon.ico", $true)',
        'New-Object Drawing.Bitmap "icon.ico"',
        'New-Object Drawing.Bitmap "icon.ico", $true',
        '[Drawing.Image]::FromStream([IO.File]::OpenRead("icon.ico"))'
        "using namespace System.Drawing`n[Icon]::new('icon.ico')",
        "using namespace System.Drawing`nNew-Object Icon 'icon.ico'",
        "using namespace System.Drawing`n[Image]::FromFile('icon.ico')",
        "New-Object -TypeName ('Drawing.' + 'Icon') -ArgumentList 'icon.ico'",
        "New-Object -TypeName ([Drawing.Icon]) -ArgumentList 'icon.ico'",
        "`$iconType = [Drawing.Icon]`n`$iconType::new('icon.ico')",
        "[System.Drawing.Icon, System.Drawing]::new('icon.ico')",
        "Microsoft.PowerShell.Utility\New-Object -TypeName Drawing.Icon -ArgumentList 'icon.ico'",
        "[Activator]::CreateInstance([Drawing.Icon], @('icon.ico'))",
        "([Drawing.Icon]).GetConstructor(@([string])).Invoke(@('icon.ico'))",
        "([Drawing.Icon]).GetConstructors()[0].Invoke(@('icon.ico'))",
        "[type]::GetType('System.Drawing.Icon').GetMethod('FromFile')",
        "[Activator]::CreateInstance('System.Drawing', 'System.Drawing.Icon')",
        "[System.Windows.Media.Imaging.BitmapImage]::new()",
        "New-Object -Ty System.Drawing.Icon 'icon.ico'",
        "Set-Alias -Name compile -Value Add-Type`ncompile -AssemblyName System.Drawing",
        "Invoke-Expression 'Add-Type -AssemblyName System.Drawing'",
        "& ('Add' + '-Type') -AssemblyName System.Drawing",
        "`$typeName = 'System.Drawing.Icon'`n`$assembly.GetType(`$typeName).GetConstructor(@([string]))",
        "`$typeName = 'System.Drawing.Icon'`n`$assembly.GetType(`$typeName).GetMethod('FromFile')",
        "`$assembly.GetTypes() | Where-Object FullName -eq 'System.Drawing.Icon'",
        "`$type = [type]'System.Drawing.Icon'`n`$type.InvokeMember('', [Reflection.BindingFlags]::CreateInstance, `$null, `$null, @('icon.ico'))",
        "`$member = 'InvokeMember'`n`$type = [type]'System.Drawing.Icon'`n`$type.`$member('', [Reflection.BindingFlags]::CreateInstance, `$null, `$null, @('icon.ico'))",
        "`$member = 'InvokeXMember'.replace('X', '')`n`$type = [type]'System.Drawing.Icon'`n`$type.`$member('', [Reflection.BindingFlags]::CreateInstance, `$null, `$null, @('icon.ico'))",
        "`$xaml = '<BitmapImage />'`n[System.Windows.Markup.XamlReader]::Parse(`$xaml)",
        "`$xaml = '<BitmapImage />'`n[Windows.Markup.XamlReader]::Parse(`$xaml)",
        "`$xaml = '<BitmapImage />'`n[System.Xaml.XamlServices]::Parse(`$xaml)",
        "using namespace System.Xaml`n`$xaml = '<BitmapImage />'`n[XamlServices]::Parse(`$xaml)"
    )
    for ($index = 0; $index -lt $bannedSources.Count; $index += 1) {
        $path = Join-Path $temporaryRoot "banned-$index.ps1"
        [IO.File]::WriteAllText($path, $bannedSources[$index])
        if ((Get-PowerShellSourceFailure $path) -cne "managed-icon-decoder") {
            throw "PowerShell source validation contract failed for banned fixture $index."
        }
    }

    $allowedPath = Join-Path $temporaryRoot "allowed.ps1"
    [IO.File]::WriteAllText(
        $allowedPath,
        @"
using namespace System.Drawing
using namespace System.Drawing.Imaging
New-Object Drawing.Bitmap 2, 2
[Drawing.Bitmap]::new(2, 2)
[Drawing.Bitmap]::new(2, 2, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
[Bitmap]::new(2, 2)
[Bitmap]::new(2, 2, [PixelFormat]::Format32bppArgb)
[Drawing.Icon]::FromHandle([IntPtr]::Zero)
`$value = 'safe'
`$null = `$value.GetType()
`$null = New-Object -Ty System.Text.StringBuilder
"@
    )
    Assert-SourceTrue ($null -eq (Get-PowerShellSourceFailure $allowedPath))

    $invalidPath = Join-Path $temporaryRoot "invalid.ps1"
    [IO.File]::WriteAllText($invalidPath, "function Broken {")
    Assert-SourceTrue ((Get-PowerShellSourceFailure $invalidPath) -ceq "syntax")

    $repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "../.."))
    $syntaxGate = Join-Path $repositoryRoot "scripts/test-install-ps1.ps1"
    $actualPaths = @(& $syntaxGate -ListOnly)
    $expectedPaths = @(
        & git -C $repositoryRoot ls-files --cached --others --exclude-standard -- "*.ps1" |
            ForEach-Object { [IO.Path]::GetFullPath((Join-Path $repositoryRoot $_)) } |
            Sort-Object -Unique
    )
    Assert-SourceTrue ($LASTEXITCODE -eq 0)
    Assert-SourceTrue ($actualPaths.Count -eq $expectedPaths.Count)
    foreach ($expectedPath in $expectedPaths) {
        Assert-SourceTrue ($expectedPath -in $actualPaths)
    }
    Assert-SourceTrue (
        (Join-Path $PSScriptRoot "check-nsis-migration.ps1") -in $actualPaths
    )

    $unicodeRepository = Join-Path $temporaryRoot "unicode-repository"
    [void](New-Item -ItemType Directory -Path $unicodeRepository)
    & git -C $unicodeRepository init --quiet
    Assert-SourceTrue ($LASTEXITCODE -eq 0)
    $unicodeScript = Join-Path $unicodeRepository "étape.ps1"
    $uppercaseScript = Join-Path $unicodeRepository "Upper.PS1"
    [IO.File]::WriteAllText($unicodeScript, "Write-Output 'ok'")
    [IO.File]::WriteAllText($uppercaseScript, "Write-Output 'ok'")
    & git -C $unicodeRepository add -- "étape.ps1" "Upper.PS1"
    Assert-SourceTrue ($LASTEXITCODE -eq 0)
    $unicodePaths = @(Get-RepositoryPowerShellFiles $unicodeRepository 8)
    Assert-SourceTrue ($unicodePaths.Count -eq 2)
    Assert-SourceTrue ($unicodeScript -in $unicodePaths)
    Assert-SourceTrue ($uppercaseScript -in $unicodePaths)

    $packagedRoot = Join-Path $temporaryRoot "packaged"
    [void](New-Item -ItemType Directory -Path (Join-Path $packagedRoot "runtime/bin") -Force)
    $packagedScripts = @("jiti.ps1", "nanoid.ps1", "xml-js.ps1")
    foreach ($name in $packagedScripts) {
        [IO.File]::WriteAllText((Join-Path $packagedRoot $name), "Write-Output 'ok'")
    }
    [IO.File]::WriteAllText(
        (Join-Path $packagedRoot "runtime/bin/npm.ps1"),
        "Write-Output 'ok'"
    )
    [IO.File]::WriteAllText(
        (Join-Path $packagedRoot "runtime/bin/npx.ps1"),
        "Write-Output 'ok'"
    )
    $junctionTarget = Join-Path $packagedRoot "vendor/disabled-package"
    [void](New-Item -ItemType Directory -Path $junctionTarget -Force)
    [IO.File]::WriteAllText((Join-Path $junctionTarget "safe.ps1"), "Write-Output 'ok'")
    [void](New-Item -ItemType Junction -Path (Join-Path $packagedRoot "linked-package") `
        -Target $junctionTarget)
    $packagedPaths = @(Get-BoundedPowerShellTreeFiles $packagedRoot 8 32)
    Assert-SourceTrue ($packagedPaths.Count -eq 6)
    foreach ($packagedPath in $packagedPaths) {
        Assert-SourceTrue ($null -eq (Get-PowerShellSourceFailure $packagedPath $packagedRoot))
    }
    Assert-SourceTrue ($null -eq (Get-PowerShellTreeFailure $packagedRoot 8 32))
    $bannedPackagedScript = Join-Path $packagedRoot "banned.ps1"
    [IO.File]::WriteAllText($bannedPackagedScript, "[Drawing.Icon]::new('icon.ico')")
    Assert-SourceTrue (
        (Get-PowerShellTreeFailure $packagedRoot 8 32) -ceq "managed-icon-decoder"
    )
    [IO.File]::Delete($bannedPackagedScript)
    & $syntaxGate
    Assert-SourceTrue ($LASTEXITCODE -eq 0)
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        if (
            [IO.Path]::GetDirectoryName($temporaryRoot) -cne $temporaryBase -or
            [IO.Path]::GetFileName($temporaryRoot) -notmatch "^[a-f0-9]{32}$"
        ) {
            throw "PowerShell source validation contract failed."
        }
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

Write-Host "PowerShell source validation contracts OK"
