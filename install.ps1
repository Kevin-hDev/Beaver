$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$Repository = "Kevin-hDev/Beaver"
$ApiUrl = "https://api.github.com/repos/$Repository/releases/latest"
$MaxApiBytes = 524288L; $MaxManifestBytes = 65536L; $MaxAssetBytes = 2147483648L
$TempDirectory = $null; $HttpClient = $null
function Write-Info([string]$Message) { Write-Host "→ $Message" -ForegroundColor Blue }
function Write-Ok([string]$Message) { Write-Host "✓ $Message" -ForegroundColor Green }
function Stop-Install { throw [InvalidOperationException]::new("installation failed") }
function Test-Version([string]$Value) {
    return $Value.Length -le 32 -and $Value -match "^(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})$"
}
function Test-Redirect([Uri]$Uri) {
    return $Uri.IsAbsoluteUri -and $Uri.Scheme -ceq "https" -and
        $Uri.Host -ceq "release-assets.githubusercontent.com" -and $Uri.IsDefaultPort -and
        [string]::IsNullOrEmpty($Uri.UserInfo) -and
        [string]::IsNullOrEmpty($Uri.Fragment) -and $Uri.OriginalString.Length -le 4096
}
function Save-BoundedFile {
    param([Uri]$Uri, [string]$Destination, [long]$Limit,
        [bool]$AllowRedirects, [int]$TimeoutSeconds)
    $current = $Uri
    $redirects = 0
    $part = "$Destination.part"
    try {
        while ($true) {
            $request = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::Get, $current)
            $response = $null
            $cancellation = [Threading.CancellationTokenSource]::new([TimeSpan]::FromSeconds($TimeoutSeconds))
            try {
                $response = $HttpClient.SendAsync(
                    $request, [Net.Http.HttpCompletionOption]::ResponseHeadersRead, $cancellation.Token
                ).GetAwaiter().GetResult()
                $status = [int]$response.StatusCode
                if (@(301, 302, 303, 307, 308) -contains $status) {
                    if (-not $AllowRedirects -or $redirects -ge 3 -or -not $response.Headers.Contains("Location")) { Stop-Install }
                    $locations = @($response.Headers.GetValues("Location"))
                    if ($locations.Count -ne 1) { Stop-Install }
                    $next = [Uri]($locations[0])
                    if (-not (Test-Redirect $next)) { Stop-Install }
                    $current = $next
                    $redirects += 1
                    continue
                }
                if ($status -ne 200) { Stop-Install }
                $declared = $response.Content.Headers.ContentLength
                if ($null -ne $declared -and ($declared -lt 1 -or $declared -gt $Limit)) { Stop-Install }
                if ([IO.File]::Exists($part)) { [IO.File]::Delete($part) }
                $inputStream = $response.Content.ReadAsStreamAsync().GetAwaiter().GetResult()
                $outputStream = [IO.FileStream]::new($part, [IO.FileMode]::CreateNew,
                    [IO.FileAccess]::Write, [IO.FileShare]::None)
                try {
                    $buffer = [byte[]]::new(65536)
                    $total = 0L
                    while (($read = $inputStream.ReadAsync(
                        $buffer, 0, $buffer.Length, $cancellation.Token
                    ).GetAwaiter().GetResult()) -gt 0) {
                        $total += $read
                        if ($total -gt $Limit) { Stop-Install }
                        $outputStream.Write($buffer, 0, $read)
                    }
                    if ($total -lt 1) { Stop-Install }
                    $outputStream.Flush($true)
                } finally {
                    $outputStream.Dispose(); $inputStream.Dispose()
                }
                [IO.File]::Move($part, $Destination)
                return $total
            } finally {
                if ($null -ne $response) { $response.Dispose() }
                $request.Dispose(); $cancellation.Dispose()
            }
        }
    } catch {
        if ([IO.File]::Exists($part)) { [IO.File]::Delete($part) }
        throw
    }
}
function Test-ExactProperties($Object, [string[]]$Names) {
    $actual = @($Object.PSObject.Properties.Name | Sort-Object)
    $expected = @($Names | Sort-Object)
    return [string]::Join("`n", $actual) -ceq [string]::Join("`n", $expected)
}
function Read-Json([string]$Path) {
    try { return [IO.File]::ReadAllText($Path) | ConvertFrom-Json }
    catch { Stop-Install }
}
function Get-Release([string]$Path) {
    $release = Read-Json $Path
    foreach ($property in @("tag_name", "draft", "prerelease", "assets")) {
        if (-not $release.PSObject.Properties[$property]) { Stop-Install }
    }
    if (-not ($release.draft -is [bool]) -or $release.draft -or
        -not ($release.prerelease -is [bool]) -or $release.prerelease) { Stop-Install }
    if ($release.tag_name -notmatch "^v(.+)$") { Stop-Install }
    $version = $Matches[1]
    if (-not (Test-Version $version)) { Stop-Install }
    $assetName = "Beaver_${version}_x64-setup.exe"
    $manifestName = "update-manifest.json"
    $baseUrl = "https://github.com/$Repository/releases/download/v$version"
    $assets = @($release.assets)
    if ($assets.Count -lt 1 -or $assets.Count -gt 64) { Stop-Install }
    $asset = @($assets | Where-Object { $_.name -ceq $assetName -and
        $_.browser_download_url -ceq "$baseUrl/$assetName" })
    $manifest = @($assets | Where-Object { $_.name -ceq $manifestName -and
        $_.browser_download_url -ceq "$baseUrl/$manifestName" })
    if ($asset.Count -ne 1 -or $manifest.Count -ne 1) { Stop-Install }
    foreach ($item in @($asset[0], $manifest[0])) {
        if (-not ($item.size -is [int] -or $item.size -is [long]) -or
            $item.size -lt 1 -or $item.size -gt $MaxAssetBytes) { Stop-Install }
    }
    if ($manifest[0].size -gt $MaxManifestBytes) { Stop-Install }
    return [PSCustomObject]@{ Version = $version; Asset = $asset[0]; Manifest = $manifest[0] }
}
function Get-ManifestAsset([string]$Path, [string]$Version, [string]$ExpectedName) {
    $manifest = Read-Json $Path
    if (-not (Test-ExactProperties $manifest @("version", "assets")) -or $manifest.version -cne $Version) { Stop-Install }
    $assets = @($manifest.assets)
    if ($assets.Count -lt 1 -or $assets.Count -gt 16) { Stop-Install }
    $seen = @()
    $selected = @()
    foreach ($asset in $assets) {
        if (-not (Test-ExactProperties $asset @("name", "sha256", "size")) -or
            $asset.name -notmatch "^Beaver_$([regex]::Escape($Version))_[A-Za-z0-9._-]+$" -or $asset.sha256 -notmatch "^[0-9a-f]{64}$" -or
            -not ($asset.size -is [int] -or $asset.size -is [long]) -or
            $asset.size -lt 1 -or $asset.size -gt $MaxAssetBytes -or $seen -ccontains $asset.name) { Stop-Install }
        $seen += $asset.name
        if ($asset.name -ceq $ExpectedName) { $selected += $asset }
    }
    if ($selected.Count -ne 1) { Stop-Install }
    return $selected[0]
}
function Test-InstallPath([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path) -or $Path.Length -gt 1024 -or $Path -notmatch "^[A-Za-z]:\\" -or $Path -match '[*?<>|"]' -or $Path.Substring(3).Contains(":")) { return $false }
    foreach ($character in $Path.ToCharArray()) {
        if ([char]::IsControl($character)) { return $false }
    }
    return -not (@($Path.Split([IO.Path]::DirectorySeparatorChar)) -contains "..")
}
function Invoke-Main {
    if ($env:PROCESSOR_ARCHITECTURE -cne "AMD64") { Stop-Install }
    Add-Type -AssemblyName System.Net.Http
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $handler = [Net.Http.HttpClientHandler]::new()
    $handler.AllowAutoRedirect = $false; $handler.MaxResponseHeadersLength = 64
    $script:HttpClient = [Net.Http.HttpClient]::new($handler)
    $HttpClient.Timeout = [Threading.Timeout]::InfiniteTimeSpan
    [void]$HttpClient.DefaultRequestHeaders.UserAgent.ParseAdd("Beaver-Installer/1")
    [void]$HttpClient.DefaultRequestHeaders.TryAddWithoutValidation("Accept-Encoding", "identity")
    $root = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    $script:TempDirectory = [IO.Path]::Combine($root, "beaver-install-$([Guid]::NewGuid().ToString('N'))")
    [void][IO.Directory]::CreateDirectory($TempDirectory)
    $releasePath = [IO.Path]::Combine($TempDirectory, "release.json")
    [void](Save-BoundedFile ([Uri]$ApiUrl) $releasePath $MaxApiBytes $false 30)
    $release = Get-Release $releasePath
    $manifestPath = [IO.Path]::Combine($TempDirectory, "update-manifest.json")
    $manifestUri = [Uri]$release.Manifest.browser_download_url
    $manifestBytes = Save-BoundedFile $manifestUri $manifestPath $MaxManifestBytes $true 30
    if ($manifestBytes -ne [long]$release.Manifest.size) { Stop-Install }
    $expected = Get-ManifestAsset $manifestPath $release.Version $release.Asset.name
    if ([long]$release.Asset.size -ne [long]$expected.size) { Stop-Install }
    $assetPath = [IO.Path]::Combine($TempDirectory, $release.Asset.name)
    $assetBytes = Save-BoundedFile ([Uri]$release.Asset.browser_download_url) $assetPath ([long]$expected.size) $true 1800
    if ($assetBytes -ne [long]$expected.size) { Stop-Install }
    $hash = (Get-FileHash -LiteralPath $assetPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -cne $expected.sha256) { Stop-Install }
    $defaultDirectory = [Environment]::GetFolderPath("LocalApplicationData")
    $defaultDirectory = [IO.Path]::Combine($defaultDirectory, "Beaver")
    Write-Host ""
    Write-Host "📁 Répertoire d'installation : $defaultDirectory" -ForegroundColor Yellow
    $customDirectory = Read-Host "   Appuie sur Entrée pour accepter, ou tape un autre chemin"
    $installDirectory = $customDirectory
    if ([string]::IsNullOrWhiteSpace($installDirectory)) { $installDirectory = $defaultDirectory }
    if (-not (Test-InstallPath $installDirectory)) { Stop-Install }
    Write-Info "Installation de Beaver v$($release.Version)..."
    $process = Start-Process -FilePath $assetPath -ArgumentList @("/S", "/D=$installDirectory") -Wait -PassThru -WindowStyle Hidden
    if ($process.ExitCode -ne 0) { Stop-Install }
    $binary = [IO.Path]::Combine($installDirectory, "cl-go-dash.exe")
    if (-not [IO.File]::Exists($binary) -or ([IO.File]::GetAttributes($binary) -band [IO.FileAttributes]::ReparsePoint)) { Stop-Install }
    Write-Ok "Beaver v$($release.Version) est installé."
}
try {
    Invoke-Main
} catch {
    Write-Host "✗ Installation impossible." -ForegroundColor Red
    exit 1
} finally {
    if ($null -ne $HttpClient) { $HttpClient.Dispose() }
    if ($null -ne $TempDirectory) {
        try {
            $full = [IO.Path]::GetFullPath($TempDirectory)
            $parent = [IO.Directory]::GetParent($full).FullName
            $tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd("\")
            if ($parent.TrimEnd("\") -ceq $tempRoot -and [IO.Path]::GetFileName($full) -match "^beaver-install-[0-9a-f]{32}$") {
                [IO.Directory]::Delete($full, $true)
            }
        } catch {}
    }
}
