use crate::library::StandardLibrary;
use std::collections::HashMap;

pub struct RequireStorage<'a> {
    stds: HashMap<&'a str, Box<dyn StandardLibrary>>,
}
