$ErrorActionPreference = "Stop"

function Assert-SyntaxInventory([bool]$Value) {
    if (-not $Value) { throw "PowerShell syntax inventory contract failed." }
}

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
    & git -C $temporaryRoot init --quiet
    Assert-SyntaxInventory ($LASTEXITCODE -eq 0)

    $unicodeScript = Join-Path $temporaryRoot "étape.ps1"
    $uppercaseScript = Join-Path $temporaryRoot "Deploy.PS1"
    [IO.File]::WriteAllText($unicodeScript, "Write-Output 'ok'")
    [IO.File]::WriteAllText($uppercaseScript, "function Broken {")
    & git -C $temporaryRoot add -- "étape.ps1"
    Assert-SyntaxInventory ($LASTEXITCODE -eq 0)

    $syntaxGate = Join-Path $PSScriptRoot "../test-install-ps1.ps1"
    $originalOutputCodePage = [Console]::OutputEncoding.CodePage
    $paths = @(& $syntaxGate -RepositoryRoot $temporaryRoot -ListOnly)
    Assert-SyntaxInventory ([Console]::OutputEncoding.CodePage -eq $originalOutputCodePage)
    Assert-SyntaxInventory ($paths.Count -eq 2)
    Assert-SyntaxInventory ($unicodeScript -in $paths)
    Assert-SyntaxInventory ($uppercaseScript -in $paths)

    $invalidSyntaxRejected = $false
    try {
        & $syntaxGate -RepositoryRoot $temporaryRoot
    } catch {
        $invalidSyntaxRejected = $_.Exception.Message -ceq "PowerShell syntax invalid."
    }
    Assert-SyntaxInventory $invalidSyntaxRejected
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        if (
            [IO.Path]::GetDirectoryName($temporaryRoot) -cne $temporaryBase -or
            [IO.Path]::GetFileName($temporaryRoot) -notmatch "^[a-f0-9]{32}$"
        ) {
            throw "PowerShell syntax inventory contract failed."
        }
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

Write-Host "PowerShell syntax inventory contracts OK"
