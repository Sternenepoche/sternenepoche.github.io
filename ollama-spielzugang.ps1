param([switch]$Restart, [string]$GameOrigin = 'https://desktop-3dei636.taila4f584.ts.net')
$ErrorActionPreference = 'Stop'
$uri = [Uri]$GameOrigin
if ($uri.Scheme -ne 'https' -or $uri.UserInfo -or $uri.AbsolutePath -ne '/' -or $uri.Query -or $uri.Fragment) { throw 'Eine genaue HTTPS-Spieladresse ohne Zugangsdaten angeben.' }
$GameOrigin = $uri.GetLeftPart([UriPartial]::Authority)
$oldOrigins = [Environment]::GetEnvironmentVariable('OLLAMA_ORIGINS','User')
$existing = @($oldOrigins, [Environment]::GetEnvironmentVariable('OLLAMA_ORIGINS','Machine'), $env:OLLAMA_ORIGINS) | Where-Object { $_ }
$origins = @($existing | ForEach-Object { $_.Split(',') } | ForEach-Object { $_.Trim() } | Where-Object { $_ })
$origins += @('http://127.0.0.1:8890','http://localhost:8890','https://sternenepoche.github.io',$GameOrigin)
$value = ($origins | Select-Object -Unique) -join ','
$evidence = Join-Path $PSScriptRoot 'data\online'
New-Item -ItemType Directory -Path $evidence -Force | Out-Null
$backup = Join-Path $evidence 'ollama-origins-before.json'
if (!(Test-Path -LiteralPath $backup)) { @{ UserOrigins=$oldOrigins } | ConvertTo-Json | Set-Content -LiteralPath $backup -Encoding utf8 }
if ($Restart) {
    if (@(Get-NetTCPConnection -LocalPort 11434 -State Established -ErrorAction SilentlyContinue).Count) { throw 'Ollama hat aktive Verbindungen. Den Neustart erst nach Abschluss der Modellanfragen ausführen.' }
    $binary = (Get-Command ollama.exe -ErrorAction Stop).Source
    $listener = Get-NetTCPConnection -LocalPort 11434 -State Listen -ErrorAction SilentlyContinue
    if ($listener) {
        if ($listener.LocalAddress -ne '127.0.0.1') { throw 'Ollama läuft auf einer anderen Netzwerkschnittstelle; diese Konfiguration bleibt unverändert.' }
        $serverInfo = Get-CimInstance Win32_Process -Filter ('ProcessId=' + $listener.OwningProcess)
        if ($serverInfo.ExecutablePath -ne $binary -or $serverInfo.CommandLine -notmatch '\bserve\b') { throw 'Der Listener ist kein eindeutig erkannter Ollama-Server; er wird nicht beendet.' }
        $serverProcess = Get-Process -Id $listener.OwningProcess
    }
}
[Environment]::SetEnvironmentVariable('OLLAMA_ORIGINS',$value,'User')
if ($Restart) {
    if ($serverProcess) { Stop-Process -Id $serverProcess.Id; $serverProcess.WaitForExit() }
    $process = Start-Process -FilePath $binary -ArgumentList 'serve' -WindowStyle Hidden -PassThru -Environment @{ OLLAMA_ORIGINS=$value; OLLAMA_HOST='127.0.0.1:11434' } -RedirectStandardOutput (Join-Path $evidence 'ollama-server.log') -RedirectStandardError (Join-Path $evidence 'ollama-server-error.log')
    Set-Content -LiteralPath (Join-Path $evidence 'ollama-server.pid') -Value $process.Id
    for ($attempt=0; $attempt -lt 30; $attempt++) {
        try { $response=Invoke-WebRequest -Uri 'http://127.0.0.1:11434/api/tags' -Headers @{ Origin=$GameOrigin } -TimeoutSec 2; if ($response.Headers['Access-Control-Allow-Origin'] -eq $GameOrigin) { Write-Host ('Ollama lokal bereit für ' + $GameOrigin); exit 0 } } catch { }
        Start-Sleep -Milliseconds 300
    }
    throw 'Ollama-Freigabe konnte nicht bestätigt werden; siehe private Ollama-Serverprotokolle.'
}
Write-Host 'Genaue Spiel-Websites wurden in OLLAMA_ORIGINS ergänzt. Ollama neu starten oder dieses Script mit -Restart nach Abschluss laufender Anfragen ausführen.'
