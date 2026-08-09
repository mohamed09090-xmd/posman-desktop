param(
  [Parameter(Mandatory = $true)][string]$FixtureInstaller,
  [Parameter(Mandatory = $true)][string]$ReleaseInstaller,
  [Parameter(Mandatory = $true)][string]$EvidenceDirectory,
  [switch]$ConfirmDisposableRunner
)

$ErrorActionPreference = 'Stop'
if (-not $ConfirmDisposableRunner -or $env:GITHUB_ACTIONS -ne 'true') {
  throw 'Installer lifecycle testing is destructive to an existing POSMAN installation and is allowed only on a disposable GitHub Actions runner.'
}

$installRoot = Join-Path $env:ProgramFiles 'POSMAN'
$application = Join-Path $installRoot 'posman-desktop.exe'
$uninstaller = Join-Path $installRoot 'uninstall.exe'
$dataRoot = Join-Path $env:LOCALAPPDATA 'POSMAN'
$sentinels = @(
  (Join-Path $dataRoot 'data\phase10-preserve.sentinel'),
  (Join-Path $dataRoot 'backups\phase10-preserve.sentinel'),
  (Join-Path $dataRoot 'documents\phase10-preserve.sentinel')
)

function Invoke-Installer([string]$Path, [string[]]$Arguments) {
  $process = Start-Process -FilePath $Path -ArgumentList $Arguments -Wait -PassThru
  if ($process.ExitCode -ne 0) {
    throw "Installer failed with exit code $($process.ExitCode): $Path $Arguments"
  }
}

function Start-And-Measure([string]$Executable) {
  $watch = [Diagnostics.Stopwatch]::StartNew()
  $process = Start-Process -FilePath $Executable -PassThru
  $windowReady = $false
  while ($watch.Elapsed.TotalSeconds -lt 5) {
    Start-Sleep -Milliseconds 100
    $process.Refresh()
    if ($process.HasExited) {
      throw "POSMAN exited before creating its main window (exit $($process.ExitCode))"
    }
    if ($process.MainWindowHandle -ne 0) {
      $windowReady = $true
      break
    }
  }
  if (-not $windowReady) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    throw 'POSMAN cold start exceeded the 5 second main-window target'
  }
  $startupMs = $watch.Elapsed.TotalMilliseconds
  Start-Sleep -Seconds 2
  $process.Refresh()
  $workingSetMb = $process.WorkingSet64 / 1MB
  Stop-Process -Id $process.Id -Force
  $process.WaitForExit(10000) | Out-Null
  if ($workingSetMb -ge 250) {
    throw "POSMAN working set exceeded 250 MB: $workingSetMb MB"
  }
  return @{
    startupMs = [Math]::Round($startupMs, 3)
    workingSetMb = [Math]::Round($workingSetMb, 3)
  }
}

New-Item -ItemType Directory -Force $EvidenceDirectory | Out-Null
if (Test-Path $application) {
  throw "The disposable runner is not clean; POSMAN already exists at $application"
}

Invoke-Installer $FixtureInstaller @('/S')
if (-not (Test-Path $application) -or -not (Test-Path $uninstaller)) {
  throw 'The v0.9.0 upgrade fixture did not install POSMAN under Program Files'
}
$fixtureMetrics = Start-And-Measure $application
if (-not (Test-Path (Join-Path $dataRoot 'data\posman.sqlite3'))) {
  throw 'First launch did not create the local POSMAN database'
}
foreach ($sentinel in $sentinels) {
  New-Item -ItemType Directory -Force (Split-Path $sentinel -Parent) | Out-Null
  'preserve-across-upgrade-and-uninstall' | Set-Content -Path $sentinel -Encoding ascii
}

Invoke-Installer $ReleaseInstaller @('/S', '/UPDATE')
if (-not (Test-Path $application)) {
  throw 'POSMAN executable disappeared during the v1.0.0 upgrade'
}
$version = (Get-Item $application).VersionInfo.ProductVersion
if ($version -notlike '1.0.0*') {
  throw "Installed application does not report v1.0.0 after upgrade: $version"
}
$releaseMetrics = Start-And-Measure $application
foreach ($sentinel in $sentinels) {
  if ((Get-Content -Raw $sentinel).Trim() -ne 'preserve-across-upgrade-and-uninstall') {
    throw "Data sentinel was lost or changed during upgrade: $sentinel"
  }
}

Invoke-Installer $uninstaller @('/S')
for ($attempt = 0; $attempt -lt 100 -and (Test-Path $application); $attempt++) {
  Start-Sleep -Milliseconds 100
}
if (Test-Path $application) {
  throw 'The application executable remains after silent uninstall'
}
foreach ($sentinel in $sentinels) {
  if ((Get-Content -Raw $sentinel).Trim() -ne 'preserve-across-upgrade-and-uninstall') {
    throw "Uninstall removed or changed protected business data: $sentinel"
  }
}

@{
  fixtureVersion = '0.9.0'
  releaseVersion = '1.0.0'
  installRoot = $installRoot
  dataRoot = $dataRoot
  databaseCreated = $true
  upgradePreservedData = $true
  uninstallPreservedData = $true
  programRemoved = $true
  fixtureMetrics = $fixtureMetrics
  releaseMetrics = $releaseMetrics
  startupTargetMs = 5000
  workingSetTargetMb = 250
} | ConvertTo-Json -Depth 4 |
  Set-Content -Path (Join-Path $EvidenceDirectory 'installer-lifecycle.json') -Encoding utf8

Write-Host 'PHASE 10 installer lifecycle PASS: clean install, upgrade, launch, uninstall, and LocalAppData preservation.'
