use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

type ErrorCallback = Box<dyn Fn(LuaError) + Send + 'static>;

#[derive(Clone)]
pub(crate) struct ThreadErrorCallback {
    inner: Rc<RefCell<Option<ErrorCallback>>>,
}

impl ThreadErrorCallback {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(None)),
        }
    }

    pub fn replace(&self, callback: impl Fn(LuaError) + Send + 'static) {
        self.inner.borrow_mut().replace(Box::new(callback));
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().take();
    }

    pub fn call(&self, error: &LuaError) {
        if let Some(cb) = &*self.inner.borrow() {
            cb(error.clone());
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn default_error_callback(e: LuaError) {
    eprintln!("{e}");
}

impl Default for ThreadErrorCallback {
    fn default() -> Self {
        let this = Self::new();
        this.replace(default_error_callback);
        this
    }
}
