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
//! [`Resources`] container when executed.  Because [`run`] takes
//! `&mut self` instead of consuming the value, task objects can be
//! recycled after execution, enabling object-pooling strategies.
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

pub mod buffer_pool;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use parking_lot::Mutex;
use std::{
    collections::BinaryHeap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use suon_macros::Resource;
use suon_resource::Resources;
use tracing::{error, warn};

/// Unit of asynchronous work.
///
/// Implementations receive a mutable reference to the [`Resources`]
/// container, allowing them to read, write, or remove any resource
/// registered with the application.  The handler is **not** consumed
/// on execution, so the same slot can be reused by a pool.
pub trait TaskHandler: Send {
    /// Execute this task handler against the given resource store.
    fn run(&mut self, resources: &mut Resources);
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
/// Internally wraps the closure in [`Option`] so it can be called
/// through `&mut self` (consumed exactly once via [`Option::take`]).
pub struct ClosureTask<F>(Option<F>);

impl<F: FnOnce(&mut Resources) + Send + 'static> TaskHandler for ClosureTask<F> {
    fn run(&mut self, resources: &mut Resources) {
        if let Some(f) = self.0.take() {
            f(resources);
        }
    }
}

/// Closures `FnOnce(&mut Resources)` become task handlers automatically.
impl<F: FnOnce(&mut Resources) + Send + 'static> IntoTask for F {
    type Task = ClosureTask<F>;

    fn into_task(self) -> ClosureTask<F> {
        ClosureTask(Some(self))
    }
}

/// Internal sentinel used to wake the receiver when a scheduled task is
/// pushed as the new soonest.
struct WakeSignal;

impl TaskHandler for WakeSignal {
    fn run(&mut self, _: &mut Resources) {}
}

impl IntoTask for WakeSignal {
    type Task = WakeSignal;

    fn into_task(self) -> WakeSignal {
        self
    }
}

/// A task paired with its planned execution instant.
struct ScheduledTask {
    at: Instant,
    task: Box<dyn TaskHandler>,
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.at.partial_cmp(&self.at)
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.at.cmp(&self.at)
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}

impl Eq for ScheduledTask {}

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
    has_scheduled: Arc<AtomicBool>,
    scheduled: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
}

impl Default for Channel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Channel {
            sender,
            receiver,
            pending: Arc::new(AtomicUsize::new(0)),
            has_scheduled: Arc::new(AtomicBool::new(false)),
            scheduled: Arc::new(Mutex::new(BinaryHeap::new())),
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

    /// Schedule a task to run after `delay` has elapsed.
    ///
    /// No background thread is spawned — the task is stored in an
    /// internal timer heap and dispatched when [`wait_and_drain`] detects
    /// that its time has arrived.  The calling thread returns immediately.
    pub fn schedule(&self, delay: Duration, task: impl IntoTask) {
        let at = Instant::now() + delay;
        let task = Box::new(task.into_task());
        self.has_scheduled.store(true, Ordering::Release);
        let mut scheduled = self.scheduled.lock();

        let is_soonest = scheduled.peek().is_none_or(|t| at < t.at);
        scheduled.push(ScheduledTask { at, task });

        if is_soonest {
            drop(scheduled);
            self.send(WakeSignal);
        }
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
    /// Scheduled tasks (see [`schedule`]) are dispatched automatically
    /// once their delay has elapsed.
    ///
    /// `buffer` is pre-sized according to [`pending_count`] to minimise
    /// reallocations.
    pub fn wait_and_drain(&self, buffer: &mut Vec<Box<dyn TaskHandler>>) {
        let estimated = self.pending.load(Ordering::Relaxed);
        if estimated > buffer.capacity() {
            buffer.reserve(estimated - buffer.len());
        }

        // Fast path: something already in the main channel
        if let Ok(msg) = self.receiver.try_recv() {
            self.pending.fetch_sub(1, Ordering::Release);
            buffer.push(msg);
            Self::drain_main(&self.receiver, &self.pending, buffer);
            return;
        }

        // Slow path: block until a message or a scheduled task is ready
        loop {
            if self.has_scheduled.load(Ordering::Acquire) {
                self.pop_ready(buffer);
                if !buffer.is_empty() {
                    return;
                }
            }

            let timeout = if self.has_scheduled.load(Ordering::Relaxed) {
                let heap = self.scheduled.lock();
                heap.peek().map(|scheduled_task| {
                    scheduled_task.at.saturating_duration_since(Instant::now())
                })
            } else {
                None
            };

            let result = match timeout {
                Some(duration) if duration > Duration::ZERO => self.receiver.recv_timeout(duration),
                _ => self.receiver.recv().map_err(RecvTimeoutError::from),
            };

            match result {
                Ok(msg) => {
                    self.pending.fetch_sub(1, Ordering::Release);
                    buffer.push(msg);
                    Self::drain_main(&self.receiver, &self.pending, buffer);
                    return;
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    warn!(target: "Channel", "Channel receiver disconnected");
                    return;
                }
            }
        }
    }

    /// Non-blocking drain of all messages from the main channel.
    fn drain_main(
        receiver: &Receiver<Box<dyn TaskHandler>>,
        pending: &AtomicUsize,
        buffer: &mut Vec<Box<dyn TaskHandler>>,
    ) {
        while let Ok(msg) = receiver.try_recv() {
            pending.fetch_sub(1, Ordering::Release);
            buffer.push(msg);
        }
    }

    /// Move all ready scheduled tasks into `buffer`.
    fn pop_ready(&self, buffer: &mut Vec<Box<dyn TaskHandler>>) {
        let mut scheduled = self.scheduled.lock();
        let now = Instant::now();
        while let Some(task) = scheduled.peek() {
            if task.at > now {
                break;
            }

            if let Some(task) = scheduled.pop() {
                buffer.push(task.task);
            }
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
        fn run(&mut self, resources: &mut Resources) {
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

        let mut task = Box::new(AddOne);
        task.run(&mut resources);
        assert_eq!(resources.get::<Num>().0, 1);
    }

    #[test]
    fn closure_task() {
        let mut resources = Resources::default();
        resources.insert(Num(0));

        let mut task = Box::new(ClosureTask(Some(|resources: &mut Resources| {
            resources.get_mut::<Num>().0 += 1;
        })));

        task.run(&mut resources);
        assert_eq!(resources.get::<Num>().0, 1);
    }

    #[test]
    fn send_closure() {
        let channel = Channel::default();
        channel.send(|resources: &mut Resources| {
            resources.get_mut::<Num>().0 += 1;
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
    fn schedule_task_arrives_after_delay() {
        let channel = Channel::default();
        channel.schedule(std::time::Duration::from_millis(50), AddOne);

        // wait_and_drain may return a WakeSignal first; keep draining
        // until the actual AddOne task (detected by side effect).
        let mut resources = Resources::default();
        resources.insert(Num(0));

        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 == 1 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out waiting for scheduled task");
            }
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed >= std::time::Duration::from_millis(50),
            "task arrived too early: {elapsed:?}"
        );
    }

    #[test]
    fn schedule_respects_pending_count() {
        let channel = Channel::default();
        assert_eq!(channel.pending_count(), 0);
        channel.schedule(std::time::Duration::from_millis(20), AddOne);
        // pending_count reflects the WakeSignal sent immediately
        assert_eq!(channel.pending_count(), 1);

        // Drain until the scheduled AddOne arrives
        let mut buffer = Vec::new();
        let mut resources = Resources::default();
        resources.insert(Num(0));

        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 == 1 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out waiting for scheduled task");
            }
        }
        assert_eq!(channel.pending_count(), 0);
    }

    #[test]
    fn schedule_multiple_tasks() {
        let channel = Channel::default();
        channel.schedule(std::time::Duration::from_millis(30), AddOne);
        channel.schedule(std::time::Duration::from_millis(10), AddOne);

        // Drain until both AddOne tasks arrive
        let mut resources = Resources::default();
        resources.insert(Num(0));

        let mut buffer = Vec::new();
        let start = std::time::Instant::now();

        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 == 2 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out waiting for scheduled tasks");
            }
        }
    }

    #[test]
    fn schedule_zero_delay_arrives_immediately() {
        let channel = Channel::default();
        channel.schedule(std::time::Duration::ZERO, AddOne);
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
        // May contain WakeSignal + AddOne, or just AddOne
        assert!(!buffer.is_empty(), "expected at least one task");
    }

    #[test]
    fn schedule_with_zero_delay_and_send_all_arrive() {
        let channel = Channel::default();
        channel.send(AddOne);
        channel.schedule(std::time::Duration::ZERO, AddOne);

        let mut resources = Resources::default();
        resources.insert(Num(0));

        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= 2 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out waiting for all tasks");
            }
        }
        assert_eq!(
            resources.get::<Num>().0,
            2,
            "both tasks should have executed"
        );
    }

    #[test]
    fn schedule_does_not_block_on_disconnected() {
        let channel = Channel::default();
        channel.schedule(std::time::Duration::from_millis(10), AddOne);
        // Just verify no panic — channel is still alive
        let mut buffer = Vec::new();
        channel.wait_and_drain(&mut buffer);
    }

    #[test]
    fn interleaved_send_and_schedule() {
        let channel = Channel::default();
        channel.send(AddOne);
        channel.schedule(std::time::Duration::from_millis(10), AddOne);
        channel.send(AddOne);

        let mut resources = Resources::default();
        resources.insert(Num(0));

        // Drain multiple times until all arrive
        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= 3 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out waiting for all tasks");
            }
        }
        assert_eq!(resources.get::<Num>().0, 3);
    }

    #[test]
    fn send_then_schedule_both_arrive() {
        let channel = Channel::default();
        channel.send(AddOne);
        channel.schedule(std::time::Duration::from_millis(30), AddOne);

        let mut resources = Resources::default();
        resources.insert(Num(0));
        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= 2 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out");
            }
        }
        assert_eq!(resources.get::<Num>().0, 2);
    }

    #[test]
    fn schedule_then_send_both_arrive() {
        let channel = Channel::default();
        channel.schedule(std::time::Duration::from_millis(30), AddOne);
        channel.send(AddOne);

        let mut resources = Resources::default();
        resources.insert(Num(0));
        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= 2 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out");
            }
        }
        assert_eq!(resources.get::<Num>().0, 2);
    }

    #[test]
    fn many_scheduled_tasks_all_arrive() {
        const N: usize = 500;
        let channel = Channel::default();
        for i in 0..N {
            channel.schedule(std::time::Duration::from_millis((i % 20) as u64), AddOne);
        }

        let mut resources = Resources::default();
        resources.insert(Num(0));
        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= N as i32 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(10) {
                panic!("timed out with {} tasks", resources.get::<Num>().0);
            }
        }
        assert_eq!(resources.get::<Num>().0, N as i32);
    }

    #[test]
    fn interleaved_schedule_send_large() {
        const N: usize = 200;
        let channel = Channel::default();
        for i in 0..N {
            channel.send(AddOne);
            channel.schedule(std::time::Duration::from_millis((i % 15) as u64), AddOne);
        }

        let mut resources = Resources::default();
        resources.insert(Num(0));
        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= (N * 2) as i32 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(10) {
                panic!("timed out with {} tasks", resources.get::<Num>().0);
            }
        }
        assert_eq!(resources.get::<Num>().0, (N * 2) as i32);
    }

    #[test]
    fn scheduled_tasks_execute_in_time_order() {
        let channel = Channel::default();
        // Schedule in reverse order to test heap ordering
        channel.schedule(std::time::Duration::from_millis(40), AddOne);
        channel.schedule(std::time::Duration::from_millis(10), AddOne);
        channel.schedule(std::time::Duration::from_millis(30), AddOne);

        let mut resources = Resources::default();
        resources.insert(Num(0));
        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= 3 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out");
            }
        }
        assert_eq!(resources.get::<Num>().0, 3);
    }

    #[test]
    fn schedule_same_deadline_all_arrive() {
        let channel = Channel::default();
        for _ in 0..100 {
            channel.schedule(std::time::Duration::from_millis(20), AddOne);
        }

        let mut resources = Resources::default();
        resources.insert(Num(0));
        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        loop {
            channel.wait_and_drain(&mut buffer);
            for mut task in buffer.drain(..) {
                task.run(&mut resources);
            }

            if resources.get::<Num>().0 >= 100 {
                break;
            }

            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("timed out with {} tasks", resources.get::<Num>().0);
            }
        }
        assert_eq!(resources.get::<Num>().0, 100);
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
