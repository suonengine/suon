use std::sync::Arc;

use suon_channel::BufferPool;
use suon_macros::Resource;

#[cfg(test)]
pub(crate) fn test_buffer_pool() -> Arc<BufferPool> {
    Arc::new(BufferPool::new(4096, 8))
}

/// Resource wrapper allowing [`Arc<BufferPool>`] to be stored in the
/// ECS [`Resources`] container.
///
/// Inserted by [`NetworkPlugin`](crate::NetworkPlugin) so that task
/// handlers (e.g. [`RawPacket`](crate::server::tcp::RawPacket)) can
/// release heap-allocated buffers back to the pool after processing.
///
/// [`Resources`]: suon_resource::Resources
#[derive(Resource, Clone)]
pub struct NetworkBufferPool(pub Arc<BufferPool>);
