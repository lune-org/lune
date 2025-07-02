use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use async_channel::{Receiver, Sender};
use async_fs::read as read_file;

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSchedulerExt;

use super::constants::FILE_CHUNK_PREFIX;

type RequireResult = LuaResult<LuaMultiValue>;
type RequireResultSender = Sender<RequireResult>;
type RequireResultReceiver = Receiver<RequireResult>;

/**
    Inner clonable state for the require loader.
*/
#[derive(Debug, Clone)]
struct RequireLoaderState {
    tx: Rc<RefCell<HashMap<PathBuf, RequireResultSender>>>,
    rx: Rc<RefCell<HashMap<PathBuf, RequireResultReceiver>>>,
}

impl RequireLoaderState {
    fn new() -> Self {
        Self {
            tx: Rc::new(RefCell::new(HashMap::new())),
            rx: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn get_pending_at_path(&self, path: &Path) -> Option<RequireResultReceiver> {
        self.rx.borrow().get(path).cloned()
    }

    fn create_pending_at_path(&self, path: &Path) -> RequireResultSender {
        let (tx, rx) = async_channel::bounded(1);
        self.tx.borrow_mut().insert(path.to_path_buf(), tx.clone());
        self.rx.borrow_mut().insert(path.to_path_buf(), rx);
        tx
    }

    fn remove_pending_at_path(&self, path: &Path) {
        self.tx.borrow_mut().remove(path);
        self.rx.borrow_mut().remove(path);
    }
}

/**
    A loader implementation for `require` that ensures modules only load
    exactly once - even if they yield / async during the loading process.
*/
#[derive(Debug, Clone)]
pub(crate) struct RequireLoader {
    state: RequireLoaderState,
}

impl RequireLoader {
    pub(crate) fn new() -> Self {
        Self {
            state: RequireLoaderState::new(),
        }
    }

    pub(crate) fn load(
        &self,
        lua: &Lua,
        relative_path: &Path,
        absolute_path: &Path,
    ) -> LuaResult<LuaFunction> {
        let relative_path = relative_path.to_path_buf();
        let absolute_path = absolute_path.to_path_buf();

        let state = self.state.clone();

        lua.create_async_function(move |lua, (): ()| {
            let relative_path = relative_path.clone();
            let absolute_path = absolute_path.clone();

            let state = state.clone();

            async move {
                if let Some(rx) = state.get_pending_at_path(&absolute_path) {
                    rx.recv().await.unwrap()
                } else {
                    let tx = state.create_pending_at_path(&absolute_path);

                    let chunk_name = format!("{FILE_CHUNK_PREFIX}{}", relative_path.display());
                    let chunk_bytes = read_file(&absolute_path).await?;

                    let chunk = lua.load(chunk_bytes).set_name(chunk_name);

                    let thread_id = lua.push_thread_back(chunk, ())?;
                    lua.track_thread(thread_id);
                    lua.wait_for_thread(thread_id).await;
                    let thread_res = lua.get_thread_result(thread_id).unwrap();

                    if tx.receiver_count() > 0 {
                        let _ = tx.send(thread_res.clone()).await;
                    }

                    state.remove_pending_at_path(&absolute_path);

                    thread_res
                }
            }
        })
    }
}
