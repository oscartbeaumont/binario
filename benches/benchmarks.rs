use std::sync::Arc;

use binario::{decode, encode};
use criterion::{criterion_group, criterion_main, Criterion};

use pprof::criterion::{Output, PProfProfiler};

const BIG_STRING_LEN: usize = 30; // TODO: Make higher once my impl is fixed
const BIG_STRING_ITERS: usize = 1000;

fn bench(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("proto-serialize-big-string", |b| {
        let string = Arc::new((0..BIG_STRING_LEN).map(|_| "X").collect::<String>());
        b.to_async(&rt).iter(move || {
            let string = string.clone();
            async move {
                for _ in 0..BIG_STRING_ITERS {
                    let mut buf = Vec::with_capacity(BIG_STRING_LEN + 1);
                    encode(&string, &mut buf).await.unwrap();
                    assert_eq!(buf.len(), BIG_STRING_LEN + 1);
                }
            }
        })
    });

    c.bench_function("bincode-serialize-big-string", |b| {
        let string = Arc::new((0..BIG_STRING_LEN).map(|_| "X").collect::<String>());
        b.to_async(&rt).iter(move || {
            let string = string.clone();
            async move {
                for _ in 0..BIG_STRING_ITERS {
                    let mut buf = Vec::with_capacity(BIG_STRING_LEN + 8); // We preallocate to try and avoid the effects of reallocation
                    bincode::serialize_into(&mut buf, &*string).unwrap();
                    assert_eq!(buf.len(), BIG_STRING_LEN + 8);
                }
            }
        })
    });

    c.bench_function("proto-bothways-big-string", |b| {
        let string = Arc::new((0..BIG_STRING_LEN).map(|_| "X").collect::<String>());
        b.to_async(&rt).iter(move || {
            let string = string.clone();
            async move {
                for _ in 0..BIG_STRING_ITERS {
                    let mut buf = Vec::with_capacity(BIG_STRING_LEN + 1);
                    encode(&string, &mut buf).await.unwrap();
                    assert_eq!(buf.len(), BIG_STRING_LEN + 1);
                    let string2: String = decode(&buf[..]).await.unwrap();
                    assert_eq!(&*string, &*string2);
                }
            }
        })
    });

    c.bench_function("bincode-bothways-big-string", |b| {
        let string = Arc::new((0..BIG_STRING_LEN).map(|_| "X").collect::<String>());
        b.to_async(&rt).iter(move || {
            let string = string.clone();
            async move {
                for _ in 0..BIG_STRING_ITERS {
                    let mut buf = Vec::with_capacity(BIG_STRING_LEN + 8); // We preallocate to try and avoid the effects of reallocation
                    bincode::serialize_into(&mut buf, &*string).unwrap();
                    assert_eq!(buf.len(), BIG_STRING_LEN + 8);
                    let string2: String = bincode::deserialize(&buf[..]).unwrap();
                    assert_eq!(&*string, &*string2);
                }
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)));
    targets = bench
}
criterion_main!(benches);
