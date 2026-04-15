# Sprint 14 — Multi-Binding (UDP + TCP Simultaneous)

## Goal

Allow a single service to be offered over both UDP and TCP simultaneously, rather than routing per-message based on payload size.

## Why Now

Real automotive deployments commonly bind the same service to both reliable (TCP) and unreliable (UDP) endpoints. The current `udp_threshold` routing is a simplification that blocks realistic multi-transport setups.

## Scope

- Offer a service on both UDP and TCP endpoints at the same time
- Client-side binding selection (prefer reliable, prefer fast, or auto)
- SD offers advertise both endpoints
- Update the battery service example to demonstrate dual binding

## Out of Scope

- DDS or MQTT transport backends
- Automatic failover between transports

## Exit Criteria

- A single service instance accepts requests on both UDP and TCP
- SD offers include both endpoint options
- Integration test confirms both paths work concurrently
