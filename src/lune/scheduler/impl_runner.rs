use std::process::ExitCode;

use mlua::prelude::*;

use super::SchedulerImpl;

impl SchedulerImpl {
    /**
        Runs all lua threads to completion, gathering any results they produce.
    */
    fn run_threads(&self) -> Vec<LuaResult<()>> {
        let mut results = Vec::new();

        while let Some((thread, args)) = self
            .pop_thread()
            .expect("Failed to pop thread from scheduler")
        {
            let res = thread.resume(args);
            self.state.add_resumption();

            if let Err(e) = &res {
                self.state.add_error();
                eprintln!("{e}"); // TODO: Pretty print the lua error here
            }

            results.push(res);

            if self.state.has_exit_code() {
                break;
            }
        }

        results
    }

    /**
        Runs the scheduler to completion, both normal lua threads and futures.

        This will emit lua output and errors to stdout and stderr.
    */
    pub async fn run_to_completion(&self) -> ExitCode {
        loop {
            // 1. Run lua threads until exit or there are none left
            let results = self.run_threads();

            // 2. If we got a manual exit code from lua we should not continue
            if self.state.has_exit_code() {
                break;
            }

            // 3. Wait for the next future to complete, this may
            // add more lua threads to run in the next iteration

            // TODO: Implement this

            // 4. If did not resume any lua threads, and we have no futures
            // queued either, we have run the scheduler until completion
            if results.is_empty() {
                break;
            }
        }

        if let Some(code) = self.state.exit_code() {
            ExitCode::from(code)
        } else if self.state.has_errored() {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}
