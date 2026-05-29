[CmdletBinding()]
param(
    [string]$SidecarBaseUrl = $(if ($env:NAKO_EXTERNAL_ACQUISITION_RUNNER_BASE_URL) { $env:NAKO_EXTERNAL_ACQUISITION_RUNNER_BASE_URL } else { 'http://127.0.0.1:9160' })
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
Assert-Equal -Actual $manifest.id -Expected 'nako.official.external-acquisition-runner' -Name 'manifest.id'
Assert-Equal -Actual $manifest.protocol_version -Expected '0.1.0-alpha.1' -Name 'manifest.protocol_version'
Assert-Equal -Actual $manifest.tasks[0].id -Expected 'external-acquisition-action' -Name 'manifest.tasks[0].id'
Assert-Equal -Actual $manifest.tasks[0].input_schema -Expected 'nako.addon.external_acquisition_action.request.v1' -Name 'manifest.tasks[0].input_schema'
Assert-Equal -Actual $manifest.tasks[0].output_schema -Expected 'nako.addon.external_acquisition_action.response.v1' -Name 'manifest.tasks[0].output_schema'
Write-Host "[sidecar] Manifest OK: $($manifest.id)@$($manifest.version)"

$healthRequest = [ordered]@{
    protocol_version = $manifest.protocol_version
    manifest_id = $manifest.id
    request_id = "local-smoke-health-$([guid]::NewGuid())"
    expected_addon_version = $manifest.version
    expected_resource_count = @($manifest.resources).Count
}
$health = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/health') -Body $healthRequest
Write-Host "[sidecar] Health status: $($health.status); active_profiles=$($health.diagnostics.active_profile_count)"

$enqueueTask = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    task_id = 'external-acquisition-action'
    job_id = "local-smoke-job-$([guid]::NewGuid())"
    request_id = "local-smoke-enqueue-$([guid]::NewGuid())"
    attempt = 1
    payload = [ordered]@{
        schema = 'nako.addon.external_acquisition_action.request.v1'
        target_ref = [ordered]@{
            kind = 'selected_link'
            selected_link_ref = 'local-smoke-selected-link-ref'
        }
        runner_profile_id = 'fixture'
        idempotency_key = 'local-smoke-idempotency-key'
        operation = 'enqueue'
        audit_ref = 'local-smoke-audit-ref'
    }
}
$enqueue = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/tasks/external-acquisition-action') -Body $enqueueTask
Assert-Equal -Actual $enqueue.output.schema -Expected 'nako.addon.external_acquisition_action.response.v1' -Name 'enqueue.output.schema'
Assert-Equal -Actual $enqueue.output.status -Expected 'accepted' -Name 'enqueue.output.status'
$runnerJobRef = $enqueue.output.runner_job_ref
Write-Host "[sidecar] Enqueue OK: runner_job_ref=$runnerJobRef"

$statusTask = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    task_id = 'external-acquisition-action'
    job_id = "local-smoke-job-$([guid]::NewGuid())"
    request_id = "local-smoke-status-$([guid]::NewGuid())"
    attempt = 1
    payload = [ordered]@{
        schema = 'nako.addon.external_acquisition_action.request.v1'
        target_ref = [ordered]@{
            kind = 'runner_job'
            runner_job_ref = $runnerJobRef
        }
        runner_profile_id = 'fixture'
        idempotency_key = 'local-smoke-status-key'
        operation = 'query_status'
    }
}
$status = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/tasks/external-acquisition-action') -Body $statusTask
Assert-Equal -Actual $status.output.state -Expected 'running' -Name 'status.output.state'
Write-Host "[sidecar] Status OK: state=$($status.output.state)"

$cancelTask = [ordered]@{
    protocol_version = $manifest.protocol_version
    addon_id = $manifest.id
    task_id = 'external-acquisition-action'
    job_id = "local-smoke-job-$([guid]::NewGuid())"
    request_id = "local-smoke-cancel-$([guid]::NewGuid())"
    attempt = 1
    payload = [ordered]@{
        schema = 'nako.addon.external_acquisition_action.request.v1'
        target_ref = [ordered]@{
            kind = 'runner_job'
            runner_job_ref = $runnerJobRef
        }
        runner_profile_id = 'fixture'
        idempotency_key = 'local-smoke-cancel-key'
        operation = 'cancel'
    }
}
$cancel = Invoke-Json -Method 'POST' -Url (Join-HttpUrl $SidecarBaseUrl '/tasks/external-acquisition-action') -Body $cancelTask
Assert-Equal -Actual $cancel.output.state -Expected 'cancelled' -Name 'cancel.output.state'
Write-Host "[sidecar] Cancel OK: state=$($cancel.output.state)"

Write-Host '[ok] Local external acquisition runner smoke completed.'
