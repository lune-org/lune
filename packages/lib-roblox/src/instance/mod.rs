use std::{collections::VecDeque, fmt, sync::RwLock};

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
    shared::instance::{class_exists, class_is_a, find_property_info},
};

pub(crate) mod data_model;

lazy_static::lazy_static! {
    static ref INTERNAL_DOM: RwLock<WeakDom> =
        RwLock::new(WeakDom::new(DomInstanceBuilder::new("ROOT")));
}

#[derive(Debug, Clone)]
pub struct Instance {
    dom_ref: DomRef,
    class_name: String,
}

impl Instance {
    /**
        Creates a new `Instance` from an existing dom object ref.

        Panics if the instance does not exist in the internal dom,
        or if the given dom object ref points to the dom root.
    */
    fn new(dom_ref: DomRef) -> Self {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        let instance = dom
            .get_by_ref(dom_ref)
            .expect("Failed to find instance in document");

        if instance.referent() == dom.root_ref() {
            panic!("Instances can not be created from dom roots")
        }

        Self {
            dom_ref,
            class_name: instance.class.clone(),
        }
    }

    /**
        Creates a new orphaned `Instance` with a given class name.

        An orphaned instance is an instance at the root of a weak dom.
    */
    fn new_orphaned(class_name: impl AsRef<str>) -> Self {
        let mut dom = INTERNAL_DOM
            .try_write()
            .expect("Failed to get write access to document");

        let class_name = class_name.as_ref();

        let instance = DomInstanceBuilder::new(class_name.to_string());

        let dom_root = dom.root_ref();
        let dom_ref = dom.insert(dom_root, instance);

        Self {
            dom_ref,
            class_name: class_name.to_string(),
        }
    }

    /**
        Creates a new orphaned `Instance` by transferring
        it from an external weak dom to the internal one.

        An orphaned instance is an instance at the root of a weak dom.

        Panics if the given dom ref is the root dom ref of the external weak dom.
    */
    pub fn from_external_dom(external_dom: &mut WeakDom, external_dom_ref: DomRef) -> Self {
        {
            let mut dom = INTERNAL_DOM
                .try_write()
                .expect("Failed to get write access to document");
            let dom_root = dom.root_ref();

            external_dom.transfer(external_dom_ref, &mut dom, dom_root);
        }

        Self::new(external_dom_ref)
    }

    /**
        Clones an instance to an external weak dom.

        This will place the instance as a child of the
        root of the weak dom, and return its referent.
    */
    pub fn clone_into_external_dom(self, external_dom: &mut WeakDom) -> DomRef {
        let cloned = self.clone_instance();

        let mut dom = INTERNAL_DOM
            .try_write()
            .expect("Failed to get write access to document");

        let internal_dom_ref = cloned.dom_ref;
        let external_root_ref = external_dom.root_ref();

        dom.transfer(internal_dom_ref, external_dom, external_root_ref);

        internal_dom_ref
    }

    /**
        Clones the instance and all of its descendants, and orphans it.

        To then save the new instance it must be re-parented,
        which matches the exact behavior of Roblox's instances.

        ### See Also
        * [`Clone`](https://create.roblox.com/docs/reference/engine/classes/Instance#Clone)
        on the Roblox Developer Hub
    */
    pub fn clone_instance(&self) -> Instance {
        // NOTE: We create a new scope here to avoid deadlocking since
        // our clone implementation must have exclusive write access
        let parent_ref = {
            INTERNAL_DOM
                .try_read()
                .expect("Failed to get read access to document")
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .parent()
        };

        let new_ref = Self::clone_inner(self.dom_ref, parent_ref);
        let new_inst = Self::new(new_ref);

        new_inst.set_parent(None);
        new_inst
    }

    pub fn clone_inner(dom_ref: DomRef, parent_ref: DomRef) -> DomRef {
        // NOTE: We create a new scope here to avoid deadlocking since
        // our clone implementation must have exclusive write access
        let (new_ref, child_refs) = {
            let mut dom = INTERNAL_DOM
                .try_write()
                .expect("Failed to get write access to document");

            let (new_class, new_name, new_props, child_refs) = {
                let instance = dom
                    .get_by_ref(dom_ref)
                    .expect("Failed to find instance in document");
                (
                    instance.class.to_string(),
                    instance.name.to_string(),
                    instance.properties.clone(),
                    instance.children().to_vec(),
                )
            };

            let new_ref = dom.insert(
                parent_ref,
                DomInstanceBuilder::new(new_class)
                    .with_name(new_name)
                    .with_properties(new_props),
            );

            (new_ref, child_refs)
        };

        for child_ref in child_refs {
            Self::clone_inner(child_ref, new_ref);
        }

        new_ref
    }

    /**
        Destroys the instance, removing it completely
        from the weak dom with no way of recovering it.

        All member methods will throw errors when called from lua and panic
        when called from rust after the instance has been destroyed.

        Returns `true` if destroyed successfully, `false` if already destroyed.

        ### See Also
        * [`Destroy`](https://create.roblox.com/docs/reference/engine/classes/Instance#Destroy)
        on the Roblox Developer Hub
    */
    pub fn destroy(&mut self) -> bool {
        if self.is_destroyed() {
            false
        } else {
            let mut dom = INTERNAL_DOM
                .try_write()
                .expect("Failed to get write access to document");

            dom.destroy(self.dom_ref);
            true
        }
    }

    fn ensure_not_destroyed(&self) -> LuaResult<()> {
        if self.is_destroyed() {
            Err(LuaError::RuntimeError(format!(
                "Tried to access destroyed instance '{}'",
                self
            )))
        } else {
            Ok(())
        }
    }

    fn is_destroyed(&self) -> bool {
        // NOTE: This property can not be cached since instance references
        // other than this one may have destroyed this one, and we don't
        // keep track of all current instance reference structs
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");
        dom.get_by_ref(self.dom_ref).is_none()
    }

    /**
        Destroys all child instances.

        ### See Also
        * [`Instance::Destroy`] for more info about what happens when an instance gets destroyed
        * [`ClearAllChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#ClearAllChildren)
        on the Roblox Developer Hub
    */
    pub fn clear_all_children(&mut self) {
        let mut dom = INTERNAL_DOM
            .try_write()
            .expect("Failed to get write access to document");

        let instance = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document");

        let child_refs = instance.children().to_vec();
        for child_ref in child_refs {
            dom.destroy(child_ref);
        }
    }

    /**
        Checks if the instance matches or inherits a given class name.

        ### See Also
        * [`IsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
        on the Roblox Developer Hub
    */
    pub fn is_a(&self, class_name: impl AsRef<str>) -> bool {
        class_is_a(&self.class_name, class_name).unwrap_or(false)
    }

    /**
        Gets the class name of the instance.

        This will return the correct class name even if the instance has been destroyed.

        ### See Also
        * [`ClassName`](https://create.roblox.com/docs/reference/engine/classes/Instance#ClassName)
        on the Roblox Developer Hub
    */
    pub fn get_class_name(&self) -> &str {
        self.class_name.as_str()
    }

    /**
        Gets the name of the instance, if it exists.

        ### See Also
        * [`Name`](https://create.roblox.com/docs/reference/engine/classes/Instance#Name)
        on the Roblox Developer Hub
    */
    pub fn get_name(&self) -> String {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        dom.get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .name
            .clone()
    }

    /**
        Sets the name of the instance, if it exists.

        ### See Also
        * [`Name`](https://create.roblox.com/docs/reference/engine/classes/Instance#Name)
        on the Roblox Developer Hub
    */
    pub fn set_name(&self, name: impl Into<String>) {
        let mut dom = INTERNAL_DOM
            .try_write()
            .expect("Failed to get write access to document");

        dom.get_by_ref_mut(self.dom_ref)
            .expect("Failed to find instance in document")
            .name = name.into()
    }

    /**
        Gets the parent of the instance, if it exists.

        ### See Also
        * [`Parent`](https://create.roblox.com/docs/reference/engine/classes/Instance#Parent)
        on the Roblox Developer Hub
    */
    pub fn get_parent(&self) -> Option<Instance> {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        let parent_ref = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .parent();

        if parent_ref == dom.root_ref() {
            None
        } else {
            Some(Self::new(parent_ref))
        }
    }

    /**
        Sets the parent of the instance, if it exists.

        If the provided parent is [`None`] the instance will become orphaned.

        An orphaned instance is an instance at the root of a weak dom.

        ### See Also
        * [`Parent`](https://create.roblox.com/docs/reference/engine/classes/Instance#Parent)
        on the Roblox Developer Hub
    */
    pub fn set_parent(&self, parent: Option<Instance>) {
        let mut dom = INTERNAL_DOM
            .try_write()
            .expect("Failed to get write access to target document");

        let parent_ref = parent
            .map(|parent| parent.dom_ref)
            .unwrap_or_else(|| dom.root_ref());

        dom.transfer_within(self.dom_ref, parent_ref);
    }

    /**
        Gets a property for the instance, if it exists.
    */
    pub fn get_property(&self, name: impl AsRef<str>) -> Option<DomValue> {
        INTERNAL_DOM
            .try_read()
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
        INTERNAL_DOM
            .try_write()
            .expect("Failed to get read access to document")
            .get_by_ref_mut(self.dom_ref)
            .expect("Failed to find instance in document")
            .properties
            .insert(name.as_ref().to_string(), value);
    }

    /**
        Gets all of the current children of this `Instance`.

        Note that this is a somewhat expensive operation and that other
        operations using weak dom referents should be preferred if possible.

        ### See Also
        * [`GetChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetChildren)
        on the Roblox Developer Hub
    */
    pub fn get_children(&self) -> Vec<Instance> {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        let children = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .children()
            .to_vec();

        children.into_iter().map(Self::new).collect()
    }

    /**
        Gets all of the current descendants of this `Instance` using a breadth-first search.

        Note that this is a somewhat expensive operation and that other
        operations using weak dom referents should be preferred if possible.

        ### See Also
        * [`GetDescendants`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetDescendants)
        on the Roblox Developer Hub
    */
    pub fn get_descendants(&self) -> Vec<Instance> {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        let mut descendants = Vec::new();
        let mut queue = VecDeque::from_iter(
            dom.get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .children(),
        );

        while let Some(queue_ref) = queue.pop_front() {
            descendants.push(*queue_ref);
            let queue_inst = dom.get_by_ref(*queue_ref).unwrap();
            for queue_ref_inner in queue_inst.children().iter().rev() {
                queue.push_front(queue_ref_inner);
            }
        }

        descendants.into_iter().map(Self::new).collect()
    }

    /**
        Gets the "full name" of this instance.

        This will be a path composed of instance names from the top-level
        ancestor of this instance down to itself, in the following format:

        `Ancestor.Child.Descendant.Instance`

        ### See Also
        * [`GetFullName`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetFullName)
        on the Roblox Developer Hub
    */
    pub fn get_full_name(&self) -> String {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");
        let dom_root = dom.root_ref();

        let mut parts = Vec::new();
        let mut instance_ref = self.dom_ref;

        while let Some(instance) = dom.get_by_ref(instance_ref) {
            if instance_ref != dom_root && instance.class != data_model::CLASS_NAME {
                instance_ref = instance.parent();
                parts.push(instance.name.clone());
            } else {
                break;
            }
        }

        parts.reverse();
        parts.join(".")
    }

    /**
        Finds a child of the instance using the given predicate callback.

        ### See Also
        * [`FindFirstChild`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChild) on the Roblox Developer Hub
        * [`FindFirstChildOfClass`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChildOfClass) on the Roblox Developer Hub
        * [`FindFirstChildWhichIsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChildWhichIsA) on the Roblox Developer Hub
    */
    pub fn find_child<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        let children = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .children()
            .to_vec();

        children.into_iter().find_map(|child_ref| {
            if let Some(child_inst) = dom.get_by_ref(child_ref) {
                if predicate(child_inst) {
                    Some(Self::new(child_ref))
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

        ### See Also
        * [`FindFirstAncestor`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestor) on the Roblox Developer Hub
        * [`FindFirstAncestorOfClass`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestorOfClass) on the Roblox Developer Hub
        * [`FindFirstAncestorWhichIsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestorWhichIsA) on the Roblox Developer Hub
    */
    pub fn find_ancestor<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        let mut ancestor_ref = dom
            .get_by_ref(self.dom_ref)
            .expect("Failed to find instance in document")
            .parent();

        while let Some(ancestor) = dom.get_by_ref(ancestor_ref) {
            if predicate(ancestor) {
                return Some(Self::new(ancestor_ref));
            } else {
                ancestor_ref = ancestor.parent();
            }
        }

        None
    }

    /**
        Finds a descendant of the instance using the given
        predicate callback and a breadth-first search.

        ### See Also
        * [`FindFirstDescendant`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstDescendant)
        on the Roblox Developer Hub
    */
    pub fn find_descendant<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        let dom = INTERNAL_DOM
            .try_read()
            .expect("Failed to get read access to document");

        let mut queue = VecDeque::from_iter(
            dom.get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .children(),
        );

        while let Some(queue_item) = queue
            .pop_front()
            .and_then(|queue_ref| dom.get_by_ref(*queue_ref))
        {
            if predicate(queue_item) {
                return Some(Self::new(queue_item.referent()));
            } else {
                queue.extend(queue_item.children())
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
                    Instance::new_orphaned(class_name).to_lua(lua)
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "Failed to create Instance - '{}' is not a valid class name",
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
            2. Check if a property exists for the wanted name
                2a. Get an existing instance property OR
                2b. Get a property from a known default value
            3. Get a current child of the instance
            4. No valid property or instance found, throw error
        */
        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, prop_name: String| {
            this.ensure_not_destroyed()?;

            match prop_name.as_str() {
                "ClassName" => return this.get_class_name().to_lua(lua),
                "Name" => {
                    return this.get_name().to_lua(lua);
                }
                "Parent" => {
                    return this.get_parent().to_lua(lua);
                }
                _ => {}
            }

            if let Some(info) = find_property_info(&this.class_name, &prop_name) {
                if let Some(prop) = this.get_property(&prop_name) {
                    if let DomValue::Enum(enum_value) = prop {
                        let enum_name = info.enum_name.ok_or_else(|| {
                            LuaError::RuntimeError(format!(
                                "Failed to get property '{}' - encountered unknown enum",
                                prop_name
                            ))
                        })?;
                        EnumItem::from_enum_name_and_value(&enum_name, enum_value.to_u32())
                            .ok_or_else(|| {
                                LuaError::RuntimeError(format!(
                                    "Failed to get property '{}' - Enum.{} does not contain numeric value {}",
                                    prop_name, enum_name, enum_value.to_u32()
                                ))
                            })?
                            .to_lua(lua)
                    } else {
                        Ok(LuaValue::dom_value_to_lua(lua, &prop)?)
                    }
                } else if let (Some(enum_name), Some(enum_value)) = (info.enum_name, info.enum_default) {
                    EnumItem::from_enum_name_and_value(&enum_name, enum_value)
                        .ok_or_else(|| {
                            LuaError::RuntimeError(format!(
                                "Failed to get property '{}' - Enum.{} does not contain numeric value {}",
                                prop_name, enum_name, enum_value
                            ))
                        })?
                        .to_lua(lua)
                } else if let Some(prop_default) = info.value_default {
                    Ok(LuaValue::dom_value_to_lua(lua, prop_default)?)
                } else if info.value_type.is_some() {
                    Err(LuaError::RuntimeError(format!(
                        "Failed to get property '{}' - missing default value",
                        prop_name
                    )))
				} else {
                    Err(LuaError::RuntimeError(format!(
                        "Failed to get property '{}' - malformed property info",
                        prop_name
                    )))
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
                2a. Set a strict enum from a given EnumItem OR
                2b. Set a normal property from a given value
        */
        methods.add_meta_method_mut(
            LuaMetaMethod::NewIndex,
            |lua, this, (prop_name, prop_value): (String, LuaValue)| {
                this.ensure_not_destroyed()?;

                match prop_name.as_str() {
                    "ClassName" => {
                        return Err(LuaError::RuntimeError(format!(
                            "Failed to set property '{}' - property is read-only",
                            prop_name
                        )));
                    }
                    "Name" => {
                        let name = String::from_lua(prop_value, lua)?;
                        this.set_name(name);
                        return Ok(());
                    }
                    "Parent" => {
                        if this.get_class_name() == data_model::CLASS_NAME {
                            return Err(LuaError::RuntimeError(format!(
                                "Failed to set property '{}' - DataModel can not be reparented",
                                prop_name
                            )));
                        }
                        type Parent = Option<Instance>;
                        let parent = Parent::from_lua(prop_value, lua)?;
                        this.set_parent(parent);
                        return Ok(());
                    }
                    _ => {}
                }

                let info = match find_property_info(&this.class_name, &prop_name) {
                    Some(b) => b,
                    None => {
                        return Err(LuaError::RuntimeError(format!(
                            "{} is not a valid member of {}",
                            prop_name, this
                        )))
                    }
                };

                if let Some(enum_name) = info.enum_name {
                    match EnumItem::from_lua(prop_value, lua) {
                        Ok(given_enum) if given_enum.parent.desc.name == enum_name => {
                            this.set_property(prop_name, DomValue::Enum(given_enum.into()));
                            Ok(())
                        }
                        Ok(given_enum) => Err(LuaError::RuntimeError(format!(
                            "Failed to set property '{}' - expected Enum.{}, got Enum.{}",
                            prop_name, enum_name, given_enum.parent.desc.name
                        ))),
                        Err(e) => Err(e),
                    }
                } else if let Some(dom_type) = info.value_type {
                    match prop_value.lua_to_dom_value(lua, dom_type) {
                        Ok(dom_value) => {
                            this.set_property(prop_name, dom_value);
                            Ok(())
                        }
                        Err(e) => Err(e.into()),
                    }
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "Failed to set property '{}' - malformed property info",
                        prop_name
                    )))
                }
            },
        );
        /*
            Implementations of base methods on the Instance class

            Currently implemented:

            * Clone
            * Destroy
            * ClearAllChildren

            * FindFirstAncestor
            * FindFirstAncestorOfClass
            * FindFirstAncestorWhichIsA
            * FindFirstChild
            * FindFirstChildOfClass
            * FindFirstChildWhichIsA
            * FindFirstDescendant

            * GetChildren
            * GetDescendants
            * GetFullName

            * IsA
            * IsAncestorOf
            * IsDescendantOf

            Not yet implemented, but planned:

            * GetAttribute
            * GetAttributes
            * SetAttribute

            It should be noted that any methods that deal with events
            and/or have functionality that affects instances other
            than this instance itself are intentionally left out.
        */
        methods.add_method("Clone", |lua, this, ()| {
            this.ensure_not_destroyed()?;
            this.clone_instance().to_lua(lua)
        });
        methods.add_method_mut("Destroy", |_, this, ()| {
            this.destroy();
            Ok(())
        });
        methods.add_method_mut("ClearAllChildren", |_, this, ()| {
            this.clear_all_children();
            Ok(())
        });
        methods.add_method("GetChildren", |lua, this, ()| {
            this.ensure_not_destroyed()?;
            this.get_children().to_lua(lua)
        });
        methods.add_method("GetDescendants", |lua, this, ()| {
            this.ensure_not_destroyed()?;
            this.get_children().to_lua(lua)
        });
        methods.add_method("GetFullName", |lua, this, ()| {
            this.ensure_not_destroyed()?;
            this.get_full_name().to_lua(lua)
        });
        methods.add_method("FindFirstAncestor", |lua, this, name: String| {
            this.ensure_not_destroyed()?;
            this.find_ancestor(|child| child.name == name).to_lua(lua)
        });
        methods.add_method(
            "FindFirstAncestorOfClass",
            |lua, this, class_name: String| {
                this.ensure_not_destroyed()?;
                this.find_ancestor(|child| child.class == class_name)
                    .to_lua(lua)
            },
        );
        methods.add_method(
            "FindFirstAncestorWhichIsA",
            |lua, this, class_name: String| {
                this.ensure_not_destroyed()?;
                this.find_ancestor(|child| class_is_a(&child.class, &class_name).unwrap_or(false))
                    .to_lua(lua)
            },
        );
        methods.add_method("FindFirstChild", |lua, this, name: String| {
            this.ensure_not_destroyed()?;
            this.find_child(|child| child.name == name).to_lua(lua)
        });
        methods.add_method("FindFirstChildOfClass", |lua, this, class_name: String| {
            this.ensure_not_destroyed()?;
            this.find_child(|child| child.class == class_name)
                .to_lua(lua)
        });
        methods.add_method("FindFirstChildWhichIsA", |lua, this, class_name: String| {
            this.ensure_not_destroyed()?;
            this.find_child(|child| class_is_a(&child.class, &class_name).unwrap_or(false))
                .to_lua(lua)
        });
        methods.add_method("FindFirstDescendant", |lua, this, name: String| {
            this.ensure_not_destroyed()?;
            this.find_descendant(|child| child.name == name).to_lua(lua)
        });
        methods.add_method("IsA", |_, this, class_name: String| {
            this.ensure_not_destroyed()?;
            Ok(class_is_a(&this.class_name, class_name).unwrap_or(false))
        });
        methods.add_method("IsAncestorOf", |_, this, instance: Instance| {
            this.ensure_not_destroyed()?;
            Ok(instance
                .find_ancestor(|ancestor| ancestor.referent() == this.dom_ref)
                .is_some())
        });
        methods.add_method("IsDescendantOf", |_, this, instance: Instance| {
            this.ensure_not_destroyed()?;
            Ok(this
                .find_ancestor(|ancestor| ancestor.referent() == instance.dom_ref)
                .is_some())
        });
        // Here we add inheritance-like behavior for instances by creating
        // methods that are restricted to specific classnames / base classes
        data_model::add_methods(methods);
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
