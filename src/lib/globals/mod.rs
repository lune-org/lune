mod console;
mod fs;
mod net;
mod process;
mod task;

pub use console::Console as ConsoleGlobal;
pub use fs::Fs as FsGlobal;
pub use net::Net as NetGlobal;
pub use process::Process as ProcessGlobal;
pub use task::Task as TaskGlobal;
