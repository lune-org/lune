use core::fmt;

use glam::Vec3;
use mlua::prelude::*;
use rbx_dom_weak::types::Ray as DomRay;

use lune_utils::TableBuilder;

use crate::exports::LuaExportsTable;

use super::{super::*, Vector3};

/**
    An implementation of the [Ray](https://create.roblox.com/docs/reference/engine/datatypes/Ray)
    Roblox datatype, backed by [`glam::Vec3`].

    This implements all documented properties, methods & constructors of the Ray class as of October 2025.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub(crate) origin: Vec3,
    pub(crate) direction: Vec3,
}

impl Ray {
    fn closest_point(&self, point: Vec3) -> Vec3 {
        let norm = self.direction.normalize();
        let lhs = point - self.origin;

        let dot_product = lhs.dot(norm).max(0.0);
        self.origin + norm * dot_product
    }
}

impl LuaExportsTable for Ray {
    const EXPORT_NAME: &'static str = "Ray";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let ray_new =
            |_: &Lua, (origin, direction): (LuaUserDataRef<Vector3>, LuaUserDataRef<Vector3>)| {
                Ok(Ray {
                    origin: origin.0,
                    direction: direction.0,
                })
            };

        TableBuilder::new(lua)?
            .with_function("new", ray_new)?
            .build_readonly()
    }
}

impl LuaUserData for Ray {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Origin", |_, this| Ok(Vector3(this.origin)));
        fields.add_field_method_get("Direction", |_, this| Ok(Vector3(this.direction)));
        fields.add_field_method_get("Unit", |_, this| {
            Ok(Ray {
                origin: this.origin,
                direction: this.direction.normalize(),
            })
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Methods
        methods.add_method("ClosestPoint", |_, this, to: LuaUserDataRef<Vector3>| {
            Ok(Vector3(this.closest_point(to.0)))
        });
        methods.add_method("Distance", |_, this, to: LuaUserDataRef<Vector3>| {
            let closest = this.closest_point(to.0);
            Ok((closest - to.0).length())
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", Vector3(self.origin), Vector3(self.direction))
    }
}

impl From<DomRay> for Ray {
    fn from(v: DomRay) -> Self {
        Ray {
            origin: Vector3::from(v.origin).0,
            direction: Vector3::from(v.direction).0,
        }
    }
}

impl From<Ray> for DomRay {
    fn from(v: Ray) -> Self {
        DomRay {
            origin: Vector3(v.origin).into(),
            direction: Vector3(v.direction).into(),
        }
    }
}
