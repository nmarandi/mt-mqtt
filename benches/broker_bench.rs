use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use mt_mqtt::topic::TopicTree;
use mt_mqtt::protocol::{frame::{Frame, ControlPacket}, definitions::ControlPacketType, packet::*};

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

fn bench_frame_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_serialization");
    
    // Benchmark PUBLISH packet serialization
    group.bench_function("publish_qos0_small", |b| {
        b.iter(|| {
            let frame = create_publish_frame("test/topic", b"Hello", 0, false);
            let serialized = Frame::serialize(black_box(frame)).unwrap();
            black_box(serialized);
        });
    });
    
    group.bench_function("publish_qos1_small", |b| {
        b.iter(|| {
            let frame = create_publish_frame("test/topic", b"Hello", 1, false);
            let serialized = Frame::serialize(black_box(frame)).unwrap();
            black_box(serialized);
        });
    });
    
    // Benchmark with different payload sizes
    for size in [100, 1024, 10240].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("publish_qos0", size), size, |b, &size| {
            let payload = vec![0u8; size];
            b.iter(|| {
                let frame = create_publish_frame("test/topic", &payload, 0, false);
                let serialized = Frame::serialize(black_box(frame)).unwrap();
                black_box(serialized);
            });
        });
    }
    
    group.finish();
}

fn bench_frame_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_deserialization");
    
    // Create serialized frames for deserialization benchmarks
    let small_frame = create_publish_frame("test/topic", b"Hello", 0, false);
    let small_bytes = Frame::serialize(small_frame).unwrap();
    
    group.bench_function("publish_qos0_small", |b| {
        b.iter(|| {
            let mut cursor = std::io::Cursor::new(black_box(&small_bytes[..]));
            let frame = Frame::deserialize(&mut cursor).unwrap();
            black_box(frame);
        });
    });
    
    // Benchmark with different payload sizes
    for size in [100, 1024, 10240].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("publish_qos0", size), size, |b, &size| {
            let payload = vec![0u8; size];
            let frame = create_publish_frame("test/topic", &payload, 0, false);
            let bytes = Frame::serialize(frame).unwrap();
            
            b.iter(|| {
                let mut cursor = std::io::Cursor::new(black_box(&bytes[..]));
                let frame = Frame::deserialize(&mut cursor).unwrap();
                black_box(frame);
            });
        });
    }
    
    group.finish();
}

fn bench_message_routing(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_routing");
    
    // Benchmark finding subscribers for a message
    group.bench_function("route_to_single_subscriber", |b| {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/temperature", "client1");
        
        b.iter(|| {
            let subs = tree.get_subscribers(black_box("sensor/temperature"));
            black_box(subs);
        });
    });
    
    group.bench_function("route_to_multiple_subscribers", |b| {
        let mut tree = TopicTree::new_root();
        for i in 0..10 {
            tree.subscribe("sensor/temperature", &format!("client{}", i));
        }
        
        b.iter(|| {
            let subs = tree.get_subscribers(black_box("sensor/temperature"));
            black_box(subs);
        });
    });
    
    group.bench_function("route_with_wildcards", |b| {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/+/temperature", "client1");
        tree.subscribe("sensor/#", "client2");
        tree.subscribe("sensor/room1/+", "client3");
        
        b.iter(|| {
            let subs = tree.get_subscribers(black_box("sensor/room1/temperature"));
            black_box(subs);
        });
    });
    
    group.finish();
}

fn bench_topic_tree_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("topic_tree_scale");
    
    for topic_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("lookup_in_tree", topic_count),
            topic_count,
            |b, &count| {
                let mut tree = TopicTree::new_root();
                for i in 0..count {
                    tree.subscribe(&format!("sensor/device{}/data", i), &format!("client{}", i % 100));
                }
                
                b.iter(|| {
                    let subs = tree.get_subscribers(black_box("sensor/device500/data"));
                    black_box(subs);
                });
            },
        );
    }
    
    group.finish();
}

// Helper function to create PUBLISH frames
fn create_publish_frame(topic: &str, payload: &[u8], qos: u8, retain: bool) -> Frame {
    use mt_mqtt::protocol::definitions::{FixHeader, Flags};
    
    let mut publish_packet = PublishControlPacket {
        variable_header: PublishVariableHeader::new(),
        payload: PublishPayload {
            data: bytes::Bytes::from(payload.to_vec()),
        },
    };
    
    publish_packet.variable_header.topic_name = topic.to_string();
    if qos > 0 {
        publish_packet.variable_header.packet_identifier = Some(1);
    }
    
    Frame {
        fix_header: FixHeader::new(
            ControlPacketType::PUBLISH,
            Flags(if retain { 1 } else { 0 }, qos, 0, 0),
        ),
        control_packet: ControlPacket::Publish(publish_packet),
    }
}

criterion_group!(
    benches,
    bench_topic_matching,
    bench_subscription_operations,
    bench_frame_serialization,
    bench_frame_deserialization,
    bench_message_routing,
    bench_topic_tree_scale
);
criterion_main!(benches);
