use criterion::{black_box, criterion_group, criterion_main, Criterion};
use criterion::async_executor::SmolExecutor;

use kosmos_tile::{TilemapGpuIterator, Tile};


pub fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("tick");
    let width = 32 * (3*2 + 1);
    let map = TilemapGpuIterator::new(width);
    let window = vec![Tile::default(); width.pow(3)];
    g.sample_size(10);
    //c.bench_function("tile sync", |b| b.to_async(SmolExecutor).iter(|| map.sync()));
    g.bench_function("tile tick", |b| b.iter(|| { map.tick(); map.wait() }));
    //c.bench_function("tile upload and tick", |b| b.iter(|| { map.upload(&window).unwrap(); map.tick(); map.wait() }));
    //c.bench_function("tile download", |b| b.to_async(SmolExecutor).iter(|| black_box(map.download())));
    /*c.bench_function("tile upload, tick, and download", |b| b.to_async(SmolExecutor).iter(|| async { 
        map.upload(&window).unwrap();
        map.tick();
        black_box(map.download().await.unwrap());
    }));*/
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
