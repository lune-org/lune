use std::{
    borrow::Borrow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mlua::{prelude::*, AppDataRef};
use thiserror::Error;

use super::Instance;

type InstanceRegistryMap = HashMap<String, HashMap<String, LuaRegistryKey>>;

#[derive(Debug, Clone, Error)]
pub enum InstanceRegistryError {
    #[error("class name '{0}' is not valid")]
    InvalidClassName(String),
    #[error("class '{class_name}' already registered method '{method_name}'")]
    MethodAlreadyExists {
        class_name: String,
        method_name: String,
    },
    #[error("class '{class_name}' already registered property '{property_name}'")]
    PropertyAlreadyExists {
        class_name: String,
        property_name: String,
    },
}

#[derive(Debug, Clone)]
pub struct InstanceRegistry {
    getters: Arc<Mutex<InstanceRegistryMap>>,
    setters: Arc<Mutex<InstanceRegistryMap>>,
    methods: Arc<Mutex<InstanceRegistryMap>>,
}

impl InstanceRegistry {
    // NOTE: We lazily create the instance registry instead
    // of always creating it together with the roblox builtin
    // since it is less commonly used and it simplifies some app
    // data borrowing relationship problems we'd otherwise have
    fn get_or_create(lua: &Lua) -> AppDataRef<'_, Self> {
        if lua.app_data_ref::<Self>().is_none() {
            lua.set_app_data(Self {
                getters: Arc::new(Mutex::new(HashMap::new())),
                setters: Arc::new(Mutex::new(HashMap::new())),
                methods: Arc::new(Mutex::new(HashMap::new())),
            });
        }
        lua.app_data_ref::<Self>()
            .expect("Missing InstanceRegistry in app data")
    }

    /**
        Inserts a method into the instance registry.

        # Errors

        - If the method already exists in the registry.
    */
    pub fn insert_method(
        lua: &Lua,
        class_name: &str,
        method_name: &str,
        method: LuaFunction,
    ) -> Result<(), InstanceRegistryError> {
        let registry = Self::get_or_create(lua);

        let mut methods = registry
            .methods
            .lock()
            .expect("Failed to lock instance registry methods");

        let class_methods = methods.entry(class_name.to_string()).or_default();
        if class_methods.contains_key(method_name) {
            return Err(InstanceRegistryError::MethodAlreadyExists {
                class_name: class_name.to_string(),
                method_name: method_name.to_string(),
            });
        }

        let key = lua
            .create_registry_value(method)
            .expect("Failed to store method in lua registry");
        class_methods.insert(method_name.to_string(), key);

        Ok(())
    }

    /**
        Inserts a property getter into the instance registry.

        # Errors

        - If the property already exists in the registry.
    */
    pub fn insert_property_getter(
        lua: &Lua,
        class_name: &str,
        property_name: &str,
        property_getter: LuaFunction,
    ) -> Result<(), InstanceRegistryError> {
        let registry = Self::get_or_create(lua);

        let mut getters = registry
            .getters
            .lock()
            .expect("Failed to lock instance registry getters");

        let class_getters = getters.entry(class_name.to_string()).or_default();
        if class_getters.contains_key(property_name) {
            return Err(InstanceRegistryError::PropertyAlreadyExists {
                class_name: class_name.to_string(),
                property_name: property_name.to_string(),
            });
        }

        let key = lua
            .create_registry_value(property_getter)
            .expect("Failed to store getter in lua registry");
        class_getters.insert(property_name.to_string(), key);

        Ok(())
    }

    /**
        Inserts a property setter into the instance registry.

        # Errors

        - If the property already exists in the registry.
    */
    pub fn insert_property_setter(
        lua: &Lua,
        class_name: &str,
        property_name: &str,
        property_setter: LuaFunction,
    ) -> Result<(), InstanceRegistryError> {
        let registry = Self::get_or_create(lua);

        let mut setters = registry
            .setters
            .lock()
            .expect("Failed to lock instance registry getters");

        let class_setters = setters.entry(class_name.to_string()).or_default();
        if class_setters.contains_key(property_name) {
            return Err(InstanceRegistryError::PropertyAlreadyExists {
                class_name: class_name.to_string(),
                property_name: property_name.to_string(),
            });
        }

        let key = lua
            .create_registry_value(property_setter)
            .expect("Failed to store getter in lua registry");
        class_setters.insert(property_name.to_string(), key);

        Ok(())
    }

    /**
        Finds a method in the instance registry.

        Returns `None` if the method is not found.
    */
    #[must_use]
    pub fn find_method(lua: &Lua, instance: &Instance, method_name: &str) -> Option<LuaFunction> {
        let registry = Self::get_or_create(lua);
        let methods = registry
            .methods
            .lock()
            .expect("Failed to lock instance registry methods");

        class_name_chain(&instance.class_name)
            .iter()
            .find_map(|&class_name| {
                methods
                    .get(class_name)
                    .and_then(|class_methods| class_methods.get(method_name))
                    .map(|key| lua.registry_value::<LuaFunction>(key).unwrap())
            })
    }

    /**
        Finds a property getter in the instance registry.

        Returns `None` if the property getter is not found.
    */
    #[must_use]
    pub fn find_property_getter(
        lua: &Lua,
        instance: &Instance,
        property_name: &str,
    ) -> Option<LuaFunction> {
        let registry = Self::get_or_create(lua);
        let getters = registry
            .getters
            .lock()
            .expect("Failed to lock instance registry getters");

        class_name_chain(&instance.class_name)
            .iter()
            .find_map(|&class_name| {
                getters
                    .get(class_name)
                    .and_then(|class_getters| class_getters.get(property_name))
                    .map(|key| lua.registry_value::<LuaFunction>(key).unwrap())
            })
    }

    /**
        Finds a property setter in the instance registry.

        Returns `None` if the property setter is not found.
    */
    #[must_use]
    pub fn find_property_setter(
        lua: &Lua,
        instance: &Instance,
        property_name: &str,
    ) -> Option<LuaFunction> {
        let registry = Self::get_or_create(lua);
        let setters = registry
            .setters
            .lock()
            .expect("Failed to lock instance registry setters");

        class_name_chain(&instance.class_name)
            .iter()
            .find_map(|&class_name| {
                setters
                    .get(class_name)
                    .and_then(|class_setters| class_setters.get(property_name))
                    .map(|key| lua.registry_value::<LuaFunction>(key).unwrap())
            })
    }
}

/**
    Gets the class name chain for a given class name.

    The chain starts with the given class name and ends with the root class.

    # Panics

    Panics if the class name is not valid.
*/
#[must_use]
pub fn class_name_chain(class_name: &str) -> Vec<&str> {
    let db = rbx_reflection_database::get();

    let mut list = vec![class_name];
    let mut current_name = class_name;

    loop {
        let class_descriptor = db
            .classes
            .get(current_name)
            .expect("Got invalid class name");
        if let Some(sup) = &class_descriptor.superclass {
            current_name = sup.borrow();
            list.push(current_name);
        } else {
            break;
        }
    }

    list
}
