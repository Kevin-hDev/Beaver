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
        "([Drawing.Icon]).GetConstructor(@([string])).Invoke(@('icon.ico'))"
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
