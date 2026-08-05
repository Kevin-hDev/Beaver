$ErrorActionPreference = "Stop"

$TauriDir = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$RepositoryRoot = [IO.Path]::GetFullPath((Join-Path $TauriDir ".."))
$SourceExecutable = Join-Path $TauriDir "target\release\cl-go-dash-helper.exe"
$Bootstrap = Join-Path $TauriDir ".cef-verified\current\bootstrap.exe"
$BrandScript = Join-Path $PSScriptRoot "copy-windows-brand-resources.ps1"
$Version = "1.1.1"

. (Join-Path $RepositoryRoot "scripts\release\windows-artifact-helpers.ps1")

$randomBytes = New-Object byte[] 16
$random = [Security.Cryptography.RandomNumberGenerator]::Create()
try {
    $random.GetBytes($randomBytes)
} finally {
    $random.Dispose()
}
$name = -join @($randomBytes | ForEach-Object { $_.ToString("x2") })
$temporaryBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd("\")
$temporaryRoot = [IO.Path]::GetFullPath((Join-Path $temporaryBase $name))

try {
    [void](New-Item -ItemType Directory -Path $temporaryRoot)
    $destination = Join-Path $temporaryRoot "cl-go-dash.exe"
    Copy-Item -LiteralPath $Bootstrap -Destination $destination
    $beforeHash = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash

    if (Test-BeaverExecutableBrand $destination $Version) {
        throw "Windows branding test failed."
    }

    Push-Location $RepositoryRoot
    try {
        $relativeAccepted = $true
        try {
            & $BrandScript `
                -SourceExecutable "src-tauri\target\release\cl-go-dash-helper.exe" `
                -DestinationExecutable $destination `
                -ExpectedProductName "Beaver" `
                -ExpectedVersion $Version
        } catch {
            $relativeAccepted = $false
        }
    } finally {
        Pop-Location
    }
    if ($relativeAccepted) {
        throw "Windows branding test failed."
    }

    & $BrandScript `
        -SourceExecutable $SourceExecutable `
        -DestinationExecutable $destination `
        -ExpectedProductName "Beaver" `
        -ExpectedVersion $Version

    $afterHash = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash
    if ($beforeHash -ceq $afterHash -or -not (Test-BeaverExecutableBrand $destination $Version)) {
        throw "Windows branding test failed."
    }
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        if (
            [IO.Path]::GetDirectoryName($temporaryRoot) -cne $temporaryBase -or
            [IO.Path]::GetFileName($temporaryRoot) -notmatch "^[a-f0-9]{32}$"
        ) {
            throw "Windows branding test failed."
        }
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

Write-Host "Windows bootstrap branding OK"
