# Milestones

## M1 - Sidecar Contract Is Declared

Exit criteria:

- Workspace contains `nako-chromecast-renderer`.
- Runtime manifest validates.
- Example manifest matches runtime container manifest.
- README and package files describe safe defaults.

## M2 - Resource Boundary Is Testable

Exit criteria:

- `/renderer-adapter` accepts Nako Addon Resource envelopes.
- Readiness and target discovery produce typed renderer adapter responses.
- Invalid resource envelopes and wrong protocol requests fail safely.

## M3 - Command Plans Are Safe

Exit criteria:

- Dispatch validates command envelopes before building plans.
- Only HTTP(S) cast-safe transport URLs are accepted.
- Forbidden raw source/local/token-like facts are rejected.
- Live LAN/hardware execution remains an explicit non-default smoke gate.

## M4 - Lane Is Ready For Host Integration

Exit criteria:

- Focused tests pass.
- Formatting and whitespace gates pass for touched paths.
- Workstream evidence and handoff are current.
- Changes are committed without staging unrelated existing dirty files.
