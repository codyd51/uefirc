use alloc::vec;
use alloc::vec::Vec;
use log::info;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TCPv4ConnectionOperation {
    Connecting,
    Transmitting,
}

#[derive(Debug)]
pub struct TCPv4ConnectionLifecycleManager {
    pending_operations: Vec<TCPv4ConnectionOperation>,
}

impl TCPv4ConnectionLifecycleManager {
    pub fn new() -> Self {
        Self {
            pending_operations: vec![],
        }
    }

    fn add_if_not_present(&mut self, op: TCPv4ConnectionOperation) {
        if !self.pending_operations.contains(&op) {
            self.pending_operations.push(op)
        }
    }

    fn remove_if_present(&mut self, op: TCPv4ConnectionOperation) {
        if self.pending_operations.contains(&op) {
            self.pending_operations.retain(|&x| x != op);
        }
    }

    pub fn register_started_connecting(&mut self) {
        self.add_if_not_present(TCPv4ConnectionOperation::Connecting);
    }

    pub fn register_connecting_complete(&mut self) {
        self.remove_if_present(TCPv4ConnectionOperation::Connecting);
        info!("Callback: connection completed!");
    }

    pub fn register_started_transmitting(&mut self) {
        self.add_if_not_present(TCPv4ConnectionOperation::Transmitting);
    }

    pub fn register_transmitting_complete(&mut self) {
        self.remove_if_present(TCPv4ConnectionOperation::Transmitting);
        info!("Callback: transmit completed!");
    }
}
