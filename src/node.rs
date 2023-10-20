use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
fn get_unique_id() -> u64 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Node {
    id: u64,
    voltage: f64,
}

#[allow(dead_code)]
impl Node {
    pub fn new() -> Self {
        Node {
            id: get_unique_id(),
            ..Default::default()
        }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn node_creation() {
        let result = Node::new();
        assert!(result.id != 0);
    }
}
