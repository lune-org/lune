use mlua::prelude::*;

use rbx_dom_weak::types::Variant as DomValue;

use crate::instance::Instance;

use super::instance::class_is_a;

pub(crate) fn add_class_restricted_getter<'lua, F: LuaUserDataFields<'lua, Instance>, R, G>(
    fields: &mut F,
    class_name: &'static str,
    field_name: &'static str,
    field_getter: G,
) where
    R: IntoLua<'lua>,
    G: 'static + Fn(&'lua Lua, &Instance) -> LuaResult<R>,
{
    fields.add_field_method_get(field_name, move |lua, this| {
        if class_is_a(this.get_class_name(), class_name).unwrap_or(false) {
            field_getter(lua, this)
        } else {
            Err(LuaError::RuntimeError(format!(
                "{} is not a valid member of {}",
                field_name, class_name
            )))
        }
    });
}

#[allow(dead_code)]
pub(crate) fn add_class_restricted_setter<'lua, F: LuaUserDataFields<'lua, Instance>, A, G>(
    fields: &mut F,
    class_name: &'static str,
    field_name: &'static str,
    field_getter: G,
) where
    A: FromLua<'lua>,
    G: 'static + Fn(&'lua Lua, &Instance, A) -> LuaResult<()>,
{
    fields.add_field_method_set(field_name, move |lua, this, value| {
        if class_is_a(this.get_class_name(), class_name).unwrap_or(false) {
            field_getter(lua, this, value)
        } else {
            Err(LuaError::RuntimeError(format!(
                "{} is not a valid member of {}",
                field_name, class_name
            )))
        }
    });
}

pub(crate) fn add_class_restricted_method<'lua, M: LuaUserDataMethods<'lua, Instance>, A, R, F>(
    methods: &mut M,
    class_name: &'static str,
    method_name: &'static str,
    method: F,
) where
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    F: 'static + Fn(&'lua Lua, &Instance, A) -> LuaResult<R>,
{
    methods.add_method(method_name, move |lua, this, args| {
        if class_is_a(this.get_class_name(), class_name).unwrap_or(false) {
            method(lua, this, args)
        } else {
            Err(LuaError::RuntimeError(format!(
                "{} is not a valid member of {}",
                method_name, class_name
            )))
        }
    });
}

#[allow(dead_code)]
pub(crate) fn add_class_restricted_method_mut<
    'lua,
    M: LuaUserDataMethods<'lua, Instance>,
    A,
    R,
    F,
>(
    methods: &mut M,
    class_name: &'static str,
    method_name: &'static str,
    method: F,
) where
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    F: 'static + Fn(&'lua Lua, &mut Instance, A) -> LuaResult<R>,
{
    methods.add_method_mut(method_name, move |lua, this, args| {
        if class_is_a(this.get_class_name(), class_name).unwrap_or(false) {
            method(lua, this, args)
        } else {
            Err(LuaError::RuntimeError(format!(
                "{} is not a valid member of {}",
                method_name, class_name
            )))
        }
    });
}

/**
    Gets or creates the instance child with the given reference prop name and class name.

    Note that the class name here must be an exact match, it is not checked using IsA.

    The instance may be in one of several states but this function will guarantee that the
    property reference is correct and that the instance exists after it has been called:

    1. Instance exists as property ref - just return it
    2. Instance exists under workspace but not as a property ref - save it and return it
    3. Instance does not exist - create it, save it, then return it
*/
pub(crate) fn get_or_create_property_ref_instance(
    this: &Instance,
    prop_name: &'static str,
    class_name: &'static str,
) -> LuaResult<Instance> {
    if let Some(DomValue::Ref(inst_ref)) = this.get_property(prop_name) {
        if let Some(inst) = Instance::new_opt(inst_ref) {
            return Ok(inst);
        }
    }
    if let Some(inst) = this.find_child(|child| child.class == class_name) {
        this.set_property(prop_name, DomValue::Ref(inst.dom_ref));
        Ok(inst)
    } else {
        let inst = Instance::new_orphaned(class_name);
        inst.set_parent(Some(this.clone()));
        this.set_property(prop_name, DomValue::Ref(inst.dom_ref));
        Ok(inst)
    }
}
