# Sprint 11 — Field Support

## Goal

Generate getter/setter/notifier code for ARXML field definitions and wire them through the transport.

## Why Now

Fields are the third leg of AUTOSAR service interfaces alongside methods and events. Production ARXML uses them heavily; without field codegen, adopters hit a wall as soon as their service definitions include fields.

## Scope

- Parse ARXML field definitions (getter, setter, notifier attributes)
- Generate field accessor methods on proxies (get, set)
- Generate notifier wiring on skeletons (on-change notifications)
- Wire field operations through the `Transport` trait
- Add field round-trip to the battery or diagnostics example

## Out of Scope

- Field persistence across restarts
- Field-level access control or authentication

## Exit Criteria

- Generated proxy has typed get/set methods for at least one field
- Skeleton publishes on-change notifications when a field is updated
- Round-trip integration test passes over loopback
