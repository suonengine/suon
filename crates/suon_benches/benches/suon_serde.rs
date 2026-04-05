use criterion::{Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use std::{hint::black_box, time::Duration};
use suon_serde::duration::{as_millis, as_secs};

#[derive(Serialize, Deserialize)]
struct MillisContainer {
    #[serde(with = "as_millis")]
    duration: Duration,
}

#[derive(Serialize, Deserialize)]
struct SecsContainer {
    #[serde(with = "as_secs")]
    duration: Duration,
}

fn benchmark_duration_serialize(c: &mut Criterion) {
    let millis_value = MillisContainer {
        duration: Duration::from_millis(1_234),
    };
    let secs_value = SecsContainer {
        duration: Duration::from_secs(42),
    };

    c.bench_function("serde/duration_serialize", |b| {
        b.iter(|| {
            let millis = serde_json::to_string(black_box(&millis_value))
                .expect("Millis serialization should succeed");
            let secs = serde_json::to_string(black_box(&secs_value))
                .expect("Seconds serialization should succeed");
            (millis, secs)
        })
    });
}

fn benchmark_duration_deserialize(c: &mut Criterion) {
    let millis_json = r#"{"duration":1234}"#;
    let secs_json = r#"{"duration":42}"#;

    c.bench_function("serde/duration_deserialize", |b| {
        b.iter(|| {
            let millis = serde_json::from_str::<MillisContainer>(black_box(millis_json))
                .expect("Millis deserialization should succeed");
            let secs = serde_json::from_str::<SecsContainer>(black_box(secs_json))
                .expect("Seconds deserialization should succeed");
            (millis, secs)
        })
    });
}

criterion_group!(
    benches,
    benchmark_duration_serialize,
    benchmark_duration_deserialize
);
criterion_main!(benches);
