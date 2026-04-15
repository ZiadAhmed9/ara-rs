//! Criterion benchmarks for ara-com-someip.
//!
//! Scenarios:
//! 1. Serialization round-trip (encode + decode) for primitives, structs, strings, vectors
//! 2. Request/response loopback latency over UDP

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use ara_com::error::AraComError;
use ara_com::transport::{AraDeserialize, AraSerialize};

// ---------------------------------------------------------------------------
// 1. Serialization benchmarks
// ---------------------------------------------------------------------------

fn bench_serialize_u64(c: &mut Criterion) {
    let val: u64 = 0xDEAD_BEEF_CAFE_BABE;
    let mut buf = Vec::with_capacity(8);

    c.bench_function("serialize/u64", |b| {
        b.iter(|| {
            buf.clear();
            black_box(&val).ara_serialize(&mut buf).unwrap();
        })
    });
}

fn bench_deserialize_u64(c: &mut Criterion) {
    let val: u64 = 0xDEAD_BEEF_CAFE_BABE;
    let mut buf = Vec::new();
    val.ara_serialize(&mut buf).unwrap();

    c.bench_function("deserialize/u64", |b| {
        b.iter(|| {
            let _ = black_box(u64::ara_deserialize(black_box(&buf)).unwrap());
        })
    });
}

fn bench_serialize_f64(c: &mut Criterion) {
    let val: f64 = 12.6;
    let mut buf = Vec::with_capacity(8);

    c.bench_function("serialize/f64", |b| {
        b.iter(|| {
            buf.clear();
            black_box(&val).ara_serialize(&mut buf).unwrap();
        })
    });
}

fn bench_serialize_string(c: &mut Criterion) {
    let val = "Hello, AUTOSAR!".to_string();
    let mut buf = Vec::with_capacity(64);

    c.bench_function("serialize/string_15b", |b| {
        b.iter(|| {
            buf.clear();
            black_box(&val).ara_serialize(&mut buf).unwrap();
        })
    });
}

fn bench_deserialize_string(c: &mut Criterion) {
    let val = "Hello, AUTOSAR!".to_string();
    let mut buf = Vec::new();
    val.ara_serialize(&mut buf).unwrap();

    c.bench_function("deserialize/string_15b", |b| {
        b.iter(|| {
            let _ = black_box(String::ara_deserialize(black_box(&buf)).unwrap());
        })
    });
}

fn bench_serialize_vec_u32(c: &mut Criterion) {
    let val: Vec<u32> = (0..256).collect();
    let mut buf = Vec::with_capacity(1028);

    let mut group = c.benchmark_group("serialize");
    group.throughput(Throughput::Bytes((256 * 4) as u64));
    group.bench_function("vec_u32_256", |b| {
        b.iter(|| {
            buf.clear();
            black_box(&val).ara_serialize(&mut buf).unwrap();
        })
    });
    group.finish();
}

fn bench_deserialize_vec_u32(c: &mut Criterion) {
    let val: Vec<u32> = (0..256).collect();
    let mut buf = Vec::new();
    val.ara_serialize(&mut buf).unwrap();

    let mut group = c.benchmark_group("deserialize");
    group.throughput(Throughput::Bytes((256 * 4) as u64));
    group.bench_function("vec_u32_256", |b| {
        b.iter(|| {
            let _ = black_box(Vec::<u32>::ara_deserialize(black_box(&buf)).unwrap());
        })
    });
    group.finish();
}

/// Benchmark a struct-like round-trip: serialize 3 fields (f64 + f64 + bool),
/// then deserialize them back. Mimics BatteryStatus { voltage, current, charging }.
fn bench_struct_roundtrip(c: &mut Criterion) {
    let voltage: f64 = 12.6;
    let current: f64 = 3.2;
    let charging: bool = true;
    let mut buf = Vec::with_capacity(17);

    c.bench_function("roundtrip/struct_3fields", |b| {
        b.iter(|| {
            buf.clear();
            voltage.ara_serialize(&mut buf).unwrap();
            current.ara_serialize(&mut buf).unwrap();
            charging.ara_serialize(&mut buf).unwrap();

            let mut offset = 0usize;
            let v = f64::ara_deserialize(&buf[offset..]).unwrap();
            offset += 8;
            let c = f64::ara_deserialize(&buf[offset..]).unwrap();
            offset += 8;
            let ch = bool::ara_deserialize(&buf[offset..]).unwrap();
            black_box((v, c, ch));
        })
    });
}

// ---------------------------------------------------------------------------
// 2. Request/response loopback latency
// ---------------------------------------------------------------------------

fn bench_request_response_loopback(c: &mut Criterion) {
    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::sync::Arc;

    use bytes::Bytes;
    use futures_core::future::BoxFuture;

    use ara_com::transport::{MessageHeader, MessageType, ReturnCode, Transport};
    use ara_com::types::{InstanceId, MethodId, ServiceId};

    use ara_com_someip::config::{
        EndpointConfig, RemoteServiceConfig, SdConfig, ServiceConfig, SomeIpConfig,
    };
    use ara_com_someip::transport::SomeIpTransport;

    let service_id = ServiceId(0xBEEF);
    let instance_id = InstanceId(0x0001);
    let method_id = MethodId(0x0001);

    let rt = tokio::runtime::Runtime::new().unwrap();

    let (proxy, _skeleton_arc) = rt.block_on(async {
        // Skeleton
        let mut skeleton = SomeIpTransport::new(SomeIpConfig {
            unicast: Ipv4Addr::LOCALHOST,
            client_id: 0x0001,
            discovery_mode: Default::default(),
            sd_config: SdConfig::default(),
            services: vec![ServiceConfig {
                service_id,
                instance_id,
                endpoint: EndpointConfig {
                    udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
                    tcp: None,
                    udp_threshold: 1400,
                },
                event_groups: vec![],
            }],
            remote_services: vec![],
            udp_threshold: 1400,
        });
        skeleton.bind().await.unwrap();
        let skel_port = skeleton.local_addr().unwrap().port();

        // Echo handler — returns whatever payload it receives
        skeleton
            .register_request_handler(
                service_id,
                instance_id,
                Box::new(
                    |_header: MessageHeader,
                     payload: Bytes|
                     -> BoxFuture<'static, Result<Bytes, AraComError>> {
                        Box::pin(async move { Ok(payload) })
                    },
                ),
            )
            .await
            .unwrap();

        let skeleton_arc = Arc::new(skeleton);

        // Proxy
        let mut proxy = SomeIpTransport::new(SomeIpConfig {
            unicast: Ipv4Addr::LOCALHOST,
            client_id: 0x0002,
            discovery_mode: Default::default(),
            sd_config: SdConfig::default(),
            services: vec![],
            remote_services: vec![RemoteServiceConfig {
                service_id,
                instance_id,
                endpoint: EndpointConfig {
                    udp: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, skel_port)),
                    tcp: None,
                    udp_threshold: 1400,
                },
            }],
            udp_threshold: 1400,
        });
        proxy.bind().await.unwrap();

        let proxy = Arc::new(proxy);
        (proxy, skeleton_arc)
    });

    let header = MessageHeader {
        service_id,
        method_id,
        instance_id,
        session_id: 0,
        message_type: MessageType::Request,
        return_code: ReturnCode::Ok,
    };
    let payload = Bytes::from_static(&[0x01]);

    c.bench_function("transport/request_response_loopback", |b| {
        b.iter(|| {
            rt.block_on(async {
                let resp = proxy
                    .send_request(black_box(header.clone()), black_box(payload.clone()))
                    .await
                    .unwrap();
                black_box(resp);
            });
        })
    });
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

criterion_group!(
    serialization,
    bench_serialize_u64,
    bench_deserialize_u64,
    bench_serialize_f64,
    bench_serialize_string,
    bench_deserialize_string,
    bench_serialize_vec_u32,
    bench_deserialize_vec_u32,
    bench_struct_roundtrip,
);

criterion_group!(transport, bench_request_response_loopback,);

criterion_main!(serialization, transport);
