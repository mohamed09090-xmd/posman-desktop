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
$localDataRoot = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
if ([string]::IsNullOrWhiteSpace($localDataRoot)) {
  throw 'Windows did not resolve FOLDERID_LocalAppData for the disposable runner user.'
}
$dataRoot = Join-Path $localDataRoot 'POSMAN'
$databasePath = Join-Path $dataRoot 'data\posman.sqlite3'
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

function Wait-ForRuntimeDatabase(
  [Diagnostics.Process]$Process,
  [string]$Path,
  [int]$TimeoutSeconds = 30
) {
  $watch = [Diagnostics.Stopwatch]::StartNew()
  while ($watch.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
    $Process.Refresh()
    if ($Process.HasExited) {
      throw "POSMAN exited before its local runtime became ready (exit $($Process.ExitCode))"
    }
    if ((Test-Path -LiteralPath $Path) -and (Get-Item -LiteralPath $Path).Length -gt 0) {
      return $watch.Elapsed.TotalMilliseconds
    }
    Start-Sleep -Milliseconds 100
  }

  $diagnosticRoots = @(
    $dataRoot,
    (Join-Path $localDataRoot 'dz.posman.desktop')
  ) | Where-Object { Test-Path -LiteralPath $_ }
  $candidates = @(
    foreach ($root in $diagnosticRoots) {
      Get-ChildItem -LiteralPath $root -Filter 'posman.sqlite3' -File -Recurse -ErrorAction SilentlyContinue |
        ForEach-Object FullName
    }
  )
  $candidateText = if ($candidates.Count -eq 0) { '<none>' } else { $candidates -join '; ' }
  throw "POSMAN did not create the expected local runtime database within $TimeoutSeconds seconds. Expected: $Path. Candidates: $candidateText"
}

function Start-And-Measure([string]$Executable, [string]$RuntimeDatabase) {
  $watch = [Diagnostics.Stopwatch]::StartNew()
  $process = Start-Process -FilePath $Executable -PassThru
  try {
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
      throw 'POSMAN cold start exceeded the 5 second main-window target'
    }
    $startupMs = $watch.Elapsed.TotalMilliseconds
    $runtimeReadyMs = Wait-ForRuntimeDatabase -Process $process -Path $RuntimeDatabase
    Start-Sleep -Seconds 2
    $process.Refresh()
    if ($process.HasExited) {
      throw "POSMAN exited after creating its main window (exit $($process.ExitCode))"
    }
    $workingSetMb = $process.WorkingSet64 / 1MB
    if ($workingSetMb -ge 250) {
      throw "POSMAN working set exceeded 250 MB: $workingSetMb MB"
    }
    return @{
      startupMs = [Math]::Round($startupMs, 3)
      runtimeReadyMs = [Math]::Round($runtimeReadyMs, 3)
      workingSetMb = [Math]::Round($workingSetMb, 3)
    }
  } finally {
    if (-not $process.HasExited) {
      Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
      $process.WaitForExit(10000) | Out-Null
    }
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
$fixtureMetrics = Start-And-Measure -Executable $application -RuntimeDatabase $databasePath
if (-not (Test-Path -LiteralPath $databasePath)) {
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
$releaseMetrics = Start-And-Measure -Executable $application -RuntimeDatabase $databasePath
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
