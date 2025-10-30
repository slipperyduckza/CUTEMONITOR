use iced::advanced::subscription::{Hasher, Recipe};
use iced::futures::stream::{self, BoxStream};
use iced_futures::subscription::Event;

use crate::what_cpu_check;
use crate::user_process_fetch;

// Recipe for CPU threads monitoring subscription
pub struct CpuThreadsMonitor;

impl Recipe for CpuThreadsMonitor {
    type Output = crate::state::Message;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, Event>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold((), |()| async {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let thread_usages = what_cpu_check::get_thread_usages().await;
            Some((crate::state::Message::UpdateThreads(thread_usages), ()))
        });
        Box::pin(stream)
    }
}

/// A subscription recipe that monitors running processes and their CPU usage
/// This helps identify which applications are using the most CPU resources
pub struct ProcessesMonitor;

impl Recipe for ProcessesMonitor {
    type Output = crate::state::Message;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, Event>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold((), |()| async {
            // Update every 2000ms
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
            // Get the top user processes by CPU usage
            let top_processes = user_process_fetch::get_top_processes();
            let processes: Vec<what_cpu_check::ProcessInfo> = top_processes.into_iter().map(|(name, description, cpu_usage)| {
                what_cpu_check::ProcessInfo {
                    name,
                    description,
                    cpu_usage: cpu_usage as f32,
                }
            }).collect();
            Some((crate::state::Message::UpdateProcesses(processes), ()))
        });
        Box::pin(stream)
    }
}

/// A subscription recipe that monitors CPU core usage
/// This provides the most frequent updates since cores are the primary CPU metric
pub struct CpuCoresMonitor;

impl Recipe for CpuCoresMonitor {
    type Output = crate::state::Message;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, Event>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold((), |()| async {
            // Update every 300ms (fastest update rate for responsive UI)
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            // Get current usage for all CPU cores
            let core_usages = what_cpu_check::get_core_usages().await;
            Some((crate::state::Message::UpdateCores(core_usages), ()))
        });
        Box::pin(stream)
    }
}