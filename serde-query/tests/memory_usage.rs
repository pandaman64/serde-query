#![cfg(not(target_env = "msvc"))]

use jemalloc_ctl::stats;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(serde_query::Deserialize)]
struct Data {
    #[allow(dead_code)]
    #[query(".name")]
    name: String,
    #[allow(dead_code)]
    #[query(".options.num")]
    value: u32,
}

#[test]
fn test_memory_usage() {
    let epoch = jemalloc_ctl::epoch::mib().unwrap();
    let allocated = stats::allocated::mib().unwrap();
    let resident = stats::resident::mib().unwrap();
    let retained = stats::retained::mib().unwrap();

    epoch.advance().unwrap();
    let initial_allocated = allocated.read().unwrap();
    let initial_resident = resident.read().unwrap();
    let initial_retained = retained.read().unwrap();

    let data = serde_json::json!({
        "name": "hoge",
        "options": {
            "a": true,
            "num": 1000,
        },
    });
    epoch.advance().unwrap();
    let before_allocated = allocated.read().unwrap();
    let before_resident = resident.read().unwrap();
    let before_retained = retained.read().unwrap();

    let _data: Data = serde_json::from_value(data).unwrap();
    epoch.advance().unwrap();
    let after_allocated = allocated.read().unwrap();
    let after_resident = resident.read().unwrap();
    let after_retained = retained.read().unwrap();

    println!(
        "initial alloc {}/resident {}/retained {}",
        initial_allocated, initial_resident, initial_retained
    );
    println!(
        "before alloc {}/resident {}/retained {}",
        before_allocated, before_resident, before_retained
    );
    println!(
        "after alloc {}/resident {}/retained {}",
        after_allocated, after_resident, after_retained
    );
}
