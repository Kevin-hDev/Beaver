$ErrorActionPreference = "Stop"

function Assert-NativeEngine([bool]$Value) {
    if (-not $Value) { throw "Windows native icon engine contract failed." }
}

$referenceIconPath = [IO.Path]::GetFullPath(
    (Join-Path $PSScriptRoot "../../src-tauri/icons/icon.ico")
)
$probe = Join-Path $PSScriptRoot "windows-native-icon.test-probe.ps1"
$engines = @(
    [PSCustomObject]@{
        Name = "powershell.exe"
        Version = "^5\.1\."
        Edition = "Desktop"
    },
    [PSCustomObject]@{
        Name = "pwsh.exe"
        Version = "^7\."
        Edition = "Core"
    }
)
$results = @{}
foreach ($engineContract in $engines) {
    $engine = Get-Command $engineContract.Name -ErrorAction SilentlyContinue
    if ($null -eq $engine) {
        throw "Windows icon validation requires $($engineContract.Name) in PATH."
    }
    foreach ($size in @(16, 32)) {
        $json = & $engine.Source -NoProfile -NonInteractive -File $probe `
            -IconPath $referenceIconPath -Size $size
        Assert-NativeEngine ($LASTEXITCODE -eq 0)
        $result = $json | ConvertFrom-Json
        Assert-NativeEngine ($result.PSVersion -match $engineContract.Version)
        Assert-NativeEngine ($result.PSEdition -ceq $engineContract.Edition)
        Assert-NativeEngine ($result.Width -eq $size -and $result.Height -eq $size)
        Assert-NativeEngine ($result.Hashes.Count -eq 2)
        foreach ($hash in $result.Hashes) {
            Assert-NativeEngine ($hash -match "^[a-f0-9]{64}$")
        }
        $results["$($engineContract.Edition)-$size"] =
            [string]::Join("`n", $result.Hashes)
    }
}
foreach ($size in @(16, 32)) {
    Assert-NativeEngine ($results["Desktop-$size"] -ceq $results["Core-$size"])
}
Assert-NativeEngine ($results["Desktop-16"] -cne $results["Desktop-32"])

Write-Host "Windows native icon engine contracts OK"
