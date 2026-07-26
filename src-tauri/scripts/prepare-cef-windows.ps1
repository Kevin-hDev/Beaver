$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$ExpectedBootstrapSha256 = "eab5d939293a666b210b8f5faec191324a017d6105485cfc45150863607bd367"
$ExpectedLicenseSha256 = "2e69c36a7eaa4fa153426eab635c607ea0356cbc7a68a70f42a49e8ab8eb8106"
$ExpectedCreditsSha256 = "333620129bfec11001385ea24d68de049ce0eeb8d012d2a1382b5340d7d62daf"
$MaxFileBytes = 536870912

$BinaryFiles = @(
  "chrome_elf.dll",
  "d3dcompiler_47.dll",
  "dxcompiler.dll",
  "dxil.dll",
  "libEGL.dll",
  "libGLESv2.dll",
  "libcef.dll",
  "v8_context_snapshot.bin",
  "vk_swiftshader.dll",
  "vk_swiftshader_icd.json",
  "vulkan-1.dll"
)
$ResourceFiles = @(
  "chrome_100_percent.pak",
  "chrome_200_percent.pak",
  "icudtl.dat",
  "resources.pak"
)
$LocaleFiles = @("de.pak", "en-US.pak", "es.pak", "fr.pak", "it.pak", "ja.pak", "zh-CN.pak")

function Assert-RegularFile([string] $Path) {
  $Item = Get-Item -LiteralPath $Path -Force
  if (-not $Item.PSIsContainer -and
      -not ($Item.Attributes -band [IO.FileAttributes]::ReparsePoint) -and
      $Item.Length -gt 0 -and $Item.Length -le $MaxFileBytes) {
    return
  }
  throw "CEF runtime validation failed"
}

function Copy-CheckedFile([string] $Source, [string] $Destination) {
  Assert-RegularFile $Source
  Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

$TauriDir = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$PrepareSource = Join-Path $TauriDir "..\scripts\cef\prepare-cef-source.mjs"
& node $PrepareSource
if ($LASTEXITCODE -ne 0) {
  throw "CEF runtime validation failed"
}
$BuildTarget = $env:CARGO_BUILD_TARGET
if ([string]::IsNullOrWhiteSpace($BuildTarget)) {
  $TargetDir = Join-Path $TauriDir "target\release"
} else {
  if ($BuildTarget -ne "x86_64-pc-windows-msvc") {
    throw "CEF runtime validation failed"
  }
  $TargetDir = Join-Path $TauriDir "target\$BuildTarget\release"
}
$CargoManifest = Join-Path $TauriDir "Cargo.toml"
& cargo build --release --lib --features tauri/custom-protocol --manifest-path $CargoManifest
if ($LASTEXITCODE -ne 0) {
  throw "CEF runtime validation failed"
}
$StageDir = Join-Path $TauriDir "target\cef-runtime\windows"
$CefRoot = Join-Path $TauriDir ".cef-verified\current"
if (-not (Test-Path -LiteralPath $CefRoot -PathType Container)) {
  throw "CEF runtime validation failed"
}

$Bootstrap = Join-Path $CefRoot "bootstrap.exe"
Assert-RegularFile $Bootstrap
$BootstrapSha256 = (Get-FileHash -LiteralPath $Bootstrap -Algorithm SHA256).Hash.ToLowerInvariant()
if ($BootstrapSha256 -ne $ExpectedBootstrapSha256) {
  throw "CEF runtime validation failed"
}

if (Test-Path -LiteralPath $StageDir) {
  Remove-Item -LiteralPath $StageDir -Recurse -Force
}
New-Item -ItemType Directory -Path $StageDir | Out-Null
$LocalesDir = Join-Path $StageDir "locales"
$LicensesDir = Join-Path $StageDir "cef"
New-Item -ItemType Directory -Path $LocalesDir | Out-Null
New-Item -ItemType Directory -Path $LicensesDir | Out-Null

foreach ($Name in $BinaryFiles) {
  Copy-CheckedFile (Join-Path $CefRoot $Name) (Join-Path $StageDir $Name)
}
foreach ($Name in $ResourceFiles) {
  Copy-CheckedFile (Join-Path $CefRoot $Name) (Join-Path $StageDir $Name)
}
foreach ($Name in $LocaleFiles) {
  Copy-CheckedFile (Join-Path $CefRoot "locales\$Name") (Join-Path $LocalesDir $Name)
}

$Credits = Join-Path $CefRoot "CREDITS.html"
Copy-CheckedFile $Credits (Join-Path $LicensesDir "CREDITS.html")
if ((Get-FileHash -LiteralPath $Credits -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedCreditsSha256) {
  throw "CEF runtime validation failed"
}

$License = Join-Path $CefRoot "LICENSE.txt"
Assert-RegularFile $License
if ((Get-FileHash -LiteralPath $License -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedLicenseSha256) {
  throw "CEF runtime validation failed"
}
Copy-Item -LiteralPath $License -Destination (Join-Path $LicensesDir "LICENSE.txt") -Force

$ApplicationDll = Join-Path $TargetDir "cl_go_dash_lib.dll"
Copy-CheckedFile $ApplicationDll (Join-Path $StageDir "cl-go-dash.dll")
Copy-Item -LiteralPath $Bootstrap -Destination (Join-Path $TargetDir "cl-go-dash.exe") -Force
Write-Host "CEF 150 Windows runtime prepared"
