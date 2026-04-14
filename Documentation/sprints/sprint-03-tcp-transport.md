# Sprint 03 - TCP Transport

## Goal

Complete the SOME/IP transport story by adding TCP transport support in a way that preserves current UDP behavior and test confidence.

## Why This Sprint Comes Now

TCP transport is core product functionality, and it should land before packaging, docs publishing, or ecosystem demos.

## Scope

- Implement TCP transport path in `ara-com-someip`.
- Support routing logic for payloads or services that should use TCP.
- Add integration tests covering TCP request and response behavior.
- Document transport selection rules and limitations.

## Dependencies

- Sprint 01 complete
- Sprint 02 complete or at least public transport APIs documented enough to review safely

## Out Of Scope

- vsomeip interop demo
- C++ bridge generation
- benchmark tuning

## Suggested PR Breakdown

1. TCP socket plumbing and config surface
2. Request and response path over TCP
3. Tests and docs

## Implementation Checklist

- Audit current `ara-com` transport abstractions for TCP assumptions or gaps.
- Implement connection management and framing where needed.
- Ensure UDP-only flows keep working unchanged.
- Add tests for:
  - request and response over TCP
  - fallback or routing thresholds
  - error handling on disconnect or timeout
- Update example or fixture configuration if transport selection needs a real usage path.

## Testing Requirements Before Merge

### Coverage Goals

- Full coverage of TCP-specific routing, framing, and lifecycle paths introduced in this sprint.
- Regression coverage proving UDP behavior is unchanged.
- Negative coverage for socket failure, timeout, disconnect, and malformed frame handling.

### Required Test Types

- Unit tests for transport selection logic and TCP framing helpers.
- Integration tests for request and response over TCP loopback.
- Concurrency tests for multiple in-flight TCP requests.
- Regression tests re-running key UDP integration scenarios.

### Required Positive Cases

- Proxy and skeleton communicate successfully over TCP.
- Payloads that should route to TCP do so deterministically.
- TCP request and response correlation works for single and multiple requests.
- Connection reuse or reconnect behavior works as designed.

### Required Negative Cases

- Remote disconnect during request handling returns a controlled error.
- Timeout on TCP request does not hang the caller indefinitely.
- Malformed or truncated TCP frame is rejected cleanly.
- Misconfigured TCP endpoint fails with actionable diagnostics.

### Non-Regression Checks

- Existing UDP request and response tests remain green.
- Existing SOME/IP-SD behavior remains unchanged unless explicitly documented.
- Existing wire-format compatibility tests still pass where applicable.

### Performance And Stability Checks

- No obvious file descriptor leak in repeated connect and disconnect scenarios.
- Repeated integration runs are stable and not flaky.

### Merge Gate

- TCP happy-path and failure-path coverage exists in automation.
- UDP regression suite passes unchanged.
- At least one end-to-end generated proxy and skeleton test uses TCP successfully.

## Exit Criteria

- A generated proxy and skeleton can communicate over TCP in tests.
- Transport selection behavior is deterministic and documented.
- Existing UDP integration tests remain green.

## Merge Notes

Keep protocol behavior, config shape, and docs in the same sprint. Avoid bundling unrelated cleanup because transport review will already be deep.
