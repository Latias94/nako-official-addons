[CmdletBinding()]
param(
    [string]$SidecarBaseUrl = $(if ($env:NAKO_NOTIFICATION_BRIDGE_BASE_URL) { $env:NAKO_NOTIFICATION_BRIDGE_BASE_URL } else { 'http://127.0.0.1:9110' })
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 3.0

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
        TimeoutSec = 15
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

Write-Host "[sidecar] Fetching manifest from $SidecarBaseUrl"
$manifest = Invoke-Json -Method 'GET' -Url (Join-HttpUrl $SidecarBaseUrl '/manifest.json')
Assert-Equal -Actual $manifest.id -Expected 'nako.official.notification-bridge' -Name 'manifest.id'
Assert-Equal -Actual $manifest.protocol_version -Expected '0.1.0-alpha.1' -Name 'manifest.protocol_version'
Assert-Equal -Actual $manifest.event_subscriptions[0].id -Expected 'library-scanned-notification' -Name 'manifest.event_subscriptions[0].id'
Assert-Equal -Actual $manifest.event_subscriptions[0].event_kind -Expected 'library.scanned' -Name 'manifest.event_subscriptions[0].event_kind'
Assert-Equal -Actual $manifest.event_subscriptions[0].path -Expected '/events/library-scanned' -Name 'manifest.event_subscriptions[0].path'
Write-Host "[sidecar] Manifest OK: $($manifest.id)@$($manifest.version)"

$healthRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    manifest_id = $manifest.id
    request_id = "notification-health-$([guid]::NewGuid())"
    expected_addon_version = $manifest.version
    expected_resource_count = $manifest.resources.Count
}
$health = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/health') -Body $healthRequest
Assert-Equal -Actual $health.status -Expected 'ok' -Name 'health.status'
Assert-Equal -Actual $health.manifest.resource_count -Expected $manifest.resources.Count -Name 'health.manifest.resource_count'
Assert-Equal -Actual $health.diagnostics.mode -Expected 'ack_only' -Name 'health.diagnostics.mode'
Assert-Equal -Actual $health.diagnostics.provider_fan_out -Expected $false -Name 'health.diagnostics.provider_fan_out'
Assert-Equal -Actual $health.diagnostics.providers[0].id -Expected 'http_webhook' -Name 'health.diagnostics.providers[0].id'
Assert-Equal -Actual $health.diagnostics.providers[0].status -Expected 'disabled' -Name 'health.diagnostics.providers[0].status'
Assert-Equal -Actual $health.diagnostics.providers[0].send_path_enabled -Expected $false -Name 'health.diagnostics.providers[0].send_path_enabled'
Write-Host '[sidecar] Health OK'

$eventRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    subscription_id = 'library-scanned-notification'
    event_id = "notification-event-$([guid]::NewGuid())"
    event_kind = 'library.scanned'
    subject_kind = 'library'
    subject_id = 'library-smoke'
    occurred_at = '2026-05-25T00:00:00.000Z'
    attempt = 1
    payload = [ordered]@{
        library_id = 'library-smoke'
        secret = 'nako_at_should_not_echo'
    }
}
$event = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/events/library-scanned') -Body $eventRequest
Assert-Equal -Actual $event.addon_id -Expected $manifest.id -Name 'event.addon_id'
Assert-Equal -Actual $event.subscription_id -Expected 'library-scanned-notification' -Name 'event.subscription_id'
Assert-Equal -Actual $event.output.schema -Expected 'nako.official.notification-bridge.library-scanned.event.v1' -Name 'event.output.schema'
Assert-Equal -Actual $event.output.mode -Expected 'ack_only' -Name 'event.output.mode'
Assert-Equal -Actual $event.output.provider.id -Expected 'http_webhook' -Name 'event.output.provider.id'
Assert-Equal -Actual $event.output.provider.status -Expected 'disabled' -Name 'event.output.provider.status'
Assert-Equal -Actual $event.output.provider.send_path_enabled -Expected $false -Name 'event.output.provider.send_path_enabled'
Write-Host '[sidecar] Event ACK OK'
Write-Host '[done] Nako Notification Bridge sidecar smoke passed.'
