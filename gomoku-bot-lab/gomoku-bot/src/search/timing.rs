use instant::Instant;
use std::time::Duration;

#[cfg(target_os = "linux")]
pub(super) fn thread_cpu_time() -> Option<Duration> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let ok = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) == 0 };
    if ok {
        Some(Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32))
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
pub(super) fn thread_cpu_time() -> Option<Duration> {
    None
}

#[derive(Clone, Copy)]
pub(super) struct SearchDeadline {
    wall_deadline: Option<Instant>,
    cpu_start: Option<Duration>,
    cpu_budget: Option<Duration>,
}

impl SearchDeadline {
    pub(super) fn new(
        wall_start: Instant,
        wall_budget: Option<Duration>,
        cpu_start: Option<Duration>,
        cpu_budget: Option<Duration>,
    ) -> Self {
        Self {
            wall_deadline: wall_budget.map(|budget| wall_start + budget),
            cpu_start,
            cpu_budget,
        }
    }

    pub(super) fn expired(self) -> bool {
        if self
            .wall_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return true;
        }

        if let (Some(start), Some(budget), Some(now)) =
            (self.cpu_start, self.cpu_budget, thread_cpu_time())
        {
            return now.saturating_sub(start) >= budget;
        }

        false
    }
}
