use core::fmt;

use mlua::prelude::*;

use lune_utils::TableBuilder;

use crate::exports::LuaExportsTable;

use super::{super::*, EnumItem};

/**
    An implementation of the [TweenInfo](https://create.roblox.com/docs/reference/engine/datatypes/TweenInfo) Roblox datatype.

    This implements all documented properties, methods & constructors of the `TweenInfo` class as of May 2026.

    Unlike most datatypes, `TweenInfo` is never stored on instances, so it has no
    `rbx_dom_weak` backing type and does not participate in DOM conversion.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct TweenInfo {
    pub(crate) time: f32,
    pub(crate) easing_style: EnumItem,
    pub(crate) easing_direction: EnumItem,
    pub(crate) repeat_count: i32,
    pub(crate) reverses: bool,
    pub(crate) delay_time: f32,
}

fn coerce_easing_enum(
    value: Option<EnumItem>,
    enum_name: &'static str,
    default_item: &'static str,
) -> LuaResult<EnumItem> {
    match value {
        Some(item) => {
            if item.parent.desc.name == enum_name {
                Ok(item)
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Expected Enum.{enum_name}, got Enum.{}",
                    item.parent.desc.name
                )))
            }
        }
        None => EnumItem::from_enum_name_and_name(enum_name, default_item).ok_or_else(|| {
            LuaError::RuntimeError(format!(
                "Failed to construct default Enum.{enum_name}.{default_item}"
            ))
        }),
    }
}

impl LuaExportsTable for TweenInfo {
    const EXPORT_NAME: &'static str = "TweenInfo";

    #[allow(clippy::type_complexity)]
    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let tween_info_new =
            |_: &Lua,
             (time, easing_style, easing_direction, repeat_count, reverses, delay_time): (
                Option<f32>,
                Option<EnumItem>,
                Option<EnumItem>,
                Option<i32>,
                Option<bool>,
                Option<f32>,
            )| {
                Ok(TweenInfo {
                    time: time.unwrap_or(1.0),
                    easing_style: coerce_easing_enum(easing_style, "EasingStyle", "Quad")?,
                    easing_direction: coerce_easing_enum(
                        easing_direction,
                        "EasingDirection",
                        "Out",
                    )?,
                    repeat_count: repeat_count.unwrap_or(0),
                    reverses: reverses.unwrap_or(false),
                    delay_time: delay_time.unwrap_or(0.0),
                })
            };

        TableBuilder::new(lua)?
            .with_function("new", tween_info_new)?
            .build_readonly()
    }
}

impl LuaUserData for TweenInfo {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Time", |_, this| Ok(this.time));
        fields.add_field_method_get("EasingStyle", |_, this| Ok(this.easing_style.clone()));
        fields.add_field_method_get("EasingDirection", |_, this| {
            Ok(this.easing_direction.clone())
        });
        fields.add_field_method_get("RepeatCount", |_, this| Ok(this.repeat_count));
        fields.add_field_method_get("Reverses", |_, this| Ok(this.reverses));
        fields.add_field_method_get("DelayTime", |_, this| Ok(this.delay_time));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for TweenInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {}",
            self.time,
            self.easing_style,
            self.easing_direction,
            self.repeat_count,
            self.reverses,
            self.delay_time
        )
    }
}
