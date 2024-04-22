use std::borrow::{Borrow, BorrowMut, Cow};

use rbx_dom_weak::types::{Variant as DomValue, VariantType as DomType};
use rbx_reflection::{ClassTag, DataType};

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
    let db = rbx_reflection_database::get();

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

    let mut class_name = Cow::Borrowed(instance_class);
    let mut class_info = None;

    while let Some(class) = db.classes.get(class_name.as_ref()) {
        if let Some(prop_definition) = class.properties.get(property_name) {
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
        } else if let Some(sup) = &class.superclass {
            // No property found, we should look at the superclass
            class_name = Cow::Borrowed(sup);
        } else {
            break;
        }
    }

    if let Some(class_info) = class_info.borrow_mut() {
        class_name = Cow::Borrowed(instance_class);
        while let Some(class) = db.classes.get(class_name.as_ref()) {
            if let Some(default) = class.default_properties.get(property_name) {
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
            } else if let Some(sup) = &class.superclass {
                // No default value found, we should look at the superclass
                class_name = Cow::Borrowed(sup);
            } else {
                break;
            }
        }
    }

    class_info
}

/**
    Checks if an instance class exists in the reflection database.
*/
pub fn class_exists(class_name: impl AsRef<str>) -> bool {
    let db = rbx_reflection_database::get();
    db.classes.contains_key(class_name.as_ref())
}

/**
    Checks if an instance class matches a given class or superclass, similar to
    [Instance::IsA](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
    from the Roblox standard library.

    Note that this function may return `None` if it encounters a class or superclass
    that does not exist in the currently known class reflection database.
*/
pub fn class_is_a(instance_class: impl AsRef<str>, class_name: impl AsRef<str>) -> Option<bool> {
    let mut instance_class = instance_class.as_ref();
    let class_name = class_name.as_ref();

    if class_name == "Instance" || instance_class == class_name {
        Some(true)
    } else {
        let db = rbx_reflection_database::get();

        while instance_class != class_name {
            let class_descriptor = db.classes.get(instance_class)?;
            if let Some(sup) = &class_descriptor.superclass {
                instance_class = sup.borrow();
            } else {
                return Some(false);
            }
        }

        Some(true)
    }
}

/**
    Checks if an instance class is a service.

    This is separate from [`class_is_a`] since services do not share a
    common base class, and are instead determined through reflection tags.

    Note that this function may return `None` if it encounters a class or superclass
    that does not exist in the currently known class reflection database.
*/
pub fn class_is_a_service(instance_class: impl AsRef<str>) -> Option<bool> {
    let mut instance_class = instance_class.as_ref();

    let db = rbx_reflection_database::get();

    loop {
        let class_descriptor = db.classes.get(instance_class)?;
        if class_descriptor.tags.contains(&ClassTag::Service) {
            return Some(true);
        } else if let Some(sup) = &class_descriptor.superclass {
            instance_class = sup.borrow();
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
}
