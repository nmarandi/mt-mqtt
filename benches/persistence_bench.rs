use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mt_mqtt::persistence::{InMemoryBackend, PersistenceBackend, PersistedSession, PersistedMessage};
use std::collections::HashSet;

#[cfg(feature = "sqlite")]
use mt_mqtt::persistence::sqlite::SqliteBackend;

fn bench_in_memory_backend(c: &mut Criterion) {
    let mut group = c.benchmark_group("persistence/in_memory");
    
    group.bench_function("save_session", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let backend = InMemoryBackend;
                let mut subscriptions = HashSet::new();
                subscriptions.insert("topic/1".to_string());
                
                let session = PersistedSession {
                    client_id: "test_client".to_string(),
                    persistent: true,
                    subscriptions,
                };
                
                backend.save_session(black_box(&session)).await.unwrap();
            });
        });
    });
    
    group.bench_function("load_session", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let backend = InMemoryBackend;
                let _ = backend.load_session(black_box("test_client")).await;
            });
        });
    });
    
    group.bench_function("queue_message", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let backend = InMemoryBackend;
                let message = PersistedMessage {
                    client_id: "test_client".to_string(),
                    topic: "topic/1".to_string(),
                    payload: vec![1, 2, 3, 4, 5],
                    qos: 1,
                    retain: false,
                };
                
                backend.queue_message(black_box(&message)).await.unwrap();
            });
        });
    });
    
    group.finish();
}

#[cfg(feature = "sqlite")]
fn bench_sqlite_backend(c: &mut Criterion) {
    let mut group = c.benchmark_group("persistence/sqlite");
    
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    // Use in-memory SQLite for benchmarking
    let mut backend = SqliteBackend::new(":memory:");
    runtime.block_on(async {
        backend.init().await.unwrap();
    });
    
    group.bench_function("save_session", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut backend = SqliteBackend::new(":memory:");
        runtime.block_on(async {
            backend.init().await.unwrap();
        });
        
        b.iter(|| {
            runtime.block_on(async {
                let mut subscriptions = HashSet::new();
                subscriptions.insert("topic/1".to_string());
                
                let session = PersistedSession {
                    client_id: format!("client_{}", rand::random::<u32>()),
                    persistent: true,
                    subscriptions,
                };
                
                backend.save_session(black_box(&session)).await.unwrap();
            });
        });
    });
    
    group.bench_function("load_session", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut backend = SqliteBackend::new(":memory:");
        runtime.block_on(async {
            backend.init().await.unwrap();
            
            // Pre-populate a session
            let mut subscriptions = HashSet::new();
            subscriptions.insert("topic/1".to_string());
            let session = PersistedSession {
                client_id: "benchmark_client".to_string(),
                persistent: true,
                subscriptions,
            };
            backend.save_session(&session).await.unwrap();
        });
        
        b.iter(|| {
            runtime.block_on(async {
                let _ = backend.load_session(black_box("benchmark_client")).await;
            });
        });
    });
    
    group.bench_function("queue_message", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut backend = SqliteBackend::new(":memory:");
        runtime.block_on(async {
            backend.init().await.unwrap();
        });
        
        b.iter(|| {
            runtime.block_on(async {
                let message = PersistedMessage {
                    client_id: "test_client".to_string(),
                    topic: "topic/1".to_string(),
                    payload: vec![1, 2, 3, 4, 5],
                    qos: 1,
                    retain: false,
                };
                
                backend.queue_message(black_box(&message)).await.unwrap();
            });
        });
    });
    
    group.finish();
}

#[cfg(not(feature = "sqlite"))]
#[allow(dead_code)]
fn bench_sqlite_backend(_c: &mut Criterion) {
    // SQLite benchmarks skipped - compile with --features sqlite to enable
}

#[cfg(feature = "sqlite")]
criterion_group!(
    benches,
    bench_in_memory_backend,
    bench_sqlite_backend
);

#[cfg(not(feature = "sqlite"))]
criterion_group!(
    benches,
    bench_in_memory_backend
);

criterion_main!(benches);
