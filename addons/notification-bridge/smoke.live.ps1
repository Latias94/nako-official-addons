[CmdletBinding()]
param(
    [string]$SidecarBaseUrl = $(if ($env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE_SIDECAR_BASE_URL) { $env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE_SIDECAR_BASE_URL } else { 'http://127.0.0.1:9110' }),
    [string]$ExpectedProviderId = $(if ($env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE_EXPECTED_PROVIDER_ID) { $env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE_EXPECTED_PROVIDER_ID } else { 'http_webhook' })
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 3.0

if ($env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE -notin @('1', 'true', 'TRUE', 'yes', 'YES')) {
    Write-Host '[skip] Live notification provider smoke is disabled. Set NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE=1 to run it locally.'
    exit 0
}

function Join-HttpUrl {
    param(
        [Parameter(Mandatory = $true)][string]$BaseUrl,
        [Parameter(Mandatory = $true)][string]$Path
    )

    return ($BaseUrl.TrimEnd('/') + '/' + $Path.TrimStart('/'))
}

function Invoke-Json {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Url,
        [object]$Body = $null
    )

    $request = @{
        Method = $Method
        Uri = $Url
        TimeoutSec = 30
    }

    if ($null -ne $Body) {
        $request['ContentType'] = 'application/json'
        $request['Body'] = ($Body | ConvertTo-Json -Depth 64 -Compress)
    }

    return Invoke-RestMethod @request
}

function Assert-Equal {
    param(
        [Parameter(Mandatory = $true)][object]$Actual,
        [Parameter(Mandatory = $true)][object]$Expected,
        [Parameter(Mandatory = $true)][string]$Name
    )

    if ($Actual -ne $Expected) {
        throw "$Name expected '$Expected' but got '$Actual'."
    }
}

function Assert-Text-Does-Not-Contain {
    param(
        [Parameter(Mandatory = $true)][string]$Text,
        [Parameter(Mandatory = $true)][string]$Needle,
        [Parameter(Mandatory = $true)][string]$Name
    )

    if ($Text.Contains($Needle)) {
        throw "$Name unexpectedly contained redacted sentinel."
    }
}

Write-Host "[sidecar] Running live provider smoke for provider '$ExpectedProviderId'."

$manifest = Invoke-Json -Method 'GET' -Url (Join-HttpUrl $SidecarBaseUrl '/manifest.json')
Assert-Equal -Actual $manifest.id -Expected 'nako.official.notification-bridge' -Name 'manifest.id'

$eventRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    subscription_id = 'library-scanned-notification'
    event_id = "notification-live-smoke-$([guid]::NewGuid())"
    event_kind = 'library.scanned'
    subject_kind = 'library'
    subject_id = 'library-live-smoke'
    occurred_at = '2026-05-25T00:00:00.000Z'
    attempt = 1
    payload = [ordered]@{
        library_id = 'library-live-smoke'
        secret = 'nako_at_live_smoke_should_not_echo'
    }
}

$event = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/events/library-scanned') -Body $eventRequest
$eventText = ($event | ConvertTo-Json -Depth 64 -Compress)

Assert-Equal -Actual $event.output.mode -Expected 'provider_send' -Name 'event.output.mode'
Assert-Equal -Actual $event.output.provider.id -Expected $ExpectedProviderId -Name 'event.output.provider.id'
Assert-Equal -Actual $event.output.provider.status -Expected 'sent' -Name 'event.output.provider.status'
Assert-Equal -Actual $event.output.provider.send_path_enabled -Expected $true -Name 'event.output.provider.send_path_enabled'
Assert-Text-Does-Not-Contain -Text $eventText -Needle 'nako_at_live_smoke_should_not_echo' -Name 'event output'

Write-Host '[done] Live notification provider smoke passed.'
