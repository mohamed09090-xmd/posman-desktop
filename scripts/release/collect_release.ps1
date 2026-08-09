param(
  [Parameter(Mandatory = $true)][string]$BundleDirectory,
  [Parameter(Mandatory = $true)][string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
$minimumBytes = 100MB
$maximumBytes = 200MB
$destinationName = 'POSMAN-Setup-Offline.exe'
$installer = Get-ChildItem -Path $BundleDirectory -Filter '*1.0.0*x64-setup.exe' -File |
  Sort-Object LastWriteTimeUtc -Descending |
  Select-Object -First 1
if (-not $installer) {
  throw "No POSMAN 1.0.0 x64 NSIS installer was found in $BundleDirectory"
}
if ($installer.Length -lt $minimumBytes) {
  throw "Installer is too small to contain the WebView2 offline payload: $($installer.Length) bytes"
}
if ($installer.Length -gt $maximumBytes) {
  throw "Installer exceeds the 200 MB release budget: $($installer.Length) bytes"
}

New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
$destination = Join-Path $OutputDirectory $destinationName
Copy-Item -LiteralPath $installer.FullName -Destination $destination -Force
$hash = Get-FileHash -LiteralPath $destination -Algorithm SHA256
"$($hash.Hash.ToLowerInvariant())  $destinationName" |
  Set-Content -Path (Join-Path $OutputDirectory 'SHA256SUMS.txt') -Encoding ascii

$signature = Get-AuthenticodeSignature -LiteralPath $destination
@{
  file = $destinationName
  version = '1.0.0'
  bytes = (Get-Item $destination).Length
  megabytes = [Math]::Round((Get-Item $destination).Length / 1MB, 3)
  sha256 = $hash.Hash.ToLowerInvariant()
  signatureStatus = [string]$signature.Status
  signer = if ($signature.SignerCertificate) { $signature.SignerCertificate.Subject } else { $null }
  webView2Mode = 'offlineInstaller'
  installerMode = 'perMachine'
} | ConvertTo-Json | Set-Content -Path (Join-Path $OutputDirectory 'release-metadata.json') -Encoding utf8

Write-Host "Release installer: $destination"
Write-Host "Installer bytes: $((Get-Item $destination).Length)"
Write-Host "SHA-256: $($hash.Hash)"
Write-Host "Authenticode: $($signature.Status)"
