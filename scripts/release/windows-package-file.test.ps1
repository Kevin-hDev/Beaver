$ErrorActionPreference = "Stop"

function Assert-PackageTrue([bool]$Value) {
    if (-not $Value) { throw "Windows package file contract failed." }
}

function Assert-PackageFalse([bool]$Value) {
    if ($Value) { throw "Windows package file contract failed." }
}

. (Join-Path $PSScriptRoot "windows-package-file.ps1")

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
    $validFile = Join-Path $temporaryRoot "valid.exe"
    $emptyFile = Join-Path $temporaryRoot "empty.exe"
    [IO.File]::WriteAllBytes($validFile, [byte[]]@(1, 2, 3))
    [IO.File]::WriteAllBytes($emptyFile, [byte[]]@())
    Assert-PackageTrue (Test-BoundedPackageFile $validFile 3)
    Assert-PackageFalse (Test-BoundedPackageFile $emptyFile 3)
    $bytes = [byte[]](Read-BoundedPackageBytes $validFile 3)
    Assert-PackageTrue ($bytes.Count -eq 3 -and $bytes[2] -eq 3)
    Assert-PackageFalse (Test-BoundedPackageFile $validFile 2)

    $missingFile = Join-Path $temporaryRoot "missing.exe"
    $boundaryOutput = @(& {
        $ErrorActionPreference = "Continue"
        try {
            [void](Get-BoundedPackageFile $missingFile 3)
        } catch {
            $_.Exception.Message
        }
    } 2>&1)
    Assert-PackageTrue ($boundaryOutput.Count -eq 1)
    Assert-PackageTrue ([string]$boundaryOutput[0] -ceq "Invalid package file.")

    $physicalParent = Join-Path $temporaryRoot "physical-parent"
    $allowedPhysical = Join-Path $physicalParent "allowed"
    [void](New-Item -ItemType Directory -Path $allowedPhysical -Force)
    $linkedParent = Join-Path $temporaryRoot "linked-parent"
    [void](New-Item -ItemType Junction -Path $linkedParent -Target $physicalParent)
    $linkedAllowed = Join-Path $linkedParent "allowed"
    $linkedFile = Join-Path $linkedAllowed "linked.exe"
    [IO.File]::WriteAllBytes((Join-Path $allowedPhysical "linked.exe"), [byte[]]@(1))
    Assert-PackageFalse (Test-BoundedPackageFile $linkedFile 3)
    Assert-PackageFalse (Test-BoundedPackageFile $linkedFile 3 $linkedAllowed)
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        if (
            [IO.Path]::GetDirectoryName($temporaryRoot) -cne $temporaryBase -or
            [IO.Path]::GetFileName($temporaryRoot) -notmatch "^[a-f0-9]{32}$"
        ) {
            throw "Windows package file contract failed."
        }
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

Write-Host "Windows package file contracts OK"
