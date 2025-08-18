use criterion::{black_box, criterion_group, criterion_main, Criterion};
use system_controller::protocol::messages::*;

fn benchmark_command_serialization(c: &mut Criterion) {
    let cmd = Command {
        id: "bench-001".to_string(),
        command_type: CommandType::MouseMove,
        payload: CommandPayload::MouseMove { x: 100, y: 200 },
        timestamp: "2025-08-18T10:30:00Z".to_string(),
    };

    c.bench_function("command_serialization", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&cmd)).unwrap();
            let _: Command = serde_json::from_str(black_box(&json)).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_command_serialization);
criterion_main!(benches);