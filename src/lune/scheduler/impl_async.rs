use futures_util::Future;

use super::SchedulerImpl;

impl SchedulerImpl {
    /**
        Schedules a plain future to run whenever the scheduler is available.
    */
    pub fn schedule_future<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.futures
            .try_lock()
            .expect("Failed to lock futures queue")
            .push(Box::pin(fut))
    }
}
