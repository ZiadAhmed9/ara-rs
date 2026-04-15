# Benchmarks

ara-rs includes criterion benchmarks measuring serialization throughput and SOME/IP transport latency.

## Running

```bash
cargo bench -p ara-com-someip
```

For a quick pass (fewer iterations, useful for smoke-testing):

```bash
cargo bench -p ara-com-someip -- --quick
```

HTML reports are generated in `target/criterion/`.

## Scenarios

### Serialization

| Benchmark | What it measures |
|-----------|-----------------|
| `serialize/u64` | Big-endian u64 encode |
| `deserialize/u64` | Big-endian u64 decode |
| `serialize/f64` | IEEE 754 f64 encode |
| `serialize/string_15b` | SOME/IP string encode (15-byte payload, BOM + NUL) |
| `deserialize/string_15b` | SOME/IP string decode |
| `serialize/vec_u32_256` | 256-element Vec\<u32\> encode (1 KiB payload) |
| `deserialize/vec_u32_256` | 256-element Vec\<u32\> decode |
| `roundtrip/struct_3fields` | Encode + decode a 3-field struct (f64 + f64 + bool) |

### Transport

| Benchmark | What it measures |
|-----------|-----------------|
| `transport/request_response_loopback` | Full UDP round-trip: proxy sends request, skeleton echoes payload, proxy receives response |

## Reference Results

Measured on a single machine (results vary by hardware and system load):

| Benchmark | Time | Throughput |
|-----------|------|------------|
| serialize/u64 | ~1.4 ns | — |
| deserialize/u64 | ~3.1 ns | — |
| serialize/f64 | ~1.4 ns | — |
| serialize/string_15b | ~3.7 ns | — |
| deserialize/string_15b | ~19 ns | — |
| serialize/vec_u32_256 | ~335 ns | ~2.8 GiB/s |
| deserialize/vec_u32_256 | ~600 ns | ~1.6 GiB/s |
| roundtrip/struct_3fields | ~8.3 ns | — |
| request_response_loopback | ~12 us | — |

**Environment:** Linux x86_64, Rust stable, UDP loopback (127.0.0.1), single-threaded criterion runner with tokio multi-thread runtime for transport benchmarks.

## Methodology

- All benchmarks use [criterion.rs](https://bheisler.github.io/criterion.rs/book/) with default statistical settings (100 samples, 5-second warm-up).
- Serialization benchmarks measure encode or decode in isolation, with pre-allocated buffers to exclude allocation noise.
- The transport benchmark measures wall-clock time for a complete UDP request/response round-trip over loopback, including kernel socket overhead.
- The `--quick` flag reduces sample count for CI smoke-testing but produces less statistically reliable results.

## Caveats

- **Loopback only.** Transport benchmarks run on `127.0.0.1` and do not reflect real network latency, jitter, or packet loss.
- **Single service.** The transport benchmark exercises one request handler; contention under multiple concurrent services is not measured.
- **No vsomeip comparison.** These are absolute numbers for ara-rs, not a head-to-head comparison. A fair vsomeip comparison would require identical service definitions, payload sizes, and network configuration.
- **System load.** Results are sensitive to background processes, CPU frequency scaling, and kernel scheduler behavior. Pin to a specific core and disable turbo boost for reproducible results.
