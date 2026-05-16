// TODO unclear why we need these extern crates here?
extern crate criterion;
extern crate pprof;
extern crate roslibrust_serde_rosmsg;
extern crate serde;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

const IMAGE_DATA: &[u8] = include_bytes!("../src/datatests/sensor_msgs_image_1080p.bin");

use pprof::criterion::{Output, PProfProfiler};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Header {
    pub seq: u32,
    pub stamp: Time,
    pub frame_id: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Time {
    pub secs: u32,
    pub nsecs: u32,
}

// Includes serde_bytes optimization
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct VecBytesImage {
    pub header: Header,
    pub height: u32,
    pub width: u32,
    pub encoding: String,
    pub is_bigendian: u8,
    pub step: u32,
    // serde_bytes optimization here makes deserialization of an image ~97% faster
    // Without it deserializing a 1080p color image took ~22.2ms on a Ryzen 3950x
    // With it that drops to 520us on the same system
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[inline]
fn parse_image() {
    let image: VecBytesImage = roslibrust_serde_rosmsg::from_slice(IMAGE_DATA).unwrap();
    black_box(image);
}

#[inline]
fn serialize_image_to_new_vec(image: &VecBytesImage) {
    black_box(roslibrust_serde_rosmsg::to_vec(image).unwrap());
}

#[inline]
fn serialize_image_to_prealloc_cursor(
    image: &VecBytesImage,
    cursor: &mut std::io::Cursor<Vec<u8>>,
) {
    cursor.set_position(0);
    black_box(roslibrust_serde_rosmsg::to_writer(cursor, image).unwrap());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_image", |b| {
        b.iter(|| parse_image())
    });

    let image: VecBytesImage = roslibrust_serde_rosmsg::from_slice(IMAGE_DATA).unwrap();

    // Benchmark serialization to a new Vec (allocates on each call)
    c.bench_function("serialize_image_to_new_vec", |b| {
        b.iter(|| serialize_image_to_new_vec(&image))
    });

    // Benchmark serialization to a pre-allocated Vec (reuses allocation)
    // Pre-allocate a buffer large enough for the serialized image
    let serialized_size = roslibrust_serde_rosmsg::to_vec(&image).unwrap().len();
    c.bench_function("serialize_image_to_prealloc_cursor", |b| {
        let mut cursor = std::io::Cursor::new(Vec::with_capacity(serialized_size));
        b.iter(|| serialize_image_to_prealloc_cursor(&image, &mut cursor))
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
);
criterion_main!(benches);
