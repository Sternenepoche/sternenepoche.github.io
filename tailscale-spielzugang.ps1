param(
    [ValidateSet('Enable','Disable','Status')][string]$Action = 'Status',
    [ValidateSet(443,8443,10000)][int]$HttpsPort = 443,
    [ValidateRange(0,20)][int]$MaxPlayers = 5,
    [string]$DataRoot = (Join-Path $PSScriptRoot 'data\online')
)
$ErrorActionPreference = 'Stop'
$tailscale = (Get-Command tailscale.exe -ErrorAction Stop).Source
if ($Action -eq 'Status') { & $tailscale funnel status; exit $LASTEXITCODE }
$device = (& $tailscale status --json | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0 -or $device.BackendState -ne 'Running' -or !$device.Self.Online) { throw 'Tailscale muss auf diesem PC angemeldet und online sein.' }
$dnsName = $device.Self.DNSName.TrimEnd('.')
if ($dnsName -notmatch '^[a-z0-9-]+\.[a-z0-9-]+\.ts\.net$') { throw 'Keine gültige Tailscale-HTTPS-Adresse vorhanden.' }
$authority = $dnsName + ':' + $HttpsPort
$url = 'https://' + $dnsName + $(if ($HttpsPort -ne 443) { ':' + $HttpsPort } else { '' })
$target = 'http://127.0.0.1:8890'
$config = (& $tailscale serve status --json | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0) { throw 'Tailscale-Konfiguration konnte nicht gelesen werden.' }
$web = if ($config.Web) { $config.Web.PSObject.Properties[$authority].Value } else { $null }
$tcp = if ($config.TCP) { $config.TCP.PSObject.Properties[[string]$HttpsPort].Value } else { $null }
if ($tcp -or $web) {
    $handlers = @($web.Handlers.PSObject.Properties)
    if (!$tcp.HTTPS -or $handlers.Count -ne 1 -or $handlers[0].Name -ne '/' -or $handlers[0].Value.Proxy -ne $target) {
        throw 'Auf diesem HTTPS-Port läuft eine andere Freigabe. Sie bleibt unverändert; einen anderen Funnel-Port wählen.'
    }
}
if ($Action -eq 'Disable') {
    if (!$web) { Write-Host 'Keine Sternenepoche-Freigabe auf diesem Port.'; exit 0 }
    & $tailscale funnel "--https=$HttpsPort" off
    exit $LASTEXITCODE
}
$lobby = Invoke-RestMethod -Uri ($target + '/api/lobby') -TimeoutSec 10
if ($lobby.api_version -ne 1 -or $lobby.bots -ne 30) { throw 'Der erwartete lokale Rust-Spielserver läuft nicht.' }
if ($lobby.freigegebene_plaetze -gt $MaxPlayers) { throw "Die Freigabe liegt über $MaxPlayers. Zuerst im lokalen Dashboard begrenzen oder bewusst -MaxPlayers angeben." }
$dataPath = [IO.Path]::GetFullPath($DataRoot)
if (!(Test-Path -LiteralPath (Join-Path $dataPath 'spiel.sqlite3'))) { throw 'Datenverzeichnis des laufenden Servers fehlt.' }
[IO.File]::WriteAllText((Join-Path $dataPath 'public-url.txt'), $url + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
try {
    $cors = Invoke-WebRequest -Uri ($target + '/api/lobby') -Method Options -Headers @{ Origin=$url; 'Access-Control-Request-Method'='POST' } -TimeoutSec 10
    if ($cors.StatusCode -ne 204 -or $cors.Headers['Access-Control-Allow-Origin'] -ne $url) { throw 'Origin fehlt' }
} catch {
    throw 'Die HTTPS-Adresse wurde in public-url.txt gespeichert. Den Spielserver mit start-server.ps1 neu starten, damit er diese Website zulässt, dann Enable erneut aufrufen.'
}
# Only the public listener. Never route the dashboard, files or directories.
& $tailscale funnel --bg "--https=$HttpsPort" --yes $target
if ($LASTEXITCODE -ne 0) { throw 'Funnel konnte nicht aktiviert werden. Die Tailscale-Meldung enthält den notwendigen Einrichtungsschritt.' }
Write-Host ('Spielzugang: ' + $url)
Write-Host 'Dashboard bleibt ausschließlich auf http://127.0.0.1:8891.'
