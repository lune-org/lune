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
pub fn instance_is_a(
    instance_class_name: impl AsRef<str>,
    class_name: impl AsRef<str>,
) -> Option<bool> {
    let instance_class_name = instance_class_name.as_ref();
    let class_name = class_name.as_ref();

    if class_name == "Instance" || instance_class_name == class_name {
        Some(true)
    } else {
        let db = rbx_reflection_database::get();

        let mut super_class_name = instance_class_name;
        while super_class_name != class_name {
            let class_descriptor = db.classes.get(super_class_name)?;
            if let Some(sup) = &class_descriptor.superclass {
                super_class_name = sup.borrow();
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
pub fn instance_is_a_service(class_name: impl AsRef<str>) -> Option<bool> {
    let mut class_name = class_name.as_ref();

    let db = rbx_reflection_database::get();

    loop {
        let class_descriptor = db.classes.get(class_name)?;
        if class_descriptor.tags.contains(&ClassTag::Service) {
            return Some(true);
        } else if let Some(sup) = &class_descriptor.superclass {
            class_name = sup.borrow();
        } else {
            break;
        }
    }

    Some(false)
}
