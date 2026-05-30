#![allow(clippy::missing_panics_doc)]

use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    hash::{Hash, Hasher},
};

#[cfg(feature = "mlua")]
use mlua::prelude::*;

use rbx_dom_weak::{
    Instance as DomInstance, InstanceBuilder as DomInstanceBuilder, Ustr, WeakDom,
    types::{Attributes as DomAttributes, Ref as DomRef, Variant as DomValue},
    ustr,
};

#[cfg(feature = "mlua")]
use lune_utils::TableBuilder;

use crate::shared::instance::class_is_a;

#[cfg(feature = "mlua")]
use crate::{exports::LuaExportsTable, shared::instance::class_exists};

#[cfg(feature = "mlua")]
pub(crate) mod base;
#[cfg(feature = "mlua")]
pub(crate) mod data_model;
#[cfg(feature = "mlua")]
pub(crate) mod terrain;
#[cfg(feature = "mlua")]
pub(crate) mod workspace;

pub(crate) mod dom_registry;

#[cfg(feature = "mlua")]
pub mod registry;

use dom_registry::DomId;

const PROPERTY_NAME_ATTRIBUTES: &str = "Attributes";
const PROPERTY_NAME_TAGS: &str = "Tags";

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub(crate) dom_id: DomId,
    pub(crate) dom_ref: DomRef,
    pub(crate) class_name: Ustr,
}

impl Instance {
    /**
        Builds an `Instance` from a dom object that has already been
        borrowed out of its dom, without acquiring any new locks.

        This is the preferred constructor to use from inside a
        [`dom::with`] / [`dom::with_mut`] closure.
    */
    pub(crate) fn from_dom_instance(dom_id: DomId, dom_ref: DomRef, inst: &DomInstance) -> Self {
        Self {
            dom_id,
            dom_ref,
            class_name: inst.class,
        }
    }

    /**
        Creates a new `Instance` from a dom id and object ref.

        Panics if the instance does not exist in the given dom,
        or if the given dom object ref points to the dom root.
    */
    #[must_use]
    pub fn new(dom_id: DomId, dom_ref: DomRef) -> Self {
        Self::new_opt(dom_id, dom_ref).expect("Failed to find instance in document")
    }

    /**
        Creates a new `Instance` from a dom id and object ref, if the instance exists.

        Panics if the given dom object ref points to the dom root.
    */
    #[must_use]
    pub fn new_opt(dom_id: DomId, dom_ref: DomRef) -> Option<Self> {
        dom_registry::with(dom_id, |dom| {
            dom.get_by_ref(dom_ref).map(|instance| {
                assert!(
                    !(instance.referent() == dom.root_ref()),
                    "Instances can not be created from dom roots"
                );
                Self::from_dom_instance(dom_id, dom_ref, instance)
            })
        })
        .flatten()
    }

    /**
        Creates a new orphaned `Instance` with a given class name.

        An orphaned instance is an instance at the root of the shared default
        scratch dom used for manually created instances.
    */
    #[must_use]
    pub fn new_orphaned(class_name: impl AsRef<str>) -> Self {
        Self::new_in_dom(dom_registry::default_dom(), class_name)
    }

    /**
        Creates a new orphaned `Instance` with a given class name, directly
        inside the dom with the given id.

        Creating children inside their eventual parent's dom this way avoids
        a needless cross-dom transfer when they are later parented.
    */
    #[must_use]
    pub fn new_in_dom(dom_id: DomId, class_name: impl AsRef<str>) -> Self {
        let class_name = class_name.as_ref();
        let dom_ref = dom_registry::with_mut(dom_id, |dom| {
            let dom_root = dom.root_ref();
            dom.insert(dom_root, DomInstanceBuilder::new(class_name))
        })
        .expect("Failed to find dom to create instance in");
        Self {
            dom_id,
            dom_ref,
            class_name: ustr(class_name),
        }
    }

    /**
        Clones an instance to an external weak dom.

        This will place the instance as a child of the
        root of the weak dom, and return its referent.
    */
    pub fn clone_into_external_dom(self, external_dom: &mut WeakDom) -> DomRef {
        dom_registry::with(self.dom_id, |dom| {
            let cloned = dom.clone_into_external(self.dom_ref, external_dom);
            external_dom.transfer_within(cloned, external_dom.root_ref());
            cloned
        })
        .expect("Failed to find dom to clone instance from")
    }

    /**
        Clones multiple instances from a single dom to an external weak dom.

        This will place the instances as children of the
        root of the weak dom, and return their referents.
    */
    pub fn clone_multiple_into_external_dom(
        dom_id: DomId,
        referents: &[DomRef],
        external_dom: &mut WeakDom,
    ) -> Vec<DomRef> {
        dom_registry::with(dom_id, |dom| {
            let cloned = dom.clone_multiple_into_external(referents, external_dom);
            for referent in &cloned {
                external_dom.transfer_within(*referent, external_dom.root_ref());
            }
            cloned
        })
        .unwrap_or_default()
    }

    /**
        Clones the instance and all of its descendants, and orphans it.

        The clone is placed at the root of the same dom as the original.

        ### See Also
        * [`Clone`](https://create.roblox.com/docs/reference/engine/classes/Instance#Clone)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn clone_instance(&self) -> Self {
        let dom_id = self.dom_id;
        let new_ref = dom_registry::with_mut(dom_id, |dom| dom.clone_within(self.dom_ref))
            .expect("Failed to find dom to clone instance in");
        let new_inst = Self::new(dom_id, new_ref);
        new_inst.set_parent(None);
        new_inst
    }

    /**
        Destroys the instance, removing it completely
        from its dom with no way of recovering it.

        If destroying the instance leaves its dom empty (e.g. destroying the
        root `DataModel` of a parsed place), the entire dom is dropped, freeing
        all of its memory.

        Returns `true` if destroyed successfully, `false` if already destroyed.

        ### See Also
        * [`Destroy`](https://create.roblox.com/docs/reference/engine/classes/Instance#Destroy)
          on the Roblox Developer Hub
    */
    pub fn destroy(&mut self) -> bool {
        if self.is_destroyed() {
            false
        } else {
            dom_registry::with_mut(self.dom_id, |dom| dom.destroy(self.dom_ref));
            dom_registry::drop_if_empty(self.dom_id);
            true
        }
    }

    fn is_destroyed(&self) -> bool {
        // NOTE: This property can not be cached since instance references
        // other than this one may have destroyed this one, and we don't
        // keep track of all current instance reference structs
        dom_registry::with(self.dom_id, |dom| dom.get_by_ref(self.dom_ref).is_none())
            .unwrap_or(true)
    }

    /**
        Destroys all child instances.

        ### See Also
        * [`Instance::Destroy`] for more info about what happens when an instance gets destroyed
        * [`ClearAllChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#ClearAllChildren)
          on the Roblox Developer Hub
    */
    pub fn clear_all_children(&mut self) {
        dom_registry::with_mut(self.dom_id, |dom| {
            if let Some(instance) = dom.get_by_ref(self.dom_ref) {
                let child_refs = instance.children().to_vec();
                for child_ref in child_refs {
                    dom.destroy(child_ref);
                }
            }
        });
    }

    /**
        Checks if the instance matches or inherits a given class name.

        ### See Also
        * [`IsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
          on the Roblox Developer Hub
    */
    pub fn is_a(&self, class_name: impl AsRef<str>) -> bool {
        class_is_a(self.class_name, class_name).unwrap_or(false)
    }

    /**
        Gets the class name of the instance.

        This will return the correct class name even if the instance has been destroyed.

        ### See Also
        * [`ClassName`](https://create.roblox.com/docs/reference/engine/classes/Instance#ClassName)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_class_name(&self) -> &str {
        self.class_name.as_str()
    }

    /**
        Gets the name of the instance, if it exists.

        ### See Also
        * [`Name`](https://create.roblox.com/docs/reference/engine/classes/Instance#Name)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_name(&self) -> String {
        dom_registry::with(self.dom_id, |dom| {
            dom.get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .name
                .clone()
        })
        .expect("Failed to find dom for instance")
    }

    /**
        Sets the name of the instance, if it exists.

        ### See Also
        * [`Name`](https://create.roblox.com/docs/reference/engine/classes/Instance#Name)
          on the Roblox Developer Hub
    */
    pub fn set_name(&self, name: impl Into<String>) {
        dom_registry::with_mut(self.dom_id, |dom| {
            dom.get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document")
                .name = name.into();
        });
    }

    /**
        Gets the parent of the instance, if it exists.

        ### See Also
        * [`Parent`](https://create.roblox.com/docs/reference/engine/classes/Instance#Parent)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_parent(&self) -> Option<Instance> {
        dom_registry::with(self.dom_id, |dom| {
            let parent_ref = dom.get_by_ref(self.dom_ref)?.parent();
            if parent_ref == dom.root_ref() {
                None
            } else {
                dom.get_by_ref(parent_ref)
                    .map(|parent| Self::from_dom_instance(self.dom_id, parent_ref, parent))
            }
        })
        .flatten()
    }

    /**
        Sets the parent of the instance, if it exists.

        If the provided parent is [`None`] the instance will become orphaned
        within its current dom.

        If the parent is in a *different* dom, the instance and all of its
        descendants are transferred into the parent's dom. The transferred
        referents are returned so that callers (such as the lua `Parent`
        setter) can update any cached userdata and `dom_id`s accordingly.

        ### See Also
        * [`Parent`](https://create.roblox.com/docs/reference/engine/classes/Instance#Parent)
          on the Roblox Developer Hub
    */
    #[allow(clippy::must_use_candidate)]
    pub fn set_parent(&self, parent: Option<Instance>) -> Vec<DomRef> {
        let parent = parent.map(|p| (p.dom_id, p.dom_ref));
        dom_registry::reparent(self.dom_id, self.dom_ref, parent)
    }

    /**
        Gets a property for the instance, if it exists.
    */
    pub fn get_property(&self, name: impl AsRef<str>) -> Option<DomValue> {
        dom_registry::with(self.dom_id, |dom| {
            dom.get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .properties
                .get(&ustr(name.as_ref()))
                .cloned()
        })
        .flatten()
    }

    /**
        Sets a property for the instance.

        Note that setting a property here will not fail even if the
        property does not actually exist for the instance class.
    */
    pub fn set_property(&self, name: impl AsRef<str>, value: DomValue) {
        dom_registry::with_mut(self.dom_id, |dom| {
            dom.get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document")
                .properties
                .insert(ustr(name.as_ref()), value);
        });
    }

    /**
        Gets an attribute for the instance, if it exists.

        ### See Also
        * [`GetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttribute)
          on the Roblox Developer Hub
    */
    pub fn get_attribute(&self, name: impl AsRef<str>) -> Option<DomValue> {
        dom_registry::with(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.get(name.as_ref()).cloned()
            } else {
                None
            }
        })
        .flatten()
    }

    /**
        Gets all known attributes for the instance.

        ### See Also
        * [`GetAttributes`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttributes)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_attributes(&self) -> BTreeMap<String, DomValue> {
        dom_registry::with(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.clone().into_iter().collect()
            } else {
                BTreeMap::new()
            }
        })
        .unwrap_or_default()
    }

    /**
        Sets an attribute for the instance.

        ### See Also
        * [`SetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#SetAttribute)
          on the Roblox Developer Hub
    */
    pub fn set_attribute(&self, name: impl AsRef<str>, value: DomValue) {
        dom_registry::with_mut(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            // NOTE: Attributes do not support integers, only floats
            let value = match value {
                DomValue::Int32(i) => DomValue::Float32(i as f32),
                DomValue::Int64(i) => DomValue::Float64(i as f64),
                value => value,
            };
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get_mut(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.insert(name.as_ref().to_string(), value);
            } else {
                let mut attributes = DomAttributes::new();
                attributes.insert(name.as_ref().to_string(), value);
                inst.properties.insert(
                    ustr(PROPERTY_NAME_ATTRIBUTES),
                    DomValue::Attributes(attributes),
                );
            }
        });
    }

    /**
        Removes an attribute from the instance.

        Note that this does not have an equivalent in the Roblox engine API,
        but separating this from `set_attribute` lets `set_attribute` be more
        ergonomic and not require an `Option<DomValue>` for the value argument.
        The equivalent in the Roblox engine API would be `instance:SetAttribute(name, nil)`.
    */
    pub fn remove_attribute(&self, name: impl AsRef<str>) {
        dom_registry::with_mut(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Attributes(attributes)) =
                inst.properties.get_mut(&ustr(PROPERTY_NAME_ATTRIBUTES))
            {
                attributes.remove(name.as_ref());
                if attributes.is_empty() {
                    inst.properties.remove(&ustr(PROPERTY_NAME_ATTRIBUTES));
                }
            }
        });
    }

    /**
        Adds a tag to the instance.

        ### See Also
        * [`AddTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#AddTag)
          on the Roblox Developer Hub
    */
    pub fn add_tag(&self, name: impl AsRef<str>) {
        dom_registry::with_mut(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get_mut(&ustr(PROPERTY_NAME_TAGS)) {
                tags.push(name.as_ref());
            } else {
                inst.properties.insert(
                    ustr(PROPERTY_NAME_TAGS),
                    DomValue::Tags(vec![name.as_ref().to_string()].into()),
                );
            }
        });
    }

    /**
        Gets all current tags for the instance.

        ### See Also
        * [`GetTags`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#GetTags)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_tags(&self) -> Vec<String> {
        dom_registry::with(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get(&ustr(PROPERTY_NAME_TAGS)) {
                tags.iter().map(ToString::to_string).collect()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default()
    }

    /**
        Checks if the instance has a specific tag.

        ### See Also
        * [`HasTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#HasTag)
          on the Roblox Developer Hub
    */
    pub fn has_tag(&self, name: impl AsRef<str>) -> bool {
        dom_registry::with(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get(&ustr(PROPERTY_NAME_TAGS)) {
                let name = name.as_ref();
                tags.iter().any(|tag| tag == name)
            } else {
                false
            }
        })
        .unwrap_or(false)
    }

    /**
        Removes a tag from the instance.

        ### See Also
        * [`RemoveTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#RemoveTag)
          on the Roblox Developer Hub
    */
    pub fn remove_tag(&self, name: impl AsRef<str>) {
        dom_registry::with_mut(self.dom_id, |dom| {
            let inst = dom
                .get_by_ref_mut(self.dom_ref)
                .expect("Failed to find instance in document");
            if let Some(DomValue::Tags(tags)) = inst.properties.get_mut(&ustr(PROPERTY_NAME_TAGS)) {
                let name = name.as_ref();
                let mut new_tags = tags.iter().map(ToString::to_string).collect::<Vec<_>>();
                new_tags.retain(|tag| tag != name);
                inst.properties
                    .insert(ustr(PROPERTY_NAME_TAGS), DomValue::Tags(new_tags.into()));
            }
        });
    }

    /**
        Gets all of the current children of this `Instance`.

        Note that this is a somewhat expensive operation and that other
        operations using weak dom referents should be preferred if possible.

        ### See Also
        * [`GetChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetChildren)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_children(&self) -> Vec<Instance> {
        dom_registry::with(self.dom_id, |dom| {
            let instance = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document");
            instance
                .children()
                .iter()
                .filter_map(|child_ref| {
                    dom.get_by_ref(*child_ref)
                        .map(|child| Self::from_dom_instance(self.dom_id, *child_ref, child))
                })
                .collect()
        })
        .unwrap_or_default()
    }

    /**
        Gets all of the current descendants of this `Instance` using a breadth-first search.

        Note that this is a somewhat expensive operation and that other
        operations using weak dom referents should be preferred if possible.

        ### See Also
        * [`GetDescendants`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetDescendants)
          on the Roblox Developer Hub
    */
    #[must_use]
    pub fn get_descendants(&self) -> Vec<Instance> {
        dom_registry::with(self.dom_id, |dom| {
            let mut descendants = Vec::new();
            let mut queue = VecDeque::from_iter(
                dom.get_by_ref(self.dom_ref)
                    .expect("Failed to find instance in document")
                    .children(),
            );

            while let Some(queue_ref) = queue.pop_front() {
                if let Some(queue_inst) = dom.get_by_ref(*queue_ref) {
                    descendants.push(Self::from_dom_instance(self.dom_id, *queue_ref, queue_inst));
                    for queue_ref_inner in queue_inst.children().iter().rev() {
                        queue.push_back(queue_ref_inner);
                    }
                }
            }

            descendants
        })
        .unwrap_or_default()
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
    #[must_use]
    pub fn get_full_name(&self) -> String {
        dom_registry::with(self.dom_id, |dom| {
            let dom_root = dom.root_ref();

            let mut parts = Vec::new();
            let mut instance_ref = self.dom_ref;

            while let Some(instance) = dom.get_by_ref(instance_ref) {
                if instance_ref != dom_root && instance.class != "DataModel" {
                    instance_ref = instance.parent();
                    parts.push(instance.name.clone());
                } else {
                    break;
                }
            }

            parts.reverse();
            parts.join(".")
        })
        .unwrap_or_default()
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
        dom_registry::with(self.dom_id, |dom| {
            let children = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .children();

            children.iter().find_map(|child_ref| {
                let child_inst = dom.get_by_ref(*child_ref)?;
                if predicate(child_inst) {
                    Some(Self::from_dom_instance(self.dom_id, *child_ref, child_inst))
                } else {
                    None
                }
            })
        })
        .flatten()
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
        dom_registry::with(self.dom_id, |dom| {
            let mut ancestor_ref = dom
                .get_by_ref(self.dom_ref)
                .expect("Failed to find instance in document")
                .parent();

            while let Some(ancestor) = dom.get_by_ref(ancestor_ref) {
                if predicate(ancestor) {
                    return Some(Self::from_dom_instance(self.dom_id, ancestor_ref, ancestor));
                }
                ancestor_ref = ancestor.parent();
            }

            None
        })
        .flatten()
    }

    /**
        Finds a descendant of the instance using the given
        predicate callback and a breadth-first search.

        ### See Also
        * [`FindFirstDescendant`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstDescendant) on the Roblox Developer Hub
    */
    pub fn find_descendant<F>(&self, predicate: F) -> Option<Instance>
    where
        F: Fn(&DomInstance) -> bool,
    {
        dom_registry::with(self.dom_id, |dom| {
            let mut queue = VecDeque::from_iter(
                dom.get_by_ref(self.dom_ref)
                    .expect("Failed to find instance in document")
                    .children(),
            );

            while let Some(queue_ref) = queue.pop_front() {
                if let Some(queue_item) = dom.get_by_ref(*queue_ref) {
                    if predicate(queue_item) {
                        return Some(Self::from_dom_instance(self.dom_id, *queue_ref, queue_item));
                    }
                    queue.extend(queue_item.children());
                }
            }

            None
        })
        .flatten()
    }
}

#[cfg(feature = "mlua")]
impl LuaExportsTable for Instance {
    const EXPORT_NAME: &'static str = "Instance";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let instance_new = |lua: &Lua, class_name: String| {
            if class_exists(&class_name) {
                instance_to_lua(lua, Instance::new_orphaned(class_name))
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Failed to create Instance - '{class_name}' is not a valid class name",
                )))
            }
        };

        TableBuilder::new(lua)?
            .with_function("new", instance_new)?
            .build_readonly()
    }
}

/*
    Userdata interning

    To preserve instance identity from a lua perspective - so that two
    references to the same underlying instance are the *same* userdata, and
    therefore work correctly as table keys and with rawequal - we keep a single
    canonical userdata per `(dom_id, dom_ref)` pair in a per-lua cache.

    The cache table has weak values, so userdata that is no longer referenced
    anywhere in lua can still be collected and will simply be recreated on next
    access (which is fine, since identity only needs to hold while a reference
    is alive).
*/
#[cfg(feature = "mlua")]
const INSTANCE_CACHE_KEY: &str = "__lune_roblox_instance_cache";

#[cfg(feature = "mlua")]
fn instance_cache(lua: &Lua) -> LuaResult<LuaTable> {
    if let Ok(cache) = lua.named_registry_value::<LuaTable>(INSTANCE_CACHE_KEY) {
        return Ok(cache);
    }
    let cache = lua.create_table()?;
    let meta = lua.create_table()?;
    meta.set("__mode", "v")?;
    cache.set_metatable(Some(meta))?;
    lua.set_named_registry_value(INSTANCE_CACHE_KEY, &cache)?;
    Ok(cache)
}

#[cfg(feature = "mlua")]
fn instance_cache_key(inst: Instance) -> String {
    format!("{}:{}", inst.dom_id, inst.dom_ref)
}

/**
    Converts an instance into its canonical lua userdata, creating and caching
    it the first time and returning the same userdata on subsequent calls.

    # Errors

    Errors if creating the userdata or accessing the interning cache fails.
*/
#[cfg(feature = "mlua")]
pub fn instance_to_lua(lua: &Lua, inst: Instance) -> LuaResult<LuaValue> {
    let cache = instance_cache(lua)?;
    let key = instance_cache_key(inst);
    if let Ok(LuaValue::UserData(existing)) = cache.get::<LuaValue>(key.as_str()) {
        return Ok(LuaValue::UserData(existing));
    }
    let userdata = lua.create_userdata(inst)?;
    cache.set(key, &userdata)?;
    Ok(LuaValue::UserData(userdata))
}

/**
    Converts an optional instance into lua, returning [`LuaValue::Nil`] for [`None`].

    # Errors

    Errors if creating the userdata or accessing the interning cache fails.
*/
#[cfg(feature = "mlua")]
pub fn opt_instance_to_lua(lua: &Lua, inst: Option<Instance>) -> LuaResult<LuaValue> {
    match inst {
        Some(inst) => instance_to_lua(lua, inst),
        None => Ok(LuaValue::Nil),
    }
}

/**
    Converts a list of instances into a lua array table of canonical userdata.

    # Errors

    Errors if creating the table or any of the userdata fails.
*/
#[cfg(feature = "mlua")]
pub fn instances_to_lua(lua: &Lua, instances: Vec<Instance>) -> LuaResult<LuaValue> {
    let tab = lua.create_table_with_capacity(instances.len(), 0)?;
    for inst in instances {
        tab.push(instance_to_lua(lua, inst)?)?;
    }
    Ok(LuaValue::Table(tab))
}

/**
    Updates the interning cache after a cross-dom transfer.

    The moved referents keep their values but now live in `new_dom_id`, so we
    re-key any cached userdata and update the `dom_id` stored inside it.
*/
#[cfg(feature = "mlua")]
pub(crate) fn rekey_cache_after_transfer(
    lua: &Lua,
    old_dom_id: DomId,
    new_dom_id: DomId,
    moved: &[DomRef],
) -> LuaResult<()> {
    if old_dom_id == new_dom_id {
        return Ok(());
    }
    let cache = instance_cache(lua)?;
    for &moved_ref in moved {
        let old_key = format!("{old_dom_id}:{moved_ref}");
        if let Ok(LuaValue::UserData(ud)) = cache.get::<LuaValue>(old_key.as_str()) {
            if let Ok(mut inst) = ud.borrow_mut::<Instance>() {
                inst.dom_id = new_dom_id;
            }
            let new_key = format!("{new_dom_id}:{moved_ref}");
            cache.set(new_key, &ud)?;
            cache.set(old_key, LuaValue::Nil)?;
        }
    }
    Ok(())
}

/*
    Here we add inheritance-like behavior for instances by creating
    fields that are restricted to specific classnames / base classes

    Note that we should try to be conservative with how many classes
    and methods we support here - we should only implement methods that
    are necessary for modifying the dom and / or having ergonomic access
    to the dom, not try to replicate Roblox engine behavior of instances

    If a user wants to replicate Roblox engine behavior, they can use the
    instance registry, and register properties + methods from the lua side
*/
#[cfg(feature = "mlua")]
impl LuaUserData for Instance {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        data_model::add_fields(fields);
        workspace::add_fields(fields);
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        base::add_methods(methods);
        data_model::add_methods(methods);
        terrain::add_methods(methods);
    }
}

impl Hash for Instance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dom_id.hash(state);
        self.dom_ref.hash(state);
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            if self.is_destroyed() {
                "<<DESTROYED>>".to_string()
            } else {
                self.get_name()
            }
        )
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.dom_id == other.dom_id && self.dom_ref == other.dom_ref
    }
}

impl From<Instance> for DomRef {
    fn from(value: Instance) -> Self {
        value.dom_ref
    }
}
