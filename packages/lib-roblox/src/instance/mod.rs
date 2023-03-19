use std::{
    fmt,
    sync::{Arc, RwLock},
};

use mlua::prelude::*;
use rbx_dom_weak::{
    types::{Ref as DomRef, Variant as DomValue},
    Instance as DomInstance, InstanceBuilder as DomInstanceBuilder, WeakDom,
};

use crate::{
    datatypes::{
        conversion::{DomValueToLua, LuaToDomValue},
        types::EnumItem,
        userdata_impl_eq, userdata_impl_to_string,
    },
    shared::instance::{
        class_exists, class_is_a, find_property_enum, find_property_type, property_is_enum,
    },
};

#[derive(Debug, Clone)]
pub struct Instance {
    pub(crate) dom: Arc<RwLock<WeakDom>>,
    pub(crate) dom_ref: DomRef,
    pub(crate) class_name: String,
}

impl Instance {
    /**
        Creates a new `Instance` from a document and dom object ref.
    */
    pub fn new(dom: &Arc<RwLock<WeakDom>>, dom_ref: DomRef) -> Self {
        let class_name = dom
            .read()
            .expect("Failed to get read access to document")
            .get_by_ref(dom_ref)
            .expect("Failed to find instance in document")
            .class
            .clone();
        Self {
            dom: Arc::clone(dom),
            dom_ref,
            class_name,
        }
    }

    /**
        Creates a new orphaned `Instance` with a given class name.

        An orphaned instance does not belong to any particular document and
        is instead part of the internal weak dom for orphaned lua instances,
        it can however be re-parented to a "real" document and weak dom.
    */
    pub fn new_orphaned(lua: &Lua, class_name: impl AsRef<str>) -> Self {
        let dom_lua = lua
            .app_data_mut::<Arc<RwLock<WeakDom>>>()
            .expect("Failed to find internal lua weak dom");
        let mut dom = dom_lua
            .write()
            .expect("Failed to get write access to document");

        let class_name = class_name.as_ref();
        let dom_root = dom.root_ref();
        let dom_ref = dom.insert(dom_root, DomInstanceBuilder::new(class_name.to_string()));

        Self {
            dom: Arc::clone(&dom_lua),
            dom_ref,
            class_name: class_name.to_string(),
        }
    }

    /**
        Checks if the instance matches or inherits a given class name.
    */
    pub fn is_a(&self, class_name: impl AsRef<str>) -> bool {
        class_is_a(&self.class_name, class_name).unwrap_or(false)
    }

    /**
        Checks if the instance has been destroyed.
    */
    pub fn is_destroyed(&self) -> bool {
        self.dom
            .read()
            .expect("Failed to get read access to document")
            .get_by_ref(self.dom_ref)
            .is_none()
    }

    /**
        Checks if the instance is the root instance.
    */
    pub fn is_root(&self) -> bool {
        self.dom
            .read()
            .expect("Failed to get read access to document")
            .root_ref()
            == self.dom_ref
    }

    /**
        Gets the name of the instance, if it exists.
    */
    pub fn get_name(&self) -> String {
        let dom = self
            .dom
            .read()
            .expect("Failed to get read access to document");
        dom.get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .name
            .clone()
    }

    /**
        Sets the name of the instance, if it exists.
    */
    pub fn set_name(&self, name: impl Into<String>) {
        let mut dom = self
            .dom
            .write()
            .expect("Failed to get write access to document");
        dom.get_by_ref_mut(self.dom_ref)
            .expect("Failed to find instance in document")
            .name = name.into()
    }

    /**
        Gets the parent of the instance, if it exists.
    */
    pub fn get_parent(&self) -> Option<Instance> {
        let dom = self
            .dom
            .read()
            .expect("Failed to get read access to document");
        let parent_ref = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .parent();
        if parent_ref == dom.root_ref() {
            None
        } else {
            Some(Self::new(&self.dom, parent_ref))
        }
    }

    /**
        Sets the parent of the instance, if it exists.

        Note that this can transfer between different weak doms,
        and assumes that separate doms always have unique root referents.

        If doms do not have unique root referents then this operation may panic.
    */
    pub fn set_parent(&self, parent: Instance) {
        let mut dom_source = self
            .dom
            .write()
            .expect("Failed to get read access to source document");
        let dom_target = parent
            .dom
            .read()
            .expect("Failed to get read access to target document");
        let target_ref = dom_target
            .get_by_ref(parent.dom_ref)
            .expect("Failed to find instance in target document")
            .parent();
        if dom_source.root_ref() == dom_target.root_ref() {
            dom_source.transfer_within(self.dom_ref, target_ref);
        } else {
            // NOTE: We must drop the previous dom_target read handle here first so
            // that we can get exclusive write access for transferring across doms
            drop(dom_target);
            let mut dom_target = parent
                .dom
                .try_write()
                .expect("Failed to get write access to target document");
            dom_source.transfer(self.dom_ref, &mut dom_target, target_ref)
        }
    }

    /**
        Sets the parent of the instance, if it exists, to nil, making it orphaned.

        An orphaned instance does not belong to any particular document and
        is instead part of the internal weak dom for orphaned lua instances,
        it can however be re-parented to a "real" document and weak dom.
    */
    pub fn set_parent_to_nil(&self, lua: &Lua) {
        let mut dom_source = self
            .dom
            .write()
            .expect("Failed to get read access to source document");
        let dom_lua = lua
            .app_data_mut::<Arc<RwLock<WeakDom>>>()
            .expect("Failed to find internal lua weak dom");
        let mut dom_target = dom_lua
            .write()
            .expect("Failed to get write access to target document");
        let target_ref = dom_target.root_ref();
        dom_source.transfer(self.dom_ref, &mut dom_target, target_ref)
    }

    /**
        Gets a property for the instance, if it exists.
    */
    pub fn get_property(&self, name: impl AsRef<str>) -> Option<DomValue> {
        self.dom
            .read()
            .expect("Failed to get read access to document")
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .properties
            .get(name.as_ref())
            .cloned()
    }

    /**
        Sets a property for the instance.

        Note that setting a property here will not fail even if the
        property does not actually exist for the instance class.
    */
    pub fn set_property(&self, name: impl AsRef<str>, value: DomValue) {
        self.dom
            .write()
            .expect("Failed to get read access to document")
            .get_by_ref_mut(self.dom_ref)
            .expect("Failed to find instance in document")
            .properties
            .insert(name.as_ref().to_string(), value);
    }

    /**
        Finds a child of the instance using the given predicate callback.
    */
    pub fn find_child<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        let dom = self
            .dom
            .read()
            .expect("Failed to get read access to document");
        let children = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .children();
        children.iter().find_map(|child_ref| {
            if let Some(child_inst) = dom.get_by_ref(*child_ref) {
                if predicate(child_inst) {
                    Some(Self::new(&self.dom, *child_ref))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /**
        Finds an ancestor of the instance using the given predicate callback.
    */
    pub fn find_ancestor<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        let dom = self
            .dom
            .read()
            .expect("Failed to get read access to document");
        let mut ancestor_ref = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .parent();
        while let Some(ancestor) = dom.get_by_ref(ancestor_ref) {
            if predicate(ancestor) {
                return Some(Self::new(&self.dom, ancestor_ref));
            } else {
                ancestor_ref = ancestor.parent();
            }
        }
        None
    }
}

impl Instance {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|lua, class_name: String| {
                if class_exists(&class_name) {
                    Instance::new_orphaned(lua, class_name).to_lua(lua)
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "{} is not a valid class name",
                        class_name
                    )))
                }
            })?,
        )
    }
}

impl LuaUserData for Instance {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        /*
            Getting a value does the following:

            1. Check if it is a special property like "ClassName", "Name" or "Parent"
            2. Try to get a known instance property
            3. Try to get a current child of the instance
            4. No valid property or instance found, throw error
        */
        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, prop_name: String| {
            match prop_name.as_str() {
                "ClassName" => return this.class_name.clone().to_lua(lua),
                "Name" => {
                    return this.get_name().to_lua(lua);
                }
                "Parent" => {
                    return this.get_parent().to_lua(lua);
                }
                _ => {}
            }

            if let Some(prop) = this.get_property(&prop_name) {
                match LuaValue::dom_value_to_lua(lua, &prop) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(e.into()),
                }
            } else if let Some(inst) = this.find_child(|inst| inst.name == prop_name) {
                Ok(LuaValue::UserData(lua.create_userdata(inst)?))
            } else {
                Err(LuaError::RuntimeError(format!(
                    "{} is not a valid member of {}",
                    prop_name, this
                )))
            }
        });
        /*
            Setting a value does the following:

            1. Check if it is a special property like "ClassName", "Name" or "Parent"
            2. Check if a property exists for the wanted name
            3a. Set a strict enum from a given EnumItem OR
            3b. Set a normal property from a given value
        */
        methods.add_meta_method_mut(
            LuaMetaMethod::NewIndex,
            |lua, this, (prop_name, prop_value): (String, LuaValue)| {
                match prop_name.as_str() {
                    "ClassName" => {
                        return Err(LuaError::RuntimeError(
                            "ClassName can not be written to".to_string(),
                        ))
                    }
                    "Name" => {
                        let name = String::from_lua(prop_value, lua)?;
                        this.set_name(name);
                        return Ok(());
                    }
                    "Parent" => {
                        type Parent = Option<Instance>;
                        match Parent::from_lua(prop_value, lua)? {
                            Some(parent) => this.set_parent(parent),
                            None => this.set_parent_to_nil(lua),
                        }
                        return Ok(());
                    }
                    _ => {}
                }

                let is_enum = match property_is_enum(&this.class_name, &prop_name) {
                    Some(b) => b,
                    None => {
                        return Err(LuaError::RuntimeError(format!(
                            "{} is not a valid member of {}",
                            prop_name, this
                        )))
                    }
                };

                if is_enum {
                    let enum_name = find_property_enum(&this.class_name, &prop_name).unwrap();
                    match EnumItem::from_lua(prop_value, lua) {
                        Ok(given_enum) if given_enum.name == enum_name => {
                            this.set_property(prop_name, DomValue::Enum(given_enum.into()));
                            Ok(())
                        }
                        Ok(given_enum) => Err(LuaError::RuntimeError(format!(
                            "Expected Enum.{}, got Enum.{}",
                            enum_name, given_enum.name
                        ))),
                        Err(e) => Err(e),
                    }
                } else {
                    let dom_type = find_property_type(&this.class_name, &prop_name).unwrap();
                    match prop_value.lua_to_dom_value(lua, dom_type) {
                        Ok(dom_value) => {
                            this.set_property(prop_name, dom_value);
                            Ok(())
                        }
                        Err(e) => Err(e.into()),
                    }
                }
            },
        );
        /*
            Implementations of base methods on the Instance class

            Currently implemented:

            * FindFirstAncestor
            * FindFirstAncestorOfClass
            * FindFirstAncestorWhichIsA
            * FindFirstChild
            * FindFirstChildOfClass
            * FindFirstChildWhichIsA
            * IsAncestorOf
            * IsDescendantOf

            Not yet implemented, but planned:

            * Clone
            * Destroy
            * FindFirstDescendant
            * GetChildren
            * GetDescendants
            * GetFullName
            * GetAttribute
            * GetAttributes
            * SetAttribute
        */
        methods.add_method("FindFirstAncestor", |lua, this, name: String| {
            this.find_ancestor(|child| child.name == name).to_lua(lua)
        });
        methods.add_method(
            "FindFirstAncestorOfClass",
            |lua, this, class_name: String| {
                this.find_ancestor(|child| child.class == class_name)
                    .to_lua(lua)
            },
        );
        methods.add_method(
            "FindFirstAncestorWhichIsA",
            |lua, this, class_name: String| {
                this.find_ancestor(|child| class_is_a(&child.class, &class_name).unwrap_or(false))
                    .to_lua(lua)
            },
        );
        methods.add_method("FindFirstChild", |lua, this, name: String| {
            this.find_child(|child| child.name == name).to_lua(lua)
        });
        methods.add_method("FindFirstChildOfClass", |lua, this, class_name: String| {
            this.find_child(|child| child.class == class_name)
                .to_lua(lua)
        });
        methods.add_method("FindFirstChildWhichIsA", |lua, this, class_name: String| {
            this.find_child(|child| class_is_a(&child.class, &class_name).unwrap_or(false))
                .to_lua(lua)
        });
        methods.add_method("IsAncestorOf", |_, this, instance: Instance| {
            Ok(instance
                .find_ancestor(|ancestor| ancestor.referent() == this.dom_ref)
                .is_some())
        });
        methods.add_method("IsDescendantOf", |_, this, instance: Instance| {
            Ok(this
                .find_ancestor(|ancestor| ancestor.referent() == instance.dom_ref)
                .is_some())
        });
        // FUTURE: We could pass the "methods" struct to some other functions
        // here to add inheritance-like behavior and class-specific methods
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.dom_ref == other.dom_ref
    }
}
