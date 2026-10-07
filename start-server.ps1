param([switch]$Restart, [switch]$Build, [switch]$Foreground, [string]$DataRoot = (Join-Path $PSScriptRoot 'data\online'), [string[]]$Origin = @(), [switch]$TrustedProxy)
$ErrorActionPreference = 'Stop'
$binary = Join-Path $PSScriptRoot 'target\release\sternenepoche-server.exe'
$dataPath = [IO.Path]::GetFullPath($DataRoot)
New-Item -ItemType Directory -Path $dataPath -Force | Out-Null
$publicUrlFile = Join-Path $dataPath 'public-url.txt'
if (Test-Path -LiteralPath $publicUrlFile) {
    $publicUrl = (Get-Content -LiteralPath $publicUrlFile -Raw).Trim()
    $publicUri = [Uri]$publicUrl
    if ($publicUri.Scheme -ne 'https' -or $publicUri.UserInfo -or $publicUri.AbsolutePath -ne '/' -or $publicUri.Query -or $publicUri.Fragment) { throw 'public-url.txt muss eine HTTPS-Adresse ohne Pfad oder Zugangsdaten enthalten.' }
    $Origin += $publicUri.GetLeftPart([UriPartial]::Authority)
}

$pidFile = Join-Path $dataPath 'server.pid'
if (Test-Path -LiteralPath $pidFile) {
    $oldServerId = Get-Content -LiteralPath $pidFile -ErrorAction SilentlyContinue
    $oldServer = Get-Process -Id $oldServerId -ErrorAction SilentlyContinue
    if ($oldServer -and $oldServer.Path -eq $binary) {
        if (!$Restart) {
            Write-Host 'Dieser Server läuft bereits. Spiel: http://127.0.0.1:8890 · Dashboard: http://127.0.0.1:8891'
            exit 0
        }
        Stop-Process -Id $oldServer.Id
        $oldServer.WaitForExit()
    }
}
if ($Build -or !(Test-Path -LiteralPath $binary)) {
    Push-Location $PSScriptRoot
    try { & cargo build -p sternenepoche-server --release; if ($LASTEXITCODE -ne 0) { throw 'Rust-Build fehlgeschlagen.' } } finally { Pop-Location }
}
$arguments = @('--data', ('"' + $dataPath + '"'))
foreach ($value in $Origin) { $arguments += @('--origin', ('"' + $value + '"')) }
if ($TrustedProxy) { $arguments += '--trusted-proxy' }
if ($Foreground) {
    $foregroundArgs = @('--data', $dataPath)
    foreach ($value in $Origin) { $foregroundArgs += @('--origin', $value) }
    if ($TrustedProxy) { $foregroundArgs += '--trusted-proxy' }
    & $binary @foregroundArgs; exit $LASTEXITCODE
}
$process = Start-Process -FilePath $binary -ArgumentList $arguments -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $dataPath 'server.log') -RedirectStandardError (Join-Path $dataPath 'server-error.log')
Start-Sleep -Milliseconds 800
$process.Refresh()
if ($process.HasExited) { throw ('Serverstart fehlgeschlagen. Siehe ' + (Join-Path $dataPath 'server-error.log')) }
Set-Content -LiteralPath $pidFile -Value $process.Id
Write-Host 'Spiel: http://127.0.0.1:8890'
Write-Host 'Lokales Dashboard: http://127.0.0.1:8891'
Write-Host ('Daten: ' + $dataPath)
