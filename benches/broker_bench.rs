use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mt_mqtt::topic::TopicTree;

fn bench_topic_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("topic_matching");
    
    // Benchmark exact match
    group.bench_function("exact_match", |b| {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/temperature", "client1");
        
        b.iter(|| {
            let subs = tree.get_subscribers(black_box("sensor/temperature"));
            black_box(subs);
        });
    });
    
    // Benchmark single-level wildcard
    group.bench_function("single_level_wildcard", |b| {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/+/data", "client1");
        
        b.iter(|| {
            let subs = tree.get_subscribers(black_box("sensor/temp/data"));
            black_box(subs);
        });
    });
    
    // Benchmark multi-level wildcard
    group.bench_function("multi_level_wildcard", |b| {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/#", "client1");
        
        b.iter(|| {
            let subs = tree.get_subscribers(black_box("sensor/temp/room1/data"));
            black_box(subs);
        });
    });
    
    // Benchmark with many subscribers
    group.bench_function("many_subscribers", |b| {
        let mut tree = TopicTree::new_root();
        for i in 0..1000 {
            tree.subscribe(&format!("sensor/device{}", i), &format!("client{}", i));
        }
        
        b.iter(|| {
            let subs = tree.get_subscribers(black_box("sensor/device500"));
            black_box(subs);
        });
    });
    
    group.finish();
}

fn bench_subscription_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("subscription_ops");
    
    group.bench_function("subscribe", |b| {
        b.iter(|| {
            let mut tree = TopicTree::new_root();
            tree.subscribe(black_box("test/topic"), black_box("client1"));
        });
    });
    
    group.bench_function("unsubscribe", |b| {
        let mut tree = TopicTree::new_root();
        tree.subscribe("test/topic", "client1");
        
        b.iter(|| {
            tree.unsubscribe(black_box("test/topic"), black_box("client1"));
            tree.subscribe("test/topic", "client1"); // Re-subscribe for next iteration
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_topic_matching, bench_subscription_operations);
criterion_main!(benches);
