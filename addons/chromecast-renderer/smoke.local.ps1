[CmdletBinding()]
param(
    [string]$SidecarBaseUrl = $(if ($env:NAKO_CHROMECAST_RENDERER_BASE_URL) { $env:NAKO_CHROMECAST_RENDERER_BASE_URL } else { 'http://127.0.0.1:9120' }),
    [switch]$RunDiscovery
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
Assert-Equal -Actual $manifest.id -Expected 'nako.official.chromecast-renderer' -Name 'manifest.id'
Assert-Equal -Actual $manifest.protocol_version -Expected '0.1.0-alpha.1' -Name 'manifest.protocol_version'
Assert-Equal -Actual $manifest.resources[0].kind -Expected 'renderer_adapter' -Name 'manifest.resources[0].kind'
Write-Host "[sidecar] Manifest OK: $($manifest.id)@$($manifest.version)"

$healthRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    manifest_id = $manifest.id
    request_id = "local-smoke-health-$([guid]::NewGuid())"
    expected_addon_version = $manifest.version
    expected_resource_count = @($manifest.resources).Count
}
$health = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/health') -Body $healthRequest
Write-Host "[sidecar] Health status: $($health.status); readiness=$($health.diagnostics.readiness.status)"

$readinessRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    resource = 'renderer_adapter'
    request_id = "local-smoke-readiness-$([guid]::NewGuid())"
    payload = [ordered]@{
        action = 'inspect_readiness'
        protocol = 'chromecast'
    }
}
$readiness = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/renderer-adapter') -Body $readinessRequest
Assert-Equal -Actual $readiness.resource -Expected 'renderer_adapter' -Name 'readiness.resource'
Assert-Equal -Actual $readiness.payload.kind -Expected 'readiness' -Name 'readiness.payload.kind'
Write-Host "[sidecar] Readiness OK: $($readiness.payload.readiness.status) ($($readiness.payload.readiness.reason_code))"

if ($RunDiscovery) {
    $discoveryRequest = [ordered]@{
        protocol_version = $manifest.protocol_version
        addon_id = $manifest.id
        resource = 'renderer_adapter'
        request_id = "local-smoke-discovery-$([guid]::NewGuid())"
        payload = [ordered]@{
            action = 'discover_targets'
            protocol = 'chromecast'
            timeout_ms = 1000
        }
    }
    $targets = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/renderer-adapter') -Body $discoveryRequest
    Assert-Equal -Actual $targets.payload.kind -Expected 'targets' -Name 'targets.payload.kind'
    Write-Host "[sidecar] Discovery OK; targets=$(@($targets.payload.targets).Count)"
}

Write-Host '[ok] Local Chromecast renderer smoke completed.'
