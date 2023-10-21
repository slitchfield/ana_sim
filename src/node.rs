use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
fn get_unique_id() -> u64 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Node {
    pub id: u64,
    pub voltage: f64,
    pub updated: bool,
    pub is_gnd: bool,
}

#[allow(dead_code)]
impl Node {
    pub fn new() -> (u64, Self) {
        let ret = Node {
            id: get_unique_id(),
            ..Default::default()
        };
        (ret.id, ret)
    }

    pub fn make_ground(&mut self) {
        self.is_gnd = true;
    }
    pub fn get_state(&self) -> f64 {
        self.voltage
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn node_creation() {
        let (resid, _resnode) = Node::new();
        assert!(resid != 0);
    }
}
