/*!
    Line coverage of the Luau chunks loaded by a runtime.
*/

use std::{collections::BTreeMap, fmt::Write, path::Path};

use mlua::prelude::*;

use crate::path::get_current_dir;

/**
    The coverage level to compile chunks with to collect their coverage.

    Level 2 counts both statements and expressions, the same as `luau --coverage`.
*/
pub const COVERAGE_LEVEL: u8 = 2;

/**
    Line coverage of Luau chunks, stored as app data in a Luau VM.

    Only tracked chunks are included in reports, and a chunk
    records its coverage only when compiled with a coverage
    level above zero - see [`COVERAGE_LEVEL`].
*/
#[derive(Debug, Default)]
pub struct Coverage {
    chunks: Vec<(String, LuaFunction)>,
}

impl Coverage {
    /**
        Tracks the coverage of a loaded chunk, if the given Luau VM has coverage enabled.

        The name should be the path to the file that the chunk was loaded from, if any.
    */
    pub fn track(lua: &Lua, name: impl Into<String>, chunk: &LuaFunction) {
        if let Some(mut coverage) = lua.app_data_mut::<Self>() {
            coverage.chunks.push((name.into(), chunk.clone()));
        }
    }

    /**
        Creates a report of the tracked chunks in the LCOV tracefile format.

        Chunks loaded from the same file more than once are combined into one record,
        and paths inside the current working directory are made relative to it.
    */
    #[must_use]
    pub fn to_lcov(&self) -> String {
        let mut files = BTreeMap::<String, FileCoverage>::new();

        for (name, chunk) in &self.chunks {
            let file = files.entry(relative_to_cwd(name)).or_default();
            chunk.coverage(|info| file.add(info));
        }

        let mut lcov = String::from("TN:\n");
        for (name, file) in &files {
            // Writing to a string does not fail
            let _ = file.write_lcov(&mut lcov, name);
        }
        lcov
    }
}

#[derive(Debug, Default)]
struct FileCoverage {
    // Calls to each function, by the line it is defined on and its name
    functions: BTreeMap<(i32, String), u64>,
    // Hits on each line with code
    lines: BTreeMap<usize, u64>,
}

impl FileCoverage {
    fn add(&mut self, info: LuaCoverageInfo) {
        // Function names are the same as in the reports of `luau --coverage`
        let name = if info.depth == 0 {
            String::from("<main>")
        } else {
            let function = info.function.as_deref().unwrap_or("<anonymous>");
            format!("{function}:{}", info.line_defined)
        };

        // A function's first line with code is hit once for each call to it
        let mut calls = None;
        for (line, hits) in info.hits.iter().enumerate() {
            // Lines without code have a negative hit count
            if let Ok(hits) = u64::try_from(*hits) {
                *self.lines.entry(line).or_default() += hits;
                calls.get_or_insert(hits);
            }
        }

        *self.functions.entry((info.line_defined, name)).or_default() += calls.unwrap_or_default();
    }

    fn write_lcov(&self, lcov: &mut String, name: &str) -> std::fmt::Result {
        writeln!(lcov, "SF:{name}")?;

        for (line, function) in self.functions.keys() {
            writeln!(lcov, "FN:{line},{function}")?;
        }
        for ((_, function), calls) in &self.functions {
            writeln!(lcov, "FNDA:{calls},{function}")?;
        }
        writeln!(lcov, "FNF:{}", self.functions.len())?;
        writeln!(lcov, "FNH:{}", count_hit(self.functions.values()))?;

        for (line, hits) in &self.lines {
            writeln!(lcov, "DA:{line},{hits}")?;
        }
        writeln!(lcov, "LF:{}", self.lines.len())?;
        writeln!(lcov, "LH:{}", count_hit(self.lines.values()))?;

        writeln!(lcov, "end_of_record")
    }
}

fn count_hit<'a>(counts: impl Iterator<Item = &'a u64>) -> usize {
    counts.filter(|count| **count > 0).count()
}

fn relative_to_cwd(name: &str) -> String {
    match Path::new(name).strip_prefix(get_current_dir()) {
        Ok(relative) => relative.display().to_string(),
        Err(_) => name.to_string(),
    }
}
