use parking_lot::Mutex;

/// A thread-safe pool of pre-sized [`Vec<u8>`] buffers.
///
/// Buffers are acquired in LIFO order (most recently released first)
/// to maximise CPU-cache locality.
pub struct BufferPool {
    pool: Mutex<Vec<Vec<u8>>>,
    buffer_size: usize,
}

impl BufferPool {
    /// Create a pool whose buffers start with `buffer_size` capacity.
    ///
    /// `prealloc_count` buffers are created eagerly so that the first
    /// N acquisitions are allocation-free.
    pub fn new(buffer_size: usize, prealloc_count: usize) -> Self {
        let mut pool = Vec::with_capacity(prealloc_count);
        for _ in 0..prealloc_count {
            pool.push(Vec::with_capacity(buffer_size));
        }

        BufferPool {
            pool: Mutex::new(pool),
            buffer_size,
        }
    }

    /// Take a buffer from the pool, or allocate a fresh one if empty.
    pub fn acquire(&self) -> Vec<u8> {
        let mut guard = self.pool.lock();
        guard
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }

    /// Return a buffer to the pool for reuse.
    pub fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        self.pool.lock().push(buf);
    }

    /// Upper bound on the number of buffers currently idle in the pool.
    pub fn idle_count(&self) -> usize {
        self.pool.lock().len()
    }

    /// The capacity that acquired buffers will have.
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}
