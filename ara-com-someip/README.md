# ara-com-someip

SOME/IP transport backend for [`ara-com`](https://crates.io/crates/ara-com) Adaptive AUTOSAR communication.

## Overview

`ara-com-someip` implements the `ara-com` `Transport` trait over SOME/IP, providing:

- **UDP and TCP transport** with payload-size-based routing
- **Request/response correlation** with session tracking
- **Fire-and-forget** (SOME/IP `RequestNoReturn`)
- **Event notifications** with typed event streams and backpressure
- **SOME/IP-SD** multicast service discovery (offer/find/subscribe lifecycle, TTL tracking)
- **Event-group-aware routing** — notifications fan out only to matching subscribers
- **Wire compatibility** with vsomeip (byte-level validated)

## Usage

```toml
[dependencies]
ara-com = "0.1"
ara-com-someip = "0.1"
```

```rust,ignore
use ara_com_someip::SomeIpTransport;
use ara_com_someip::config::SomeIpConfig;

let config = SomeIpConfig::new([127, 0, 0, 1], 30509);
let transport = SomeIpTransport::new(config).await?;
```

In practice, use `cargo-arxml` to generate typed proxies and skeletons from ARXML, then plug in `SomeIpTransport` as the backend.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Part of the [ara-rs](https://github.com/ZiadAhmed9/ara-rs) workspace.
