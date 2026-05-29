#!/usr/bin/env python3
import argparse
import json
import sys
import time
import urllib.error
import urllib.request


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--kind", choices=["metadata", "notification", "chromecast"], required=True)
    parser.add_argument("--base-url", required=True)
    parser.add_argument("--manifest-id", required=True)
    parser.add_argument("--addon-version", required=True)
    parser.add_argument("--protocol-version", required=True)
    args = parser.parse_args()

    manifest = get_json_with_retry(f"{args.base_url}/manifest.json")
    assert_equal(manifest["id"], args.manifest_id, "manifest.id")
    assert_equal(manifest["version"], args.addon_version, "manifest.version")
    assert_equal(
        manifest["protocol_version"],
        args.protocol_version,
        "manifest.protocol_version",
    )

    if args.kind == "metadata":
        smoke_metadata(args.base_url, manifest)
    elif args.kind == "notification":
        smoke_notification(args.base_url, manifest)
    else:
        smoke_chromecast(args.base_url, manifest)

    print(
        json.dumps(
            {
                "kind": args.kind,
                "id": manifest["id"],
                "version": manifest["version"],
                "protocol_version": manifest["protocol_version"],
            },
            separators=(",", ":"),
        )
    )
    return 0


def smoke_metadata(base_url: str, manifest: dict) -> None:
    health = post_json(f"{base_url}/health", health_request(manifest, "metadata-health"))
    assert_equal(health["status"], "ok", "health.status")

    metadata = post_json(
        f"{base_url}/metadata",
        {
            "protocol_version": manifest["protocol_version"],
            "addon_id": manifest["id"],
            "resource": "metadata",
            "request_id": "metadata-resource-smoke",
            "payload": {
                "title": "The Matrix",
                "year": 1999,
                "language": "en-US",
            },
        },
    )
    assert_equal(metadata["addon_id"], manifest["id"], "metadata.addon_id")
    assert_equal(metadata["resource"], "metadata", "metadata.resource")
    assert_min_count(metadata["payload"]["candidates"], 1, "metadata.payload.candidates")
    assert_min_count(metadata["artifacts"], 1, "metadata.artifacts")


def smoke_notification(base_url: str, manifest: dict) -> None:
    health = post_json(f"{base_url}/health", health_request(manifest, "notification-health"))
    assert_equal(health["status"], "ok", "health.status")
    assert_equal(health["diagnostics"]["mode"], "ack_only", "health.diagnostics.mode")

    event = post_json(
        f"{base_url}/events/library-scanned",
        {
            "protocol_version": manifest["protocol_version"],
            "addon_id": manifest["id"],
            "subscription_id": "library-scanned-notification",
            "event_id": "notification-event-smoke",
            "event_kind": "library.scanned",
            "subject_kind": "library",
            "subject_id": "library-smoke",
            "occurred_at": "2026-05-25T00:00:00.000Z",
            "attempt": 1,
            "payload": {"library_id": "library-smoke", "secret": "nako_at_should_not_echo"},
        },
    )
    assert_equal(event["addon_id"], manifest["id"], "event.addon_id")
    assert_equal(event["output"]["mode"], "ack_only", "event.output.mode")
    if "nako_at_should_not_echo" in json.dumps(event, separators=(",", ":")):
        raise AssertionError("event response echoed a secret payload value")


def smoke_chromecast(base_url: str, manifest: dict) -> None:
    health = post_json(f"{base_url}/health", health_request(manifest, "chromecast-health"))
    readiness = health["diagnostics"]["readiness"]
    assert_equal(readiness["protocol"], "chromecast", "health.diagnostics.readiness.protocol")

    response = post_json(
        f"{base_url}/renderer-adapter",
        {
            "protocol_version": manifest["protocol_version"],
            "addon_id": manifest["id"],
            "resource": "renderer_adapter",
            "request_id": "chromecast-readiness-smoke",
            "payload": {
                "action": "inspect_readiness",
                "protocol": "chromecast",
            },
        },
    )
    assert_equal(response["resource"], "renderer_adapter", "readiness.resource")
    assert_equal(response["payload"]["kind"], "readiness", "readiness.payload.kind")


def health_request(manifest: dict, request_id: str) -> dict:
    return {
        "protocol_version": manifest["protocol_version"],
        "manifest_id": manifest["id"],
        "request_id": request_id,
        "expected_addon_version": manifest["version"],
        "expected_resource_count": len(manifest["resources"]),
    }


def get_json_with_retry(url: str, timeout_seconds: float = 20.0) -> dict:
    deadline = time.time() + timeout_seconds
    last_error = None
    while time.time() < deadline:
        try:
            return get_json(url)
        except Exception as error:  # noqa: BLE001 - this is a smoke retry loop.
            last_error = error
            time.sleep(0.5)
    raise RuntimeError(f"GET {url} failed before deadline: {last_error}")


def get_json(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=2) as response:
        return json.load(response)


def post_json(url: str, body: dict) -> dict:
    data = json.dumps(body, separators=(",", ":")).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        detail = error.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"POST {url} failed with HTTP {error.code}: {detail}") from error


def assert_equal(actual, expected, name: str) -> None:
    if actual != expected:
        raise AssertionError(f"{name} expected {expected!r} but got {actual!r}")


def assert_min_count(items, minimum: int, name: str) -> None:
    if len(items) < minimum:
        raise AssertionError(f"{name} expected at least {minimum} item(s) but got {len(items)}")


if __name__ == "__main__":
    sys.exit(main())
