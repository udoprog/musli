use criterion::Criterion;

#[cfg(feature = "musli-zerocopy")]
use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
#[cfg(feature = "rkyv")]
use rkyv::rancor::Failure;
#[cfg(feature = "rkyv")]
use rkyv::vec::ArchivedVec;
#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize, Portable, Serialize};

// Musli zero copy
#[derive(ZeroCopy, Debug)]
#[repr(C)]
#[cfg(feature = "musli-zerocopy")]
struct DataMusli {
    slice: Ref<[Ref<[u8]>]>,
}

// Rkyv zero copy
#[derive(Archive, Serialize)]
#[cfg(feature = "rkyv")]
struct DataRkyv {
    slice: Vec<Vec<u8>>,
}

fn criterion_benchmark(c: &mut Criterion) {
    #[allow(unused)]
    let vec_size = 1000;
    #[allow(unused)]
    let some_long_str = "some_long_string".repeat(100).to_string();
    #[allow(unused)]
    let mut g = c.benchmark_group("zerocopy");

    #[cfg(feature = "musli-zerocopy")]
    g.bench_function("musli/checked", |b| {
        use std::hint::black_box;

        let mut buf = OwnedBuf::new();

        let maybe_uninit = buf.store_uninit::<DataMusli>();

        let mut vec = Vec::new();

        for _ in 0..vec_size {
            let str = format!("some_long_string{}", some_long_str);
            vec.push(buf.store_slice(str.as_bytes()));
        }

        let person = DataMusli {
            slice: buf.store_slice(&vec),
        };

        buf.load_uninit_mut(maybe_uninit).write(&person);

        b.iter(|| {
            let mut len = 0;

            let key = buf.load_at::<DataMusli>(0).unwrap();
            let slice = buf.load(key.slice).unwrap();

            for item in slice.iter() {
                len += black_box(buf.load(*item).unwrap()).len();
            }

            len
        });
    });

    #[cfg(feature = "rkyv")]
    g.bench_function("rkyv/unchecked", |b| {
        use std::hint::black_box;

        let data = DataRkyv {
            slice: (0..vec_size)
                .map(|_| some_long_str.as_bytes().to_vec())
                .collect(),
        };

        let bytes = rkyv::to_bytes::<Failure>(&data).unwrap();

        b.iter(|| {
            let mut len = 0;

            let archived = unsafe { rkyv::access_unchecked::<ArchivedDataRkyv>(&bytes) };

            for item in archived.slice.iter() {
                len += black_box(item.as_slice()).len();
            }

            len
        });
    });

    #[cfg(feature = "rkyv")]
    g.bench_function("rkyv/checked", |b| {
        use std::hint::black_box;

        let data = DataRkyv {
            slice: (0..vec_size)
                .map(|_| some_long_str.as_bytes().to_vec())
                .collect(),
        };

        let bytes = rkyv::to_bytes::<Failure>(&data).unwrap();

        b.iter(|| {
            let mut len = 0;

            let archived = rkyv::access::<ArchivedDataRkyv, Failure>(&bytes).unwrap();

            for item in archived.slice.iter() {
                len += black_box(item.as_slice()).len();
            }

            len
        });
    });
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
