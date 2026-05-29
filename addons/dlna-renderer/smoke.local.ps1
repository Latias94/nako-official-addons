param(
    [string]$SidecarBaseUrl = "http://127.0.0.1:9150"
)

$ErrorActionPreference = "Stop"

$manifest = Invoke-RestMethod -Method Get -Uri "$SidecarBaseUrl/manifest.json"
if ($manifest.id -ne "nako.official.dlna-renderer") {
    throw "unexpected manifest id: $($manifest.id)"
}

$healthBody = @{
    protocol_version = $manifest.protocol_version
    manifest_id = $manifest.id
    request_id = "dlna-health-smoke"
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
    resource = "renderer_adapter"
    request_id = "dlna-readiness-smoke"
    payload = @{
        action = "inspect_readiness"
        protocol = "dlna_renderer"
    }
} | ConvertTo-Json -Depth 16
$resource = Invoke-RestMethod -Method Post -Uri "$SidecarBaseUrl/renderer-adapter" -ContentType "application/json" -Body $resourceBody
if ($resource.payload.kind -ne "readiness") {
    throw "expected readiness payload"
}

Write-Host "dlna-renderer smoke passed"
