[CmdletBinding()]
param(
    [string]$SidecarBaseUrl = $(if ($env:NAKO_METADATA_SCRAPER_BASE_URL) { $env:NAKO_METADATA_SCRAPER_BASE_URL } else { 'http://127.0.0.1:9100' }),
    [string]$NakoBaseUrl = $(if ($env:NAKO_BASE_URL) { $env:NAKO_BASE_URL } else { 'http://127.0.0.1:3000' }),
    [string]$AdminToken = $env:NAKO_ADMIN_TOKEN,
    [switch]$RegisterInNako,
    [switch]$Enable,
    [switch]$RunResourceCall,
    [switch]$RunTaskPath,
    [switch]$RunWriteback,
    [switch]$IssueAddonToken,
    [switch]$RequireNako,
    [switch]$NoAdminAuth,
    [string]$MetadataWritebackLibraryId = $env:NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_LIBRARY_ID,
    [string]$MetadataWritebackTargetKind = $(if ($env:NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_TARGET_KIND) { $env:NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_TARGET_KIND } else { 'media_source' }),
    [string]$MetadataWritebackTargetId = $env:NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_TARGET_ID,
    [string]$MetadataWritebackIdempotencyKey = $(if ($env:NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_IDEMPOTENCY_KEY) { $env:NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_IDEMPOTENCY_KEY } else { "local-smoke-metadata-writeback-$([guid]::NewGuid())" }),
    [string]$ExpectedWritebackStatus = $env:NAKO_METADATA_SCRAPER_SMOKE_EXPECTED_WRITEBACK_STATUS,
    [string]$ExpectedWritebackSafeErrorCode = $env:NAKO_METADATA_SCRAPER_SMOKE_EXPECTED_WRITEBACK_SAFE_ERROR_CODE,
    [string]$TaskPathLibraryId = $env:NAKO_METADATA_SCRAPER_SMOKE_TASK_LIBRARY_ID,
    [string]$TaskPathSourceId = $env:NAKO_METADATA_SCRAPER_SMOKE_TASK_SOURCE_ID,
    [string]$TaskPathIdempotencyKey = $(if ($env:NAKO_METADATA_SCRAPER_SMOKE_TASK_IDEMPOTENCY_KEY) { $env:NAKO_METADATA_SCRAPER_SMOKE_TASK_IDEMPOTENCY_KEY } else { "local-smoke-bulk-task-$([guid]::NewGuid())" })
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

function Assert-SmokeOptions {
    $nakoFlags = @()
    if ($Enable) { $nakoFlags += '-Enable' }
    if ($RunResourceCall) { $nakoFlags += '-RunResourceCall' }
    if ($RunTaskPath) { $nakoFlags += '-RunTaskPath' }
    if ($IssueAddonToken) { $nakoFlags += '-IssueAddonToken' }
    if ($RequireNako) { $nakoFlags += '-RequireNako' }

    if ($nakoFlags.Count -gt 0 -and -not $RegisterInNako) {
        throw "$($nakoFlags -join ', ') require -RegisterInNako so the smoke cannot silently skip Nako Admin paths."
    }

    if (-not [string]::IsNullOrWhiteSpace($ExpectedWritebackStatus)) {
        $allowedStatuses = @('submitted', 'skipped', 'failed', 'any')
        if ($ExpectedWritebackStatus -notin $allowedStatuses) {
            throw "-ExpectedWritebackStatus must be one of: $($allowedStatuses -join ', ')."
        }
        if (-not $RunWriteback) {
            throw '-ExpectedWritebackStatus requires -RunWriteback.'
        }
    }

    if (-not [string]::IsNullOrWhiteSpace($ExpectedWritebackSafeErrorCode) -and -not $RunWriteback) {
        throw '-ExpectedWritebackSafeErrorCode requires -RunWriteback.'
    }
}

function Assert-ManifestTask {
    param(
        [Parameter(Mandatory = $true)][object]$Manifest,
        [Parameter(Mandatory = $true)][string]$TaskId,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $matches = @($Manifest.tasks | Where-Object { $_.id -eq $TaskId })
    Assert-MinCount -Items $matches -Minimum 1 -Name "manifest.tasks[$TaskId]"
    Assert-Equal -Actual $matches[0].path -Expected $Path -Name "manifest.tasks[$TaskId].path"
}

function Assert-ManifestEventSubscription {
    param(
        [Parameter(Mandatory = $true)][object]$Manifest,
        [Parameter(Mandatory = $true)][string]$SubscriptionId,
        [Parameter(Mandatory = $true)][string]$EventKind,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $matches = @($Manifest.event_subscriptions | Where-Object { $_.id -eq $SubscriptionId })
    Assert-MinCount -Items $matches -Minimum 1 -Name "manifest.event_subscriptions[$SubscriptionId]"
    Assert-Equal -Actual $matches[0].event_kind -Expected $EventKind -Name "manifest.event_subscriptions[$SubscriptionId].event_kind"
    Assert-Equal -Actual $matches[0].path -Expected $Path -Name "manifest.event_subscriptions[$SubscriptionId].path"
}

function Assert-WritebackExpectation {
    param(
        [object]$Writeback,
        [string]$ExpectedStatus,
        [string]$ExpectedSafeErrorCode
    )

    if (
        [string]::IsNullOrWhiteSpace($ExpectedStatus) -and
        [string]::IsNullOrWhiteSpace($ExpectedSafeErrorCode)
    ) {
        return
    }

    if ($null -eq $Writeback) {
        throw 'metadata response did not include a writeback summary.'
    }

    if (-not [string]::IsNullOrWhiteSpace($ExpectedStatus) -and $ExpectedStatus -ne 'any') {
        Assert-Equal -Actual ([string]$Writeback.status) -Expected $ExpectedStatus -Name 'metadata writeback status'
    }

    if (-not [string]::IsNullOrWhiteSpace($ExpectedSafeErrorCode)) {
        Assert-Equal -Actual ([string]$Writeback.safe_error_code) -Expected $ExpectedSafeErrorCode -Name 'metadata writeback safe_error_code'
    }
}

function New-MetadataWritebackPayload {
    param(
        [Parameter(Mandatory = $true)][switch]$Enabled,
        [string]$LibraryId,
        [string]$TargetKind,
        [string]$TargetId,
        [string]$IdempotencyKey
    )

    if (-not $Enabled) {
        return $null
    }

    if ([string]::IsNullOrWhiteSpace($LibraryId)) {
        throw '-RunWriteback requires -MetadataWritebackLibraryId or NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_LIBRARY_ID.'
    }
    if ([string]::IsNullOrWhiteSpace($TargetKind)) {
        throw '-RunWriteback requires -MetadataWritebackTargetKind or NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_TARGET_KIND.'
    }
    if ([string]::IsNullOrWhiteSpace($TargetId)) {
        throw '-RunWriteback requires -MetadataWritebackTargetId or NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_TARGET_ID.'
    }
    if ([string]::IsNullOrWhiteSpace($IdempotencyKey)) {
        throw '-RunWriteback requires -MetadataWritebackIdempotencyKey or NAKO_METADATA_SCRAPER_SMOKE_WRITEBACK_IDEMPOTENCY_KEY.'
    }

    return [ordered]@{
        library_id = $LibraryId
        target = [ordered]@{
            kind = $TargetKind
            id = $TargetId
        }
        idempotency_key = $IdempotencyKey
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

function Wait-NakoAddonTaskRun {
    param(
        [Parameter(Mandatory = $true)][string]$AddonId,
        [Parameter(Mandatory = $true)][string]$JobId,
        [Parameter(Mandatory = $true)][hashtable]$Headers,
        [int]$TimeoutSeconds = 90
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        $taskRun = Invoke-Json -Method 'GET' -Url (Join-HttpUrl $NakoBaseUrl "/admin/v1/addons/$AddonId/task-runs/$JobId") -Headers $Headers
        $status = [string]$taskRun.run.status
        if ($status -in @('succeeded', 'failed', 'cancelled')) {
            return $taskRun
        }

        if ((Get-Date) -ge $deadline) {
            throw "Addon task run $JobId did not reach a terminal status within $TimeoutSeconds seconds. Last status: '$status'."
        }

        Start-Sleep -Seconds 1
    } while ($true)
}

Assert-SmokeOptions

Write-Host "[sidecar] Fetching manifest from $SidecarBaseUrl"
$manifest = Invoke-Json -Method 'GET' -Url (Join-HttpUrl $SidecarBaseUrl '/manifest.json')
Assert-Equal -Actual $manifest.id -Expected 'nako.official.metadata-scraper' -Name 'manifest.id'
Assert-Equal -Actual $manifest.protocol_version -Expected '0.1.0-alpha.1' -Name 'manifest.protocol_version'
Assert-MinCount -Items @($manifest.resources) -Minimum 1 -Name 'manifest.resources'
Assert-ManifestTask -Manifest $manifest -TaskId 'bulk-metadata-scrape' -Path '/tasks/bulk-metadata-scrape'
Assert-ManifestEventSubscription -Manifest $manifest -SubscriptionId 'library-scanned' -EventKind 'library.scanned' -Path '/events/library-scanned'
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
$metadataWriteback = New-MetadataWritebackPayload `
    -Enabled:$RunWriteback `
    -LibraryId $MetadataWritebackLibraryId `
    -TargetKind $MetadataWritebackTargetKind `
    -TargetId $MetadataWritebackTargetId `
    -IdempotencyKey $MetadataWritebackIdempotencyKey
if ($null -ne $metadataWriteback) {
    $metadataRequest.payload['writeback'] = $metadataWriteback
}
$metadata = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/metadata') -Body $metadataRequest
Assert-Equal -Actual $metadata.addon_id -Expected $manifest.id -Name 'metadata.addon_id'
Assert-Equal -Actual $metadata.resource -Expected 'metadata' -Name 'metadata.resource'
Assert-MinCount -Items @($metadata.payload.candidates) -Minimum 1 -Name 'metadata.payload.candidates'
Assert-MinCount -Items @($metadata.artifacts) -Minimum 1 -Name 'metadata.artifacts'
Write-Host "[sidecar] Metadata resource OK; candidates=$(@($metadata.payload.candidates).Count), artifacts=$(@($metadata.artifacts).Count)"
if ($RunWriteback) {
    if ($null -eq $metadata.payload.writeback) {
        throw 'metadata response did not include a writeback summary.'
    }

    Assert-WritebackExpectation `
        -Writeback $metadata.payload.writeback `
        -ExpectedStatus $ExpectedWritebackStatus `
        -ExpectedSafeErrorCode $ExpectedWritebackSafeErrorCode
    Write-Host "[sidecar] Writeback status: $($metadata.payload.writeback.status)"
}

$eventRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    subscription_id = 'library-scanned'
    event_id = "local-smoke-event-$([guid]::NewGuid())"
    event_kind = 'library.scanned'
    subject_kind = 'library'
    subject_id = 'local-smoke-library'
    occurred_at = '2026-05-25T00:00:00.000Z'
    attempt = 1
    payload = [ordered]@{
        library_id = 'local-smoke-library'
        secret = 'nako_at_should_not_echo'
    }
}
$event = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/events/library-scanned') -Body $eventRequest
Assert-Equal -Actual $event.addon_id -Expected $manifest.id -Name 'event.addon_id'
Assert-Equal -Actual $event.subscription_id -Expected 'library-scanned' -Name 'event.subscription_id'
Assert-Equal -Actual $event.event_id -Expected $eventRequest.event_id -Name 'event.event_id'
Assert-Equal -Actual $event.output.schema -Expected 'nako.official.metadata-scraper.library-scanned.event.v1' -Name 'event.output.schema'
Assert-Equal -Actual $event.output.accepted -Expected $true -Name 'event.output.accepted'
$eventJson = $event | ConvertTo-Json -Depth 64 -Compress
if ($eventJson.Contains('nako_at_should_not_echo')) {
    throw 'event response echoed a secret payload value.'
}
Write-Host "[sidecar] Event subscription OK; subscription=$($event.subscription_id)"

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
$requiredScopes = @('item_metadata_read', 'item_metadata_suggest', 'webhook_event_read')
if ($RunTaskPath) {
    $requiredScopes += 'automation_run'
}

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
    $addonStatus = $enabled.addon.summary.status
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

if ($RunTaskPath) {
    if ($addonStatus -ne 'enabled') {
        throw '-RunTaskPath requires an enabled addon. Pass -Enable or reuse an already-enabled registration.'
    }

    $routing = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $NakoBaseUrl "/admin/v1/addons/$addonId/routing-plans") -Headers $adminHeaders
    Write-Host "[nako] Routing plans synced; plans=$(@($routing.plans).Count)"

    $taskPayloadItem = [ordered]@{
        title = 'The Matrix'
        year = 1999
        language = 'en-US'
    }
    $taskRunRequest = [ordered]@{
        declaration_id = 'bulk-metadata-scrape'
        idempotency_key = $TaskPathIdempotencyKey
        dispatch = 'direct'
        payload = [ordered]@{
            batch_size = 1
            items = @($taskPayloadItem)
        }
    }
    if (-not [string]::IsNullOrWhiteSpace($TaskPathLibraryId)) {
        $taskRunRequest['library_id'] = $TaskPathLibraryId
    }
    if (-not [string]::IsNullOrWhiteSpace($TaskPathSourceId)) {
        $taskRunRequest['source_id'] = $TaskPathSourceId
    }

    $createdTaskRun = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $NakoBaseUrl "/admin/v1/addons/$addonId/task-runs") -Body $taskRunRequest -Headers $adminHeaders
    Assert-Equal -Actual $createdTaskRun.run.declaration_id -Expected 'bulk-metadata-scrape' -Name 'Addon Task declaration_id'
    Assert-Equal -Actual $createdTaskRun.run.has_input -Expected $true -Name 'Addon Task has_input'
    Write-Host "[nako] Created direct Addon Task run $($createdTaskRun.run.job_id); status=$($createdTaskRun.run.status)"

    $completedTaskRun = Wait-NakoAddonTaskRun -AddonId $addonId -JobId $createdTaskRun.run.job_id -Headers $adminHeaders
    Assert-Equal -Actual $completedTaskRun.run.status -Expected 'succeeded' -Name 'Addon Task run status'
    Assert-Equal -Actual $completedTaskRun.run.result.status -Expected 'succeeded' -Name 'Addon Task result status'
    Assert-Equal -Actual $completedTaskRun.run.result.output.schema -Expected 'nako.official.metadata-scraper.bulk-metadata-scrape.result.v1' -Name 'Addon Task output schema'
    Assert-Equal -Actual $completedTaskRun.run.result.output.processed_items -Expected 1 -Name 'Addon Task processed_items'
    $taskItems = @($completedTaskRun.run.result.output.items | Where-Object { $null -ne $_ })
    Assert-MinCount -Items $taskItems -Minimum 1 -Name 'Addon Task output.items'
    $taskCandidates = @($taskItems[0].payload.candidates | Where-Object { $null -ne $_ })
    Assert-MinCount -Items $taskCandidates -Minimum 1 -Name 'Addon Task first item candidates'
    Write-Host "[nako] Direct Addon Task path OK; job_id=$($completedTaskRun.run.job_id), candidates=$($taskCandidates.Count)"
}

Write-Host '[ok] Local metadata scraper smoke completed.'
