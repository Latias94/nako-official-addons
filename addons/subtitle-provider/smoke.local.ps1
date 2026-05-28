param(
    [string]$SidecarBaseUrl = "http://127.0.0.1:9140"
)

$ErrorActionPreference = "Stop"

$manifest = Invoke-RestMethod -Method Get -Uri "$SidecarBaseUrl/manifest.json"
if ($manifest.id -ne "nako.official.subtitle-provider") {
    throw "unexpected manifest id: $($manifest.id)"
}

$healthBody = @{
    protocol_version = $manifest.protocol_version
    manifest_id = $manifest.id
    request_id = "subtitle-health-smoke"
    expected_addon_version = $manifest.version
    expected_resource_count = $manifest.resources.Count
} | ConvertTo-Json -Depth 8
$health = Invoke-RestMethod -Method Post -Uri "$SidecarBaseUrl/health" -ContentType "application/json" -Body $healthBody
if ($health.manifest_id -ne $manifest.id) {
    throw "unexpected health manifest id: $($health.manifest_id)"
}

$resourceBody = @{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    resource = "subtitle"
    request_id = "subtitle-smoke"
    payload = @{
        schema = "nako.official.subtitle_provider.request.v1"
        query = "Demo Movie"
        languages = @("en")
        limit = 1
    }
} | ConvertTo-Json -Depth 16
$resource = Invoke-RestMethod -Method Post -Uri "$SidecarBaseUrl/subtitle" -ContentType "application/json" -Body $resourceBody
if ($resource.payload.total -lt 1) {
    throw "expected at least one subtitle candidate"
}

Write-Host "subtitle-provider smoke passed"
