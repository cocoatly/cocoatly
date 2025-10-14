use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use cocoatly_core::*;
use std::path::Path;
use std::fs;
use tempfile::tempdir;

// Benchmark for file operations
fn file_operations_benchmark(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    
    c.bench_function("write_1mb_file", |b| b.iter(|| {
        let content = vec![0u8; 1024 * 1024]; // 1MB
        fs::write(&file_path, &content).unwrap();
    }));
    
    c.bench_function("read_1mb_file", |b| {
        let content = vec![0u8; 1024 * 1024];
        fs::write(&file_path, &content).unwrap();
        
        b.iter(|| {
            let _ = fs::read(&file_path).unwrap();
        })
    });
}

// Benchmark for compression
fn compression_benchmark(c: &mut Criterion) {
    use cocoatly_compression::{compress, decompress};
    
    let data = vec![0u8; 1024 * 1024]; // 1MB of zeros
    
    let mut group = c.benchmark_group("compression");
    
    group.bench_function("compress_1mb_zeros", |b| {
        b.iter(|| compress(&data).unwrap())
    });
    
    let compressed = compress(&data).unwrap();
    group.bench_function("decompress_1mb_zeros", |b| {
        b.iter(|| decompress(&compressed).unwrap())
    });
    
    group.finish();
}

// Benchmark for hashing
fn hashing_benchmark(c: &mut Criterion) {
    use cocoatly_crypto::hash::hash_data;
    
    let data_sizes = [1024, 1024 * 10, 1024 * 100]; // 1KB, 10KB, 100KB
    
    let mut group = c.benchmark_group("hashing");
    
    for size in &data_sizes {
        let data = vec![0u8; *size];
        
        group.bench_with_input(
            BenchmarkId::new("blake3", size),
            &data,
            |b, data| b.iter(|| hash_data(data, "blake3").unwrap())
        );
        
        group.bench_with_input(
            BenchmarkId::new("sha256", size),
            &data,
            |b, data| b.iter(|| hash_data(data, "sha256").unwrap())
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    file_operations_benchmark,
    compression_benchmark,
    hashing_benchmark
);
criterion_main!(benches);
