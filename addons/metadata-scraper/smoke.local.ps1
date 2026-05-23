[CmdletBinding()]
param(
    [string]$SidecarBaseUrl = $(if ($env:NAKO_METADATA_SCRAPER_BASE_URL) { $env:NAKO_METADATA_SCRAPER_BASE_URL } else { 'http://127.0.0.1:9100' }),
    [string]$NakoBaseUrl = $(if ($env:NAKO_BASE_URL) { $env:NAKO_BASE_URL } else { 'http://127.0.0.1:3000' }),
    [string]$AdminToken = $env:NAKO_ADMIN_TOKEN,
    [switch]$RegisterInNako,
    [switch]$Enable,
    [switch]$RunResourceCall,
    [switch]$IssueAddonToken,
    [switch]$RequireNako,
    [switch]$NoAdminAuth
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
        [object]$Body = $null,
        [hashtable]$Headers = @{}
    )

    $request = @{
        Method = $Method
        Uri = $Url
        Headers = $Headers
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

function Assert-MinCount {
    param(
        [Parameter(Mandatory = $true)][object[]]$Items,
        [Parameter(Mandatory = $true)][int]$Minimum,
        [Parameter(Mandatory = $true)][string]$Name
    )

    if ($Items.Count -lt $Minimum) {
        throw "$Name expected at least $Minimum item(s) but got $($Items.Count)."
    }
}

function New-AdminHeaders {
    if ($NoAdminAuth) {
        return @{ Accept = 'application/json' }
    }

    if ([string]::IsNullOrWhiteSpace($AdminToken)) {
        return $null
    }

    return @{
        Accept = 'application/json'
        Authorization = "Bearer $AdminToken"
    }
}

Write-Host "[sidecar] Fetching manifest from $SidecarBaseUrl"
$manifest = Invoke-Json -Method 'GET' -Url (Join-HttpUrl $SidecarBaseUrl '/manifest.json')
Assert-Equal -Actual $manifest.id -Expected 'nako.official.metadata-scraper' -Name 'manifest.id'
Assert-Equal -Actual $manifest.protocol_version -Expected '0.1.0-alpha.1' -Name 'manifest.protocol_version'
Assert-MinCount -Items @($manifest.resources) -Minimum 1 -Name 'manifest.resources'
Write-Host "[sidecar] Manifest OK: $($manifest.id)@$($manifest.version)"

$healthRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    manifest_id = $manifest.id
    request_id = "local-smoke-health-$([guid]::NewGuid())"
    expected_addon_version = $manifest.version
    expected_resource_count = @($manifest.resources).Count
}
$health = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/health') -Body $healthRequest
Assert-Equal -Actual $health.status -Expected 'ok' -Name 'sidecar health status'
$enabledProviders = @($health.diagnostics.enabled_providers) -join ', '
Write-Host "[sidecar] Health OK; enabled providers: $enabledProviders"

$metadataRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    resource = 'metadata'
    request_id = "local-smoke-metadata-$([guid]::NewGuid())"
    payload = [ordered]@{
        title = 'The Matrix'
        year = 1999
        language = 'en-US'
    }
}
$metadata = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/metadata') -Body $metadataRequest
Assert-Equal -Actual $metadata.addon_id -Expected $manifest.id -Name 'metadata.addon_id'
Assert-Equal -Actual $metadata.resource -Expected 'metadata' -Name 'metadata.resource'
Assert-MinCount -Items @($metadata.payload.candidates) -Minimum 1 -Name 'metadata.payload.candidates'
Assert-MinCount -Items @($metadata.artifacts) -Minimum 1 -Name 'metadata.artifacts'
Write-Host "[sidecar] Metadata resource OK; candidates=$(@($metadata.payload.candidates).Count), artifacts=$(@($metadata.artifacts).Count)"

if (-not $RegisterInNako) {
    if ($RequireNako) {
        throw '-RequireNako requires -RegisterInNako.'
    }

    Write-Host '[skip] Nako Admin smoke skipped. Pass -RegisterInNako to register or reuse this manifest through Nako Admin API.'
    exit 0
}

$adminHeaders = New-AdminHeaders
if ($null -eq $adminHeaders) {
    $message = 'Nako Admin smoke skipped because NAKO_ADMIN_TOKEN is not set. Pass -NoAdminAuth only for an unauthenticated local dev server.'
    if ($RequireNako) {
        throw $message
    }

    Write-Host "[skip] $message"
    exit 0
}

if ($manifest.base_url.TrimEnd('/') -ne $SidecarBaseUrl.TrimEnd('/')) {
    Write-Warning "Manifest base_url is '$($manifest.base_url)'. Nako will call that URL, not -SidecarBaseUrl."
}

$addons = Invoke-Json -Method 'GET' -Url (Join-HttpUrl $NakoBaseUrl '/admin/v1/addons') -Headers $adminHeaders
$existingMatches = @($addons.addons | Where-Object { $_.manifest_id -eq $manifest.id -and $_.status -ne 'unregistered' })
$requiredScopes = @('item_metadata_read', 'item_metadata_suggest')

if ($existingMatches.Count -gt 0) {
    $addonId = $existingMatches[0].id
    $addonStatus = $existingMatches[0].status
    Write-Host "[nako] Reusing registered addon $addonId with status '$addonStatus'"
    $refreshRegistrationRequest = [ordered]@{
        id = $addonId
        manifest = $manifest
        granted_scopes = $requiredScopes
        status = $addonStatus
    }
    $refreshed = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $NakoBaseUrl '/admin/v1/addons') -Body $refreshRegistrationRequest -Headers $adminHeaders
    $addonStatus = $refreshed.addon.summary.status
    Write-Host "[nako] Refreshed manifest-granted scopes for addon $addonId"
} else {
    $registrationRequest = [ordered]@{
        id = $null
        manifest = $manifest
        granted_scopes = $requiredScopes
        status = 'disabled'
    }
    $registered = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $NakoBaseUrl '/admin/v1/addons') -Body $registrationRequest -Headers $adminHeaders
    $addonId = $registered.addon.summary.id
    $addonStatus = $registered.addon.summary.status
    Write-Host "[nako] Registered addon $addonId with status '$addonStatus'"
}

$healthCheck = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $NakoBaseUrl "/admin/v1/addons/$addonId/health-check") -Headers $adminHeaders
Assert-Equal -Actual $healthCheck.status -Expected 'reachable' -Name 'Nako Addon Health Check status'
Write-Host "[nako] Health Check OK; latency_ms=$($healthCheck.latency_ms)"

if ($IssueAddonToken) {
    $issued = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $NakoBaseUrl "/admin/v1/addons/$addonId/tokens") -Body @{ label = 'metadata scraper local smoke' } -Headers $adminHeaders
    if ([string]::IsNullOrWhiteSpace($issued.raw_token)) {
        throw 'Nako token issue response did not include the one-time raw token.'
    }

    Write-Host "[nako] Issued Addon Token $($issued.token.id) with prefix $($issued.token.token_prefix). Raw token intentionally not printed."
}

if ($Enable) {
    $enabled = Invoke-Json -Method 'PATCH' -Url (Join-HttpUrl $NakoBaseUrl "/admin/v1/addons/$addonId/status") -Body @{ status = 'enabled' } -Headers $adminHeaders
    Assert-Equal -Actual $enabled.addon.summary.status -Expected 'enabled' -Name 'Nako addon status'
    Write-Host "[nako] Addon enabled"
}

if ($RunResourceCall) {
    $diagnosticRequest = [ordered]@{
        resource = 'metadata'
        payload = [ordered]@{
            title = 'The Matrix'
            year = 1999
            language = 'en-US'
        }
    }
    $diagnostic = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $NakoBaseUrl "/admin/v1/addons/$addonId/diagnostics/resource-call") -Body $diagnosticRequest -Headers $adminHeaders
    if ($diagnostic.status -ne 'succeeded') {
        throw "Nako metadata resource diagnostic failed with status '$($diagnostic.status)' and safe_error_code '$($diagnostic.safe_error_code)'."
    }

    Write-Host "[nako] Resource diagnostic OK; attempts=$($diagnostic.attempts), http_status=$($diagnostic.http_status)"
}

Write-Host '[ok] Local metadata scraper smoke completed.'
