use tokio::sync::{mpsc, oneshot};

use crate::{api, common::container::Container};

use super::{console, exit_status::ExitStatus};

pub(crate) type EventTx = mpsc::Sender<Event>;

#[derive(Debug)]
pub(crate) enum Event {
    /// Incoming console command
    Console(console::Request, oneshot::Sender<api::model::Response>),
    /// Northstar shall shut down
    Shutdown,
    /// Container event
    Container(Container, ContainerEvent),
}

#[derive(Clone, Debug)]
pub(crate) enum ContainerEvent {
    /// Container has been started
    Started,
    /// Container exited with status
    Exit(ExitStatus),
    /// Container is installed
    Installed,
    /// Container is uninstalled
    Uninstalled,
    /// CGroup event
    CGroup(CGroupEvent),
}

/// Events generated by cgroup controllers
#[derive(Clone, Debug)]
pub(crate) enum CGroupEvent {
    Memory(MemoryEvent),
}

#[derive(Clone, Default, Debug)]
pub(crate) struct MemoryEvent {
    /// The number of times the cgroup is reclaimed due to
    /// high memory pressure even though its usage is under
    /// the low boundary.  This usually indicates that the low
    /// boundary is over-committed.
    pub low: Option<u64>,
    /// The number of times processes of the cgroup are
    /// throttled and routed to perform direct memory reclaim
    /// because the high memory boundary was exceeded.  For a
    /// cgroup whose memory usage is capped by the high limit
    /// rather than global memory pressure, this event's
    /// occurrences are expected.
    pub high: Option<u64>,
    /// The number of times the cgroup's memory usage was
    /// about to go over the max boundary.  If direct reclaim
    /// fails to bring it down, the cgroup goes to OOM state.
    pub max: Option<u64>,
    /// The number of time the cgroup's memory usage was
    /// reached the limit and allocation was about to fail.
    /// Depending on context result could be invocation of OOM
    /// killer and retrying allocation or failing allocation.
    /// Failed allocation in its turn could be returned into
    /// userspace as -ENOMEM or silently ignored in cases like
    /// disk readahead. For now OOM in memory cgroup kills
    /// tasks if shortage has happened inside page fault.
    pub oom: Option<u64>,
    /// The number of processes belonging to this cgroup
    /// killed by any kind of OOM killer.
    pub oom_kill: Option<u64>,
}