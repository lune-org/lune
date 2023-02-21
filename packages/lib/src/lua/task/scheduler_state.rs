use std::{fmt, process::ExitCode};

use mlua::prelude::*;

use super::scheduler::TaskScheduler;

/// Struct representing the current state of the task scheduler
#[derive(Debug, Clone)]
#[must_use = "Scheduler state must be checked after every resumption"]
pub struct TaskSchedulerState {
    pub(super) lua_error: Option<LuaError>,
    pub(super) exit_code: Option<ExitCode>,
    pub(super) num_blocking: usize,
    pub(super) num_futures: usize,
    pub(super) num_background: usize,
}

impl TaskSchedulerState {
    pub(super) fn new(sched: &TaskScheduler) -> Self {
        Self {
            lua_error: None,
            exit_code: sched.exit_code.get(),
            num_blocking: sched.tasks_count.get(),
            num_futures: sched.futures_count.get(),
            num_background: sched.futures_background_count.get(),
        }
    }

    pub(super) fn err(sched: &TaskScheduler, err: LuaError) -> Self {
        let mut this = Self::new(sched);
        this.lua_error = Some(err);
        this
    }

    /**
        Returns a clone of the error from
        this task scheduler result, if any.
    */
    pub fn get_lua_error(&self) -> Option<LuaError> {
        self.lua_error.clone()
    }

    /**
        Returns a clone of the exit code from
        this task scheduler result, if any.
    */
    pub fn get_exit_code(&self) -> Option<ExitCode> {
        self.exit_code
    }

    /**
        Returns `true` if the task scheduler still
        has blocking lua threads left to run.
    */
    pub fn is_blocking(&self) -> bool {
        self.num_blocking > 0
    }

    /**
        Returns `true` if the task scheduler has finished all
        blocking lua tasks, but still has yielding tasks running.
    */
    pub fn is_yielding(&self) -> bool {
        self.num_blocking == 0 && self.num_futures > 0
    }

    /**
        Returns `true` if the task scheduler has finished all
        lua threads, but still has background tasks running.
    */
    pub fn is_background(&self) -> bool {
        self.num_blocking == 0 && self.num_futures == 0 && self.num_background > 0
    }

    /**
        Returns `true` if the task scheduler is done,
        meaning it has no lua threads left to run, and
        no spawned tasks are running in the background.

        Also returns `true` if a task has requested to exit the process.
    */
    pub fn is_done(&self) -> bool {
        self.exit_code.is_some()
            || (self.num_blocking == 0 && self.num_futures == 0 && self.num_background == 0)
    }
}

impl fmt::Display for TaskSchedulerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_blocking() {
            "Busy"
        } else if self.is_yielding() {
            "Yielding"
        } else if self.is_background() {
            "Background"
        } else {
            "Done"
        };
        let code = match self.get_exit_code() {
            Some(code) => format!("{code:?}"),
            None => "-".to_string(),
        };
        let err = match self.get_lua_error() {
            Some(e) => format!("{e:?}")
                .as_bytes()
                .chunks(42) // Kinda arbitrary but should fit in most terminals
                .enumerate()
                .map(|(idx, buf)| {
                    format!(
                        "{}{}{}{}{}",
                        if idx == 0 { "" } else { "\n│ " },
                        if idx == 0 {
                            "".to_string()
                        } else {
                            " ".repeat(16)
                        },
                        if idx == 0 { "" } else { " │ " },
                        String::from_utf8_lossy(buf),
                        if buf.len() == 42 { " │" } else { "" },
                    )
                })
                .collect::<String>(),
            None => "-".to_string(),
        };
        let parts = vec![
            format!("Status           │ {status}"),
            format!("Tasks active     │ {}", self.num_blocking),
            format!("Tasks background │ {}", self.num_background),
            format!("Status code      │ {code}"),
            format!("Lua error        │ {err}"),
        ];
        let lengths = parts
            .iter()
            .map(|part| {
                part.lines()
                    .next()
                    .unwrap()
                    .trim_end_matches(" │")
                    .chars()
                    .count()
            })
            .collect::<Vec<_>>();
        let longest = &parts
            .iter()
            .enumerate()
            .fold(0, |acc, (index, _)| acc.max(lengths[index]));
        let sep = "─".repeat(longest + 2);
        writeln!(f, "┌{}┐", &sep)?;
        for (index, part) in parts.iter().enumerate() {
            writeln!(
                f,
                "│ {}{} │",
                part.trim_end_matches(" │"),
                " ".repeat(
                    longest
                        - part
                            .lines()
                            .last()
                            .unwrap()
                            .trim_end_matches(" │")
                            .chars()
                            .count()
                )
            )?;
            if index < parts.len() - 1 {
                writeln!(f, "┝{}┥", &sep)?;
            }
        }
        write!(f, "└{}┘", &sep)?;
        Ok(())
    }
}
