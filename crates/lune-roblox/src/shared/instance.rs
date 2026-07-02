#![allow(dead_code)]

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use rbx_dom_weak::types::{Variant as DomValue, VariantType as DomType};
use rbx_reflection::{ClassTag, DataType};
use thiserror::Error;

/**
    A class defined at runtime that the reflection database does not know about.

    These are registered through [`register_custom_class`] and let consumers
    implement classes and services (such as `FileSystemService`) that only exist
    in specific Roblox builds, in the same vein as `implementMethod` /
    `implementProperty`.
*/
#[derive(Debug, Clone)]
struct CustomClass {
    superclass: String,
    is_service: bool,
}

fn custom_classes() -> &'static RwLock<HashMap<String, CustomClass>> {
    static CUSTOM_CLASSES: LazyLock<RwLock<HashMap<String, CustomClass>>> =
        LazyLock::new(|| RwLock::new(HashMap::new()));
    &CUSTOM_CLASSES
}

/**
    Error that may occur when registering a custom class through
    [`register_custom_class`].
*/
#[derive(Debug, Clone, Error)]
pub enum CustomClassError {
    #[error("class '{0}' already exists and cannot be redefined")]
    AlreadyExists(String),
    #[error("superclass '{0}' is not a valid class name")]
    InvalidSuperclass(String),
}

/**
    Registers a custom class that the reflection database does not know about.

    The `superclass` must be a valid class name - either a built-in class known
    to the reflection database, or another custom class registered beforehand.

    # Errors

    - If a class with the given name already exists (built-in or custom).
    - If the given superclass is not a valid class name.

    # Panics

    - If the custom class registry lock has been poisoned.
*/
pub fn register_custom_class(
    class_name: &str,
    superclass: &str,
    is_service: bool,
) -> Result<(), CustomClassError> {
    if class_exists(class_name) {
        return Err(CustomClassError::AlreadyExists(class_name.to_string()));
    }
    if !class_exists(superclass) {
        return Err(CustomClassError::InvalidSuperclass(superclass.to_string()));
    }

    custom_classes().write().unwrap().insert(
        class_name.to_string(),
        CustomClass {
            superclass: superclass.to_string(),
            is_service,
        },
    );

    Ok(())
}

/**
    Resolves the immediate superclass of a class, consulting both the built-in
    reflection database and the custom class registry.

    Returns `None` if the class is unknown or is a root class (such as
    `Instance`) with no superclass.
*/
fn superclass_of(class_name: &str) -> Option<String> {
    let db = rbx_reflection_database::get().unwrap();
    if let Some(class) = db.classes.get(class_name) {
        return class.superclass.as_ref().map(ToString::to_string);
    }
    custom_classes()
        .read()
        .unwrap()
        .get(class_name)
        .map(|c| c.superclass.clone())
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PropertyInfo {
    pub enum_name: Option<Cow<'static, str>>,
    pub enum_default: Option<u32>,
    pub value_type: Option<DomType>,
    pub value_default: Option<&'static DomValue>,
}

/**
    Finds the info of a property of the given class.

    This will also check superclasses if the property
    was not directly found for the given class.

    Returns `None` if the class or property does not exist.
*/
pub(crate) fn find_property_info(
    instance_class: impl AsRef<str>,
    property_name: impl AsRef<str>,
) -> Option<PropertyInfo> {
    let db = rbx_reflection_database::get().unwrap();

    let instance_class = instance_class.as_ref();
    let property_name = property_name.as_ref();

    // Attributes and tags are *technically* properties but we don't
    // want to treat them as such when looking up property info, any
    // reading or modification of these should always be explicit
    if matches!(property_name, "Attributes" | "Tags") {
        return None;
    }

    // FUTURE: We can probably cache the result of calling this
    // function, if property access is being used in a tight loop
    // in a build step or something similar then it would be beneficial

    // Walk the (custom-aware) class hierarchy looking for the property
    // definition. Custom classes carry no reflection properties, so links in
    // the chain that are not present in the reflection database are skipped,
    // while still allowing inherited properties from a built-in superclass.
    let mut class_info = None;
    let mut current = Some(instance_class.to_string());
    while let Some(class_name) = current {
        if let Some(class) = db.classes.get(class_name.as_str())
            && let Some(prop_definition) = class.properties.get(property_name)
        {
            /*
                We found a property, create a property info containing name/type

                Note that we might have found the property in the
                base class but the default value can be part of
                some separate class, it will be checked below
            */
            class_info = Some(match &prop_definition.data_type {
                DataType::Enum(enum_name) => PropertyInfo {
                    enum_name: Some(Cow::Borrowed(enum_name)),
                    ..Default::default()
                },
                DataType::Value(value_type) => PropertyInfo {
                    value_type: Some(*value_type),
                    ..Default::default()
                },
                _ => PropertyInfo::default(),
            });
            break;
        }
        current = superclass_of(&class_name);
    }

    if let Some(class_info) = class_info.as_mut() {
        let mut current = Some(instance_class.to_string());
        while let Some(class_name) = current {
            if let Some(class) = db.classes.get(class_name.as_str())
                && let Some(default) = class.default_properties.get(property_name)
            {
                // We found a default value, map it to a more useful value for us
                if class_info.enum_name.is_some() {
                    class_info.enum_default = match default {
                        DomValue::Enum(enum_default) => Some(enum_default.to_u32()),
                        _ => None,
                    };
                } else if class_info.value_type.is_some() {
                    class_info.value_default = Some(default);
                }
                break;
            }
            current = superclass_of(&class_name);
        }
    }

    class_info
}

/**
    Checks if an instance class exists in the reflection database, or has been
    registered as a custom class through [`register_custom_class`].
*/
pub fn class_exists(class_name: impl AsRef<str>) -> bool {
    let class_name = class_name.as_ref();
    let db = rbx_reflection_database::get().unwrap();
    db.classes.contains_key(class_name) || custom_classes().read().unwrap().contains_key(class_name)
}

/**
    Gets the class name chain for a given class name.

    The chain starts with the given class name and ends with the root class,
    consulting both the reflection database and any registered custom classes.

    If the class name is not known at all the chain will contain just the given
    class name.
*/
#[must_use]
pub fn class_name_chain(class_name: &str) -> Vec<String> {
    let mut list = vec![class_name.to_string()];
    let mut current = class_name.to_string();
    while let Some(sup) = superclass_of(&current) {
        list.push(sup.clone());
        current = sup;
    }
    list
}

/**
    Checks if an instance class matches a given class or superclass, similar to
    [Instance::IsA](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
    from the Roblox standard library.

    Note that this function may return `None` if it encounters a class or superclass
    that does not exist in the currently known class reflection database.
*/
pub fn class_is_a(instance_class: impl AsRef<str>, class_name: impl AsRef<str>) -> Option<bool> {
    let class_name = class_name.as_ref();
    let mut current = instance_class.as_ref().to_string();

    if class_name == "Instance" || current == class_name {
        return Some(true);
    }

    while current != class_name {
        // Bail out (with None) if we encounter a class we know nothing about
        if !class_exists(&current) {
            return None;
        }
        match superclass_of(&current) {
            Some(sup) => current = sup,
            None => return Some(false),
        }
    }

    Some(true)
}

/**
    Checks if an instance class is a service.

    This is separate from [`class_is_a`] since services do not share a
    common base class, and are instead determined through reflection tags.

    Note that this function may return `None` if it encounters a class or superclass
    that does not exist in the currently known class reflection database.
*/
pub fn class_is_a_service(instance_class: impl AsRef<str>) -> Option<bool> {
    let mut current = instance_class.as_ref().to_string();

    let db = rbx_reflection_database::get().unwrap();

    loop {
        // A custom class may be flagged as a service directly, otherwise we
        // keep walking up its superclass chain like any built-in class
        if let Some(custom) = custom_classes().read().unwrap().get(&current).cloned() {
            if custom.is_service {
                return Some(true);
            }
            current = custom.superclass;
            continue;
        }

        let class_descriptor = db.classes.get(current.as_str())?;
        if class_descriptor.tags.contains(&ClassTag::Service) {
            return Some(true);
        } else if let Some(sup) = &class_descriptor.superclass {
            current = sup.to_string();
        } else {
            break;
        }
    }

    Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_a_class_valid() {
        assert_eq!(class_is_a("Part", "Part"), Some(true));
        assert_eq!(class_is_a("Part", "BasePart"), Some(true));
        assert_eq!(class_is_a("Part", "PVInstance"), Some(true));
        assert_eq!(class_is_a("Part", "Instance"), Some(true));

        assert_eq!(class_is_a("Workspace", "Workspace"), Some(true));
        assert_eq!(class_is_a("Workspace", "Model"), Some(true));
        assert_eq!(class_is_a("Workspace", "Instance"), Some(true));
    }

    #[test]
    fn is_a_class_invalid() {
        assert_eq!(class_is_a("Part", "part"), Some(false));
        assert_eq!(class_is_a("Part", "Base-Part"), Some(false));
        assert_eq!(class_is_a("Part", "Model"), Some(false));
        assert_eq!(class_is_a("Part", "Paart"), Some(false));

        assert_eq!(class_is_a("Workspace", "Service"), Some(false));
        assert_eq!(class_is_a("Workspace", "."), Some(false));
        assert_eq!(class_is_a("Workspace", ""), Some(false));
    }

    #[test]
    fn is_a_service_valid() {
        assert_eq!(class_is_a_service("Workspace"), Some(true));
        assert_eq!(class_is_a_service("PhysicsService"), Some(true));
        assert_eq!(class_is_a_service("ReplicatedFirst"), Some(true));
        assert_eq!(class_is_a_service("CSGDictionaryService"), Some(true));
    }

    #[test]
    fn is_a_service_invalid() {
        assert_eq!(class_is_a_service("Camera"), Some(false));
        assert_eq!(class_is_a_service("Terrain"), Some(false));
        assert_eq!(class_is_a_service("Work-space"), None);
        assert_eq!(class_is_a_service("CSG Dictionary Service"), None);
    }

    // NOTE: The custom class registry is process-global, so each test below
    // uses unique class names to stay independent of the others.

    #[test]
    fn custom_class_register_and_exists() {
        assert!(!class_exists("CustomReg"));
        register_custom_class("CustomReg", "Instance", false).unwrap();

        assert!(class_exists("CustomReg"));
        assert_eq!(class_is_a("CustomReg", "CustomReg"), Some(true));
        assert_eq!(class_is_a("CustomReg", "Instance"), Some(true));
        assert_eq!(class_is_a("CustomReg", "Part"), Some(false));
        assert_eq!(class_is_a_service("CustomReg"), Some(false));

        // Inherited (Instance) properties should still resolve, while
        // unknown ones should not
        assert!(find_property_info("CustomReg", "Archivable").is_some());
        assert!(find_property_info("CustomReg", "DefinitelyNotAProperty").is_none());
    }

    #[test]
    fn custom_service_is_service() {
        register_custom_class("CustomSvc", "Instance", true).unwrap();

        assert!(class_exists("CustomSvc"));
        assert_eq!(class_is_a_service("CustomSvc"), Some(true));
        assert_eq!(class_is_a("CustomSvc", "Instance"), Some(true));
    }

    #[test]
    fn custom_class_register_errors() {
        // A built-in class can not be redefined
        assert!(matches!(
            register_custom_class("Part", "Instance", false),
            Err(CustomClassError::AlreadyExists(_))
        ));

        // A custom class can not be registered twice
        register_custom_class("CustomDup", "Instance", false).unwrap();
        assert!(matches!(
            register_custom_class("CustomDup", "Instance", false),
            Err(CustomClassError::AlreadyExists(_))
        ));

        // The superclass must be a valid class name
        assert!(matches!(
            register_custom_class("CustomBadSuper", "NotARealClass", false),
            Err(CustomClassError::InvalidSuperclass(_))
        ));
    }

    #[test]
    fn custom_class_inheritance() {
        register_custom_class("CustomBase", "Instance", false).unwrap();
        register_custom_class("CustomDerived", "CustomBase", false).unwrap();

        assert_eq!(class_is_a("CustomDerived", "CustomBase"), Some(true));
        assert_eq!(class_is_a("CustomDerived", "Instance"), Some(true));
        assert_eq!(class_is_a("CustomDerived", "Part"), Some(false));

        // The chain should begin with the custom classes and then continue up
        // the built-in hierarchy (which roots above `Instance`)
        let chain = class_name_chain("CustomDerived");
        assert_eq!(&chain[..3], &["CustomDerived", "CustomBase", "Instance"]);
    }

    #[test]
    fn class_name_chain_unknown_is_singleton() {
        // Unknown classes must not panic - they yield a single-element chain
        assert_eq!(
            class_name_chain("TotallyUnknownClass"),
            vec!["TotallyUnknownClass"]
        );
    }
}
