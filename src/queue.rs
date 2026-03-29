use std::sync::Arc;
use std::sync::Mutex;
use crossbeam_channel::{bounded, Sender};

/// Execution mode for the queue
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionMode {
    /// Process items sequentially, one at a time
    Sequential,
    /// Process items in parallel
    Parallel,
}

impl ExecutionMode {
    /// Convert from integer: 0 = Sequential, 1 = Parallel
    pub fn from_int(val: u8) -> Self {
        match val {
            0 => ExecutionMode::Sequential,
            _ => ExecutionMode::Parallel,
        }
    }

    /// Convert to integer: 0 = Sequential, 1 = Parallel
    pub fn to_int(&self) -> u8 {
        match self {
            ExecutionMode::Sequential => 0,
            ExecutionMode::Parallel => 1,
        }
    }
}

/// Item in the queue
#[derive(Clone, Debug)]
pub struct QueueItem {
    pub id: u64,
    pub data: Vec<u8>,
}

/// Worker function type for processing queue items
pub type WorkerFn = Arc<dyn Fn(u64, Vec<u8>) + Send + Sync>;

/// Queue statistics
#[derive(Clone, Debug)]
pub struct QueueStats {
    pub total_pushed: u64,
    pub total_processed: u64,
    pub total_errors: u64,
    pub active_workers: usize,
}

/// Async Queue with parallel/sequential processing capabilities
pub struct AsyncQueue {
    sender: Arc<Mutex<Option<Sender<QueueItem>>>>,
    mode: Arc<Mutex<ExecutionMode>>,
    counter: Arc<Mutex<u64>>,
    active_workers: Arc<Mutex<usize>>,
    processed_count: Arc<Mutex<u64>>,
    error_count: Arc<Mutex<u64>>,
}

impl AsyncQueue {
    /// Create a new AsyncQueue
    ///
    /// # Arguments
    ///
    /// * `mode` - 0 for Sequential, 1 for Parallel
    /// * `buffer_size` - Channel buffer size
    ///
    /// # Example
    ///
    /// ```
    /// let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
    /// ```
    pub fn new(mode: u8, _buffer_size: usize) -> Result<Self, String> {
        let execution_mode = ExecutionMode::from_int(mode);

        Ok(AsyncQueue {
            sender: Arc::new(Mutex::new(None)),
            mode: Arc::new(Mutex::new(execution_mode)),
            counter: Arc::new(Mutex::new(0)),
            active_workers: Arc::new(Mutex::new(0)),
            processed_count: Arc::new(Mutex::new(0)),
            error_count: Arc::new(Mutex::new(0)),
        })
    }

    /// Push an item to the queue
    pub fn push(&self, data: Vec<u8>) -> Result<(), String> {
        let sender_arc = Arc::clone(&self.sender);
        let counter_arc = Arc::clone(&self.counter);

        let mut counter = counter_arc.lock().unwrap();
        *counter += 1;
        let id = *counter;
        drop(counter);

        if let Some(ref tx) = *sender_arc.lock().unwrap() {
            let item = QueueItem { id, data };
            tx.send(item)
                .map_err(|e| format!("Failed to send item: {}", e))?;
        }
        Ok(())
    }

    /// Get the current mode (0 for Sequential, 1 for Parallel)
    pub fn get_mode(&self) -> u8 {
        let mode = *self.mode.lock().unwrap();
        mode.to_int()
    }

    /// Set the execution mode
    pub fn set_mode(&self, mode: u8) -> Result<(), String> {
        *self.mode.lock().unwrap() = ExecutionMode::from_int(mode);
        Ok(())
    }

    /// Get number of active workers
    pub fn active_workers(&self) -> usize {
        *self.active_workers.lock().unwrap()
    }

    /// Get total items pushed to the queue
    pub fn total_pushed(&self) -> u64 {
        *self.counter.lock().unwrap()
    }

    /// Get total items processed
    pub fn total_processed(&self) -> u64 {
        *self.processed_count.lock().unwrap()
    }

    /// Get total errors during processing
    pub fn total_errors(&self) -> u64 {
        *self.error_count.lock().unwrap()
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> QueueStats {
        QueueStats {
            total_pushed: self.total_pushed(),
            total_processed: self.total_processed(),
            total_errors: self.total_errors(),
            active_workers: self.active_workers(),
        }
    }

    /// Start the queue with a worker function
    ///
    /// # Arguments
    ///
    /// * `worker` - A function that processes queue items
    /// * `num_workers` - Number of parallel workers (ignored in sequential mode)
    ///
    /// # Example
    ///
    /// ```
    /// let queue = AsyncQueue::new(1, 128).unwrap();
    /// let worker = Arc::new(|id: u64, data: Vec<u8>| {
    ///     println!("Processing item {}: {:?}", id, data);
    /// });
    /// queue.start(worker, 4).unwrap();
    /// ```
    pub fn start(&mut self, worker: WorkerFn, num_workers: usize) -> Result<(), String> {
        let (tx, rx) = bounded::<QueueItem>(128);

        let mode = *self.mode.lock().unwrap();

        // Store sender
        *self.sender.lock().unwrap() = Some(tx);

        let rx_arc = Arc::new(Mutex::new(Some(rx)));

        match mode {
            ExecutionMode::Sequential => {
                let rx_clone = Arc::clone(&rx_arc);
                let worker_clone = Arc::clone(&worker);
                let active_clone = Arc::clone(&self.active_workers);
                let processed_clone = Arc::clone(&self.processed_count);

                std::thread::spawn(move || {
                    if let Some(receiver) = rx_clone.lock().unwrap().take() {
                        for item in receiver {
                            *active_clone.lock().unwrap() += 1;
                            worker_clone(item.id, item.data);
                            *processed_clone.lock().unwrap() += 1;
                            *active_clone.lock().unwrap() -= 1;
                        }
                    }
                });
            }
            ExecutionMode::Parallel => {
                for _ in 0..num_workers {
                    let rx_clone = Arc::clone(&rx_arc);
                    let worker_clone = Arc::clone(&worker);
                    let active_clone = Arc::clone(&self.active_workers);
                    let processed_clone = Arc::clone(&self.processed_count);

                    std::thread::spawn(move || {
                        if let Some(receiver) = rx_clone.lock().unwrap().take() {
                            for item in receiver {
                                *active_clone.lock().unwrap() += 1;
                                worker_clone(item.id, item.data);
                                *processed_clone.lock().unwrap() += 1;
                                *active_clone.lock().unwrap() -= 1;
                            }
                        }
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_queue_creation() {
        let queue = AsyncQueue::new(0, 128);
        assert!(queue.is_ok());
    }

    #[test]
    fn test_push_item() {
        let queue = AsyncQueue::new(0, 128).unwrap();
        let result = queue.push("test".as_bytes().to_vec());
        assert!(result.is_ok());
        assert_eq!(queue.total_pushed(), 1);
    }

    #[test]
    fn test_get_mode() {
        let queue = AsyncQueue::new(0, 128).unwrap();
        assert_eq!(queue.get_mode(), 0);

        let queue_parallel = AsyncQueue::new(1, 128).unwrap();
        assert_eq!(queue_parallel.get_mode(), 1);
    }

    #[test]
    fn test_set_mode() {
        let queue = AsyncQueue::new(0, 128).unwrap();
        assert_eq!(queue.get_mode(), 0);
        queue.set_mode(1).unwrap();
        assert_eq!(queue.get_mode(), 1);
    }

    #[test]
    fn test_stats() {
        let queue = AsyncQueue::new(0, 128).unwrap();
        queue.push("test1".as_bytes().to_vec()).unwrap();
        queue.push("test2".as_bytes().to_vec()).unwrap();

        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 2);
        assert_eq!(stats.active_workers, 0);
    }
}
