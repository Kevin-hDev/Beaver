[CmdletBinding()]
param(
    [switch]$ListOnly,
    [string]$RepositoryRoot = ""
)

$ErrorActionPreference = "Stop"
$MaxPowerShellScripts = 256
$repositoryRoot = if ([string]::IsNullOrWhiteSpace($RepositoryRoot)) {
    [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
} else {
    [IO.Path]::GetFullPath($RepositoryRoot)
}
$repositoryPrefix = $repositoryRoot.TrimEnd(
    [IO.Path]::DirectorySeparatorChar,
    [IO.Path]::AltDirectorySeparatorChar
) + [IO.Path]::DirectorySeparatorChar

# Git emits UTF-8; force its decoding only for this bounded inventory and restore the console.
$previousConsoleOutputEncoding = [Console]::OutputEncoding
try {
    [Console]::OutputEncoding = New-Object Text.UTF8Encoding($false)
    $relativePaths = @(
        & git -C $repositoryRoot -c core.quotepath=false `
            ls-files --cached --others --exclude-standard -- `
            ":(icase,glob)*.ps1" ":(icase,glob)**/*.ps1"
    )
} finally {
    [Console]::OutputEncoding = $previousConsoleOutputEncoding
}
if ($LASTEXITCODE -ne 0 -or $relativePaths.Count -le 0 -or
    $relativePaths.Count -gt $MaxPowerShellScripts) {
    throw "PowerShell syntax inventory failed."
}

$paths = foreach ($relativePath in $relativePaths) {
    if (
        [string]::IsNullOrWhiteSpace($relativePath) -or
        $relativePath.Length -gt 4096 -or
        [IO.Path]::IsPathRooted($relativePath) -or
        $relativePath -match "(^|[\\/])\.\.([\\/]|$)" -or
        [IO.Path]::GetExtension($relativePath) -ine ".ps1"
    ) {
        throw "PowerShell syntax inventory failed."
    }
    $path = [IO.Path]::GetFullPath((Join-Path $repositoryRoot $relativePath))
    if (-not $path.StartsWith($repositoryPrefix, [StringComparison]::OrdinalIgnoreCase) -or
        -not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "PowerShell syntax inventory failed."
    }
    $path
}

if ($ListOnly) {
    $paths
    return
}

foreach ($path in $paths) {
    $tokens = $null
    $errors = $null
    [void][System.Management.Automation.Language.Parser]::ParseFile(
        $path,
        [ref]$tokens,
        [ref]$errors
    )
    if ($errors.Count -ne 0) {
        throw "PowerShell syntax invalid."
    }
}

$previousTestMode = $env:BEAVER_INSTALLER_TEST_MODE
$env:BEAVER_INSTALLER_TEST_MODE = "1"
. (Join-Path $repositoryRoot "install.ps1")
$runRoot = Join-Path ([IO.Path]::GetTempPath()) "beaver-purge-test-$([Guid]::NewGuid().ToString('N'))"
$sentinel = Join-Path ([IO.Path]::GetTempPath()) "beaver-purge-sentinel-$([Guid]::NewGuid().ToString('N'))"
$junction = $null
$validId = "11111111111111111111111111111111"
$forgedId = "22222222222222222222222222222222"
$junctionId = "33333333333333333333333333333333"
try {
    [void][IO.Directory]::CreateDirectory($runRoot)
    [void][IO.Directory]::CreateDirectory($sentinel)
    [IO.File]::WriteAllText((Join-Path $sentinel "outside.txt"), "outside")
    foreach ($id in @($validId, $forgedId, $junctionId)) {
        $run = Join-Path $runRoot "beaver-install-$id"
        [void][IO.Directory]::CreateDirectory($run)
        $markerId = if ($id -ceq $forgedId) { "wrong" } else { $id }
        [IO.File]::WriteAllText((Join-Path $run ".beaver-installer-owner.json"),
            "{`"schema`":1,`"runId`":`"$markerId`"}")
    }
    $junction = Join-Path $runRoot "beaver-install-$junctionId\outside"
    [void](New-Item -ItemType Junction -Path $junction -Target $sentinel)
    if (-not (Test-OwnedRun $runRoot (Join-Path $runRoot "beaver-install-$validId") $validId) -or
        (Test-OwnedRun $runRoot (Join-Path $runRoot "beaver-install-$forgedId") $forgedId) -or
        (Test-OwnedRun $runRoot (Join-Path $runRoot "beaver-install-$junctionId") $junctionId) -or
        [IO.File]::ReadAllText((Join-Path $sentinel "outside.txt")) -cne "outside") {
        throw "PowerShell owned-run validation failed."
    }
} finally {
    if (Test-Path $junction) { [IO.Directory]::Delete($junction) }
    if (Test-Path $runRoot) { [IO.Directory]::Delete($runRoot, $true) }
    if (Test-Path $sentinel) { [IO.Directory]::Delete($sentinel, $true) }
    $env:BEAVER_INSTALLER_TEST_MODE = $previousTestMode
}

Write-Host "PowerShell syntax OK"
