//! Multi-producer, multi-consumer task channel.
//!
//! # Overview
//!
//! [`Channel`] wraps an unbounded **crossbeam** channel that carries
//! boxed [`TaskHandler`] trait objects.  Because both the sender and the
//! receiver halves are cloneable, any number of producers or consumers
//! can share the same channel from different threads.
//!
//! # TaskHandler trait
//!
//! A [`TaskHandler`] is any `Send` value that knows how to mutate a
//! [`Resources`] container when executed.  Task handlers are always
//! consumed on execution (`self: Box<Self>`), so they run exactly once.
//!
//! # IntoTask trait
//!
//! [`IntoTask`] converts any value — whether a named [`TaskHandler`] impl
//! or a closure — into a boxed [`TaskHandler`].  This is the trait
//! consumed by [`Channel::send`]; callers never need `Box::new`
//! themselves.
//!
//! # Resource integration
//!
//! `Channel` implements [`Resource`], so it can be registered in
//! [`Resources`] and accessed by tasks or startup systems via the
//! standard `resources.get::<Channel>()` API.

use crossbeam_channel::{Receiver, Sender};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use suon_macros::Resource;
use suon_resource::Resources;
use tracing::{error, warn};

/// Unit of asynchronous work.
///
/// Implementations receive a mutable reference to the [`Resources`]
/// container, allowing them to read, write, or remove any resource
/// registered with the application.  Values are always consumed on
/// execution (`self: Box<Self>`), so they run exactly once.
pub trait TaskHandler: Send {
    /// Execute this task handler against the given resource store.
    fn run(self: Box<Self>, resources: &mut Resources);
}

/// Converts a type into a boxed [`TaskHandler`].
///
/// Blanket implementations are provided for:
/// * Closures `FnOnce(&mut Resources) + Send + 'static` (wraps in
///   [`ClosureTask`]).
///
/// Named structs use the derive macro `#[derive(Task)]` from
/// `suon_macros` to generate this impl automatically.
pub trait IntoTask: Send + 'static {
    /// The concrete [`TaskHandler`] type produced by this conversion.
    type Task: TaskHandler;

    /// Convert this value into a boxed, runnable task handler.
    fn into_task(self) -> Self::Task;
}

/// Wraps a `FnOnce(&mut Resources)` closure as a [`TaskHandler`].
///
/// Created automatically when a closure is passed to [`Channel::send`].
pub struct ClosureTask<F>(pub F);

impl<F: FnOnce(&mut Resources) + Send + 'static> TaskHandler for ClosureTask<F> {
    fn run(self: Box<Self>, resources: &mut Resources) {
        let ClosureTask(f) = *self;
        f(resources);
    }
}

/// Closures `FnOnce(&mut Resources)` become task handlers automatically.
impl<F: FnOnce(&mut Resources) + Send + 'static> IntoTask for F {
    type Task = ClosureTask<F>;

    fn into_task(self) -> ClosureTask<F> {
        ClosureTask(self)
    }
}

/// Cloneable MPMC channel for dispatching [`TaskHandler`]s.
///
/// Both halves (sender and receiver) are part of the same struct,
/// making it trivial to share a channel across threads — simply
/// call [`Clone::clone()`] and move the copy into the other thread.
#[derive(Clone, Resource)]
pub struct Channel {
    sender: Sender<Box<dyn TaskHandler>>,
    receiver: Receiver<Box<dyn TaskHandler>>,
    pending: Arc<AtomicUsize>,
}

impl Default for Channel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Channel {
            sender,
            receiver,
            pending: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Channel {
    /// Enqueue a task handler for later execution.
    ///
    /// Accepts any [`IntoTask`] — a closure
    /// `\|resources: &mut Resources\| { … }`, or a named struct with
    /// `#[derive(Task)]`.
    pub fn send(&self, task: impl IntoTask) {
        if self.sender.send(Box::new(task.into_task())).is_err() {
            error!(target: "Channel", "send failed: receiver disconnected, dropping task");
            return;
        }
        self.pending.fetch_add(1, Ordering::Release);
    }

    /// Returns the approximate number of tasks currently enqueued.
    ///
    /// The count is indicative: senders may increment concurrently, and
    /// the value is a point-in-time snapshot.
    pub fn pending_count(&self) -> usize {
        self.pending.load(Ordering::Relaxed)
    }

    /// Block until at least one task is available, then drain all pending
    /// tasks without blocking.
    ///
    /// The thread sleeps while the queue is empty, consuming zero CPU.
    /// Once a task arrives it is pushed into `buffer`, followed by any
    /// additional tasks that accumulated during the wait.
    ///
    /// `buffer` is pre-sized according to [`pending_count`] to minimise
    /// reallocations.
    pub fn wait_and_drain(&self, buffer: &mut Vec<Box<dyn TaskHandler>>) {
        let estimated = self.pending.load(Ordering::Relaxed);
        if estimated > buffer.capacity() {
            buffer.reserve(estimated - buffer.len());
        }

        if let Ok(msg) = self.receiver.try_recv() {
            self.pending.fetch_sub(1, Ordering::Release);
            buffer.push(msg);
        } else if let Ok(msg) = self.receiver.recv() {
            self.pending.fetch_sub(1, Ordering::Release);
            buffer.push(msg);
        } else {
            warn!(target: "Channel", "Channel receiver disconnected");
            return;
        }

        while let Ok(msg) = self.receiver.try_recv() {
            self.pending.fetch_sub(1, Ordering::Release);
            buffer.push(msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Resource)]
    struct Num(i32);

    struct AddOne;
    impl TaskHandler for AddOne {
        fn run(self: Box<Self>, resources: &mut Resources) {
            resources.get_mut::<Num>().0 += 1;
        }
    }
    impl IntoTask for AddOne {
        type Task = AddOne;
        fn into_task(self) -> AddOne {
            self
        }
    }

    #[test]
    fn send_and_receive() {
        let channel = Channel::default();
        channel.send(AddOne);
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn cloned_channel_shares_queue() {
        let first = Channel::default();
        let second = first.clone();
        first.send(AddOne);
        second.send(AddOne);
        let mut buffer = Vec::new();
        first.wait_and_drain(&mut buffer);
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn task_executes() {
        let mut resources = Resources::default();
        resources.insert(Num(0));

        let task = Box::new(AddOne);
        task.run(&mut resources);
        assert_eq!(resources.get::<Num>().0, 1);
    }

    #[test]
    fn closure_task() {
        let mut resources = Resources::default();
        resources.insert(Num(0));

        let task = Box::new(ClosureTask(|r: &mut Resources| {
            r.get_mut::<Num>().0 += 1;
        }));
        task.run(&mut resources);
        assert_eq!(resources.get::<Num>().0, 1);
    }

    #[test]
    fn send_closure() {
        let channel = Channel::default();
        channel.send(|r: &mut Resources| {
            r.get_mut::<Num>().0 += 1;
        });
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn many_tasks() {
        let channel = Channel::default();
        for _ in 0..1000 {
            channel.send(AddOne);
        }
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        assert_eq!(buffer.len(), 1000);
    }

    #[test]
    fn concurrent_senders() {
        let channel = Channel::default();
        let mut thread_handles = Vec::new();
        for _ in 0..10 {
            let clone = channel.clone();
            thread_handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    clone.send(AddOne);
                }
            }));
        }
        for handle in thread_handles {
            handle
                .join()
                .expect("test thread should complete successfully");
        }
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        assert_eq!(buffer.len(), 1000);
    }

    #[test]
    fn send_after_partial_drain() {
        let channel = Channel::default();
        channel.send(AddOne);
        channel.send(AddOne);
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn drain_after_drop_returns_remaining() {
        let channel = Channel::default();
        let other = channel.clone();
        channel.send(AddOne);
        drop(other);
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn pending_count_starts_at_zero() {
        let channel = Channel::default();
        assert_eq!(channel.pending_count(), 0);
    }

    #[test]
    fn pending_count_reflects_sends() {
        let channel = Channel::default();
        channel.send(AddOne);
        assert_eq!(channel.pending_count(), 1);
        channel.send(AddOne);
        assert_eq!(channel.pending_count(), 2);
    }

    #[test]
    fn pending_count_decrements_on_drain() {
        let channel = Channel::default();
        channel.send(AddOne);
        channel.send(AddOne);
        assert_eq!(channel.pending_count(), 2);
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        assert_eq!(channel.pending_count(), 0);
    }

    #[test]
    fn pending_count_shared_across_clones() {
        let a = Channel::default();
        let b = a.clone();
        a.send(AddOne);
        b.send(AddOne);
        assert_eq!(a.pending_count(), 2);
        assert_eq!(b.pending_count(), 2);
    }

    #[test]
    fn task_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AddOne>();
    }

    #[test]
    fn channel_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Channel>();
    }

    #[test]
    fn channel_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Channel>();
    }
}
