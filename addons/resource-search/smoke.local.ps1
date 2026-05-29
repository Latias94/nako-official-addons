[CmdletBinding()]
param(
    [string]$SidecarBaseUrl = $(if ($env:NAKO_RESOURCE_SEARCH_BASE_URL) { $env:NAKO_RESOURCE_SEARCH_BASE_URL } else { 'http://127.0.0.1:9130' }),
    [string]$Query = $(if ($env:NAKO_RESOURCE_SEARCH_SMOKE_QUERY) { $env:NAKO_RESOURCE_SEARCH_SMOKE_QUERY } else { 'Demo Movie' })
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

Write-Host "[sidecar] Fetching manifest from $SidecarBaseUrl"
$manifest = Invoke-Json -Method 'GET' -Url (Join-HttpUrl $SidecarBaseUrl '/manifest.json')
Assert-Equal -Actual $manifest.id -Expected 'nako.official.resource-search' -Name 'manifest.id'
Assert-Equal -Actual $manifest.protocol_version -Expected '0.1.0-alpha.1' -Name 'manifest.protocol_version'
Assert-Equal -Actual $manifest.resources[0].kind -Expected 'resource_search' -Name 'manifest.resources[0].kind'
Assert-Equal -Actual $manifest.resources[0].path -Expected '/resource-search' -Name 'manifest.resources[0].path'
Assert-Equal -Actual $manifest.resources[0].input_schema -Expected 'nako.addon.resource_search.request.v1' -Name 'manifest.resources[0].input_schema'
Assert-Equal -Actual $manifest.resources[0].output_schema -Expected 'nako.addon.resource_search.response.v1' -Name 'manifest.resources[0].output_schema'
Assert-Equal -Actual $manifest.resources[1].kind -Expected 'resource_link_check' -Name 'manifest.resources[1].kind'
Assert-Equal -Actual $manifest.resources[1].path -Expected '/resource-link-check' -Name 'manifest.resources[1].path'
Assert-Equal -Actual $manifest.resources[1].input_schema -Expected 'nako.addon.resource_link_check.request.v1' -Name 'manifest.resources[1].input_schema'
Assert-Equal -Actual $manifest.resources[1].output_schema -Expected 'nako.addon.resource_link_check.response.v1' -Name 'manifest.resources[1].output_schema'
Assert-Equal -Actual $manifest.scopes[0] -Expected 'acquisition_search_read' -Name 'manifest.scopes[0]'
Assert-Equal -Actual $manifest.scopes[1] -Expected 'acquisition_link_check_read' -Name 'manifest.scopes[1]'
Write-Host "[sidecar] Manifest OK: $($manifest.id)@$($manifest.version)"

$healthRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    manifest_id = $manifest.id
    request_id = "local-smoke-health-$([guid]::NewGuid())"
    expected_addon_version = $manifest.version
    expected_resource_count = @($manifest.resources).Count
}
$health = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/health') -Body $healthRequest
Write-Host "[sidecar] Health status: $($health.status); providers=$($health.diagnostics.runtime_provider_count)"

$searchRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    resource = 'resource_search'
    request_id = "local-smoke-search-$([guid]::NewGuid())"
    payload = [ordered]@{
        schema = 'nako.addon.resource_search.request.v1'
        intent = [ordered]@{
            kind = 'free_text'
            text = $Query
        }
        query = $Query
        limit = 5
    }
}
$search = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/resource-search') -Body $searchRequest
Assert-Equal -Actual $search.resource -Expected 'resource_search' -Name 'search.resource'
Assert-Equal -Actual $search.payload.schema -Expected 'nako.addon.resource_search.response.v1' -Name 'search.payload.schema'
Assert-MinCount -Items @($search.payload.results) -Minimum 1 -Name 'search.payload.results'
Write-Host "[sidecar] Search OK: total=$($search.payload.total); query='$($search.payload.query)'"

Assert-MinCount -Items @($search.payload.results[0].links) -Minimum 1 -Name 'search.payload.results[0].links'
$linkCheckRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    resource = 'resource_link_check'
    request_id = "local-smoke-link-check-$([guid]::NewGuid())"
    payload = [ordered]@{
        schema = 'nako.addon.resource_link_check.request.v1'
        link = $search.payload.results[0].links[0]
        refresh = $false
        context = [ordered]@{
            search_id = 'local-smoke'
            selection_id = 'local-smoke-selection'
        }
    }
}
$linkCheck = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/resource-link-check') -Body $linkCheckRequest
Assert-Equal -Actual $linkCheck.resource -Expected 'resource_link_check' -Name 'linkCheck.resource'
Assert-Equal -Actual $linkCheck.payload.schema -Expected 'nako.addon.resource_link_check.response.v1' -Name 'linkCheck.payload.schema'
Assert-Equal -Actual $linkCheck.payload.link_type -Expected $search.payload.results[0].links[0].link_type -Name 'linkCheck.payload.link_type'
Write-Host "[sidecar] Link check OK: status=$($linkCheck.payload.status); type=$($linkCheck.payload.link_type)"

Write-Host '[ok] Local resource search smoke completed.'
