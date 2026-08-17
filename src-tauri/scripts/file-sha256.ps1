function Get-BeaverFileSha256([string] $Path) {
  if ([string]::IsNullOrWhiteSpace($Path)) {
    throw "CEF runtime validation failed"
  }

  $Stream = $null
  $Sha256 = $null
  try {
    $Stream = [IO.File]::Open(
      $Path,
      [IO.FileMode]::Open,
      [IO.FileAccess]::Read,
      [IO.FileShare]::Read
    )
    # Tauri can pass PowerShell 7 module paths to Windows PowerShell 5.1,
    # so hashing must not depend on the ambient Get-FileHash command.
    $Sha256 = [Security.Cryptography.SHA256]::Create()
    $Digest = $Sha256.ComputeHash($Stream)
    return [BitConverter]::ToString($Digest).Replace("-", "").ToLowerInvariant()
  } finally {
    if ($null -ne $Sha256) { $Sha256.Dispose() }
    if ($null -ne $Stream) { $Stream.Dispose() }
  }
}
