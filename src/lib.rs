pub mod queue;
pub use queue::*;

use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use std::sync::Arc;

/// Python module exposing the high-performance queue
#[pymodule]
fn _rst_queue(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAsyncQueue>()?;
    m.add_class::<PyQueueStats>()?;
    Ok(())
}

/// Python wrapper for QueueStats
#[pyclass(module = "rst_queue._rst_queue")]
pub struct PyQueueStats {
    pub total_pushed: u64,
    pub total_processed: u64,
    pub total_errors: u64,
    pub active_workers: usize,
}

#[pymethods]
impl PyQueueStats {
    #[getter]
    fn total_pushed(&self) -> u64 {
        self.total_pushed
    }

    #[getter]
    fn total_processed(&self) -> u64 {
        self.total_processed
    }

    #[getter]
    fn total_errors(&self) -> u64 {
        self.total_errors
    }

    #[getter]
    fn active_workers(&self) -> usize {
        self.active_workers
    }

    fn __repr__(&self) -> String {
        format!(
            "QueueStats(total_pushed={}, total_processed={}, total_errors={}, active_workers={})",
            self.total_pushed, self.total_processed, self.total_errors, self.active_workers
        )
    }
}

/// Python wrapper for AsyncQueue
#[pyclass(module = "rst_queue._rst_queue")]
pub struct PyAsyncQueue {
    inner: AsyncQueue,
}

#[pymethods]
impl PyAsyncQueue {
    /// Create a new AsyncQueue
    ///
    /// Args:
    ///     mode: 0 for SEQUENTIAL, 1 for PARALLEL (default: 1)
    ///     buffer_size: Channel buffer size (default: 128)
    #[new]
    #[pyo3(signature = (mode = 1, buffer_size = 128))]
    fn new(mode: u8, buffer_size: usize) -> PyResult<Self> {
        let queue = AsyncQueue::new(mode, buffer_size)
            .map_err(|e| PyTypeError::new_err(e))?;
        Ok(PyAsyncQueue { inner: queue })
    }

    /// Push an item to the queue
    ///
    /// Args:
    ///     data: bytes to push to the queue
    fn push(&self, data: &[u8]) -> PyResult<()> {
        self.inner.push(data.to_vec())
            .map_err(|e| PyTypeError::new_err(e))
    }

    /// Get the current execution mode
    ///
    /// Returns:
    ///     0 for SEQUENTIAL, 1 for PARALLEL
    fn get_mode(&self) -> u8 {
        self.inner.get_mode()
    }

    /// Set the execution mode
    ///
    /// Args:
    ///     mode: 0 for SEQUENTIAL, 1 for PARALLEL
    fn set_mode(&self, mode: u8) -> PyResult<()> {
        self.inner.set_mode(mode)
            .map_err(|e| PyTypeError::new_err(e))
    }

    /// Get number of active workers
    fn active_workers(&self) -> usize {
        self.inner.active_workers()
    }

    /// Get total items pushed to the queue
    fn total_pushed(&self) -> u64 {
        self.inner.total_pushed()
    }

    /// Get total items processed
    fn total_processed(&self) -> u64 {
        self.inner.total_processed()
    }

    /// Get total errors during processing
    fn total_errors(&self) -> u64 {
        self.inner.total_errors()
    }

    /// Get queue statistics
    fn get_stats(&self) -> PyResult<PyQueueStats> {
        let stats = self.inner.get_stats();
        Ok(PyQueueStats {
            total_pushed: stats.total_pushed,
            total_processed: stats.total_processed,
            total_errors: stats.total_errors,
            active_workers: stats.active_workers,
        })
    }

    /// Start processing queue items
    ///
    /// Args:
    ///     worker: A callable that accepts (item_id: int, data: bytes)
    ///     num_workers: Number of parallel workers (default: 1)
    fn start(&mut self, worker: PyObject, num_workers: usize) -> PyResult<()> {
        // Create a wrapper function that calls the Python callable
        let worker = Arc::new(move |id: u64, data: Vec<u8>| {
            Python::with_gil(|py| {
                if let Err(e) = worker.call1(py, (id, data)) {
                    eprintln!("Error in worker: {}", e);
                }
            });
        });

        self.inner.start(worker, num_workers)
            .map_err(|e| PyTypeError::new_err(e))
    }
}
