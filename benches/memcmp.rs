use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use rayon::ScopeFifo;

/// Compares lexicographically the common part of these slices, i.e. takes the smallest length and
/// compares within that.
fn min_memcmp(a: &[u8], b: &[u8]) -> Ordering {
    let len = a.len().min(b.len());
    a[..len].cmp(&b[..len])
}

fn rayon_memcmp(a: &[u8], b: &[u8]) -> Ordering {
    let len = a.len().min(b.len());
    let (a1, a2) = a[..len].split_at(len/2);
    let (b1, b2) = b[..len].split_at(len/2);
    let (first, last) = rayon::join(||a1.cmp(b1), ||a2.cmp(b2));
    first.then(last)
}

fn scope_memcmp(a: &[u8], b: &[u8], s: &ScopeFifo) -> Ordering {
    let len = a.len().min(b.len());
    let (a1, a2) = a[..len].split_at(len/2);
    let (b1, b2) = b[..len].split_at(len/2);
    let (first, last) = rayon::join(||a1.cmp(b1), ||a2.cmp(b2));
    first.then(last)
}

fn bench_fibs(c: &mut Criterion) {
    let mut group = c.benchmark_group("memcmp");
    group.sample_size(10);
    for &i in &[5_000, 100_000, 1_000_000, 5_000_000,500_000_000, 1_000_000_000, 2_000_000_000] {
        for &part in &[0.1, 0.5, 0.9] {
            let mut a = vec![0; i];
            thread_rng().fill(&mut a[..]);
            let point = (part * i as f64) as usize;
            let mut b = a.clone();
            b[point] = !b[point];
            let desc = format!("{} @ {}", i, point);
            group.bench_with_input(
                BenchmarkId::new("simple", &desc),
                &(&a[..], &b[..]),
                |b, i| b.iter(|| min_memcmp(i.0, i.1)),
            );
            group.bench_with_input(
                BenchmarkId::new("rayon", &desc),
                &(&a[..], &b[..]),
                |b, i| b.iter(|| rayon_memcmp(i.0, i.1)),
            );
            rayon::scope_fifo(|s| {
                group.bench_with_input(
                    BenchmarkId::new("rayon_scope", &desc),
                    &(&a[..], &b[..], s),
                    |b, i| b.iter(|| scope_memcmp(i.0, i.1, i.2)),
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_fibs);
criterion_main!(benches);
