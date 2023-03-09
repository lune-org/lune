use std::borrow::Borrow;

use rbx_reflection::ClassTag;

/**
    Checks if an instance class matches a given class or superclass, similar to
    [Instance::IsA](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
    from the Roblox standard library.

    Note that this function may return `None` if it encounters a class or superclass
    that does not exist in the currently known class reflection database.
*/
#[allow(dead_code)]
pub fn instance_is_a(instance_class: impl AsRef<str>, class_name: impl AsRef<str>) -> Option<bool> {
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

    This is separate from [`instance_is_a`] since services do not share a
    common base class, and are instead determined through reflection tags.

    Note that this function may return `None` if it encounters a class or superclass
    that does not exist in the currently known class reflection database.
*/
pub fn instance_is_a_service(instance_class: impl AsRef<str>) -> Option<bool> {
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
        assert_eq!(instance_is_a("Part", "Part"), Some(true));
        assert_eq!(instance_is_a("Part", "BasePart"), Some(true));
        assert_eq!(instance_is_a("Part", "PVInstance"), Some(true));
        assert_eq!(instance_is_a("Part", "Instance"), Some(true));

        assert_eq!(instance_is_a("Workspace", "Workspace"), Some(true));
        assert_eq!(instance_is_a("Workspace", "Model"), Some(true));
        assert_eq!(instance_is_a("Workspace", "Instance"), Some(true));
    }

    #[test]
    fn is_a_class_invalid() {
        assert_eq!(instance_is_a("Part", "part"), Some(false));
        assert_eq!(instance_is_a("Part", "Base-Part"), Some(false));
        assert_eq!(instance_is_a("Part", "Model"), Some(false));
        assert_eq!(instance_is_a("Part", "Paart"), Some(false));

        assert_eq!(instance_is_a("Workspace", "Service"), Some(false));
        assert_eq!(instance_is_a("Workspace", "."), Some(false));
        assert_eq!(instance_is_a("Workspace", ""), Some(false));
    }

    #[test]
    fn is_a_service_valid() {
        assert_eq!(instance_is_a_service("Workspace"), Some(true));
        assert_eq!(instance_is_a_service("PhysicsService"), Some(true));
        assert_eq!(instance_is_a_service("ReplicatedFirst"), Some(true));
        assert_eq!(instance_is_a_service("CSGDictionaryService"), Some(true));
    }

    #[test]
    fn is_a_service_invalid() {
        assert_eq!(instance_is_a_service("Camera"), Some(false));
        assert_eq!(instance_is_a_service("Terrain"), Some(false));
        assert_eq!(instance_is_a_service("Work-space"), None);
        assert_eq!(instance_is_a_service("CSG Dictionary Service"), None);
    }
}
