use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
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
    let mut group = c.benchmark_group("serde/duration_serialize");

    for millis in [1_234u64, 50_000u64, 999_999u64] {
        let millis_value = MillisContainer {
            duration: Duration::from_millis(millis),
        };

        group.bench_with_input(BenchmarkId::new("as_millis", millis), &millis_value, |b, value| {
            b.iter(|| {
                serde_json::to_string(black_box(value))
                    .expect("Millis serialization should succeed")
            })
        });
    }

    for secs in [42u64, 600u64, 3_600u64] {
        let secs_value = SecsContainer {
            duration: Duration::from_secs(secs),
        };

        group.bench_with_input(BenchmarkId::new("as_secs", secs), &secs_value, |b, value| {
            b.iter(|| {
                serde_json::to_string(black_box(value))
                    .expect("Seconds serialization should succeed")
            })
        });
    }

    group.finish();
}

fn benchmark_duration_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde/duration_deserialize");

    for millis_json in [
        r#"{"duration":1234}"#,
        r#"{"duration":50000}"#,
        r#"{"duration":999999}"#,
    ] {
        group.bench_with_input(
            BenchmarkId::new("as_millis", millis_json.len()),
            &millis_json,
            |b, json| {
                b.iter(|| {
                    serde_json::from_str::<MillisContainer>(black_box(json))
                        .expect("Millis deserialization should succeed")
                })
            },
        );
    }

    for secs_json in [
        r#"{"duration":42}"#,
        r#"{"duration":600}"#,
        r#"{"duration":3600}"#,
    ] {
        group.bench_with_input(
            BenchmarkId::new("as_secs", secs_json.len()),
            &secs_json,
            |b, json| {
                b.iter(|| {
                    serde_json::from_str::<SecsContainer>(black_box(json))
                        .expect("Seconds deserialization should succeed")
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_duration_serialize,
    benchmark_duration_deserialize
);
criterion_main!(benches);
