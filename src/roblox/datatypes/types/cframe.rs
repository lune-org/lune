use core::fmt;
use std::ops;

use glam::{EulerRot, Mat3, Mat4, Quat, Vec3};
use mlua::{prelude::*, Variadic};
use rbx_dom_weak::types::{CFrame as DomCFrame, Matrix3 as DomMatrix3, Vector3 as DomVector3};

use crate::{lune::util::TableBuilder, roblox::exports::LuaExportsTable};

use super::{super::*, Vector3};

/**
    An implementation of the [CFrame](https://create.roblox.com/docs/reference/engine/datatypes/CFrame)
    Roblox datatype, backed by [`glam::Mat4`].

    This implements all documented properties, methods &
    constructors of the CFrame class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CFrame(pub Mat4);

impl CFrame {
    pub const IDENTITY: Self = Self(Mat4::IDENTITY);

    fn position(&self) -> Vec3 {
        self.0.w_axis.truncate()
    }

    fn orientation(&self) -> Mat3 {
        Mat3::from_cols(
            self.0.x_axis.truncate(),
            self.0.y_axis.truncate(),
            self.0.z_axis.truncate(),
        )
    }

    fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }
}

impl LuaExportsTable<'_> for CFrame {
    const EXPORT_NAME: &'static str = "CFrame";

    fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
        let cframe_angles = |_, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::XYZ, rx, ry, rz)))
        };

        let cframe_from_axis_angle =
            |_, (v, r): (LuaUserDataRef<Vector3>, f32)| Ok(CFrame(Mat4::from_axis_angle(v.0, r)));

        let cframe_from_euler_angles_xyz = |_, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::XYZ, rx, ry, rz)))
        };

        let cframe_from_euler_angles_yxz = |_, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::YXZ, ry, rx, rz)))
        };

        let cframe_from_matrix = |_,
                                  (pos, rx, ry, rz): (
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            Option<LuaUserDataRef<Vector3>>,
        )| {
            Ok(CFrame(Mat4::from_cols(
                rx.0.extend(0.0),
                ry.0.extend(0.0),
                rz.map(|r| r.0)
                    .unwrap_or_else(|| rx.0.cross(ry.0).normalize())
                    .extend(0.0),
                pos.0.extend(1.0),
            )))
        };

        let cframe_from_orientation = |_, (rx, ry, rz): (f32, f32, f32)| {
            Ok(CFrame(Mat4::from_euler(EulerRot::YXZ, ry, rx, rz)))
        };

        let cframe_look_at = |_,
                              (from, to, up): (
            LuaUserDataRef<Vector3>,
            LuaUserDataRef<Vector3>,
            Option<LuaUserDataRef<Vector3>>,
        )| {
            Ok(CFrame(look_at(
                from.0,
                to.0,
                up.as_deref().unwrap_or(&Vector3(Vec3::Y)).0,
            )))
        };

        // Dynamic args constructor
        type ArgsPos<'lua> = LuaUserDataRef<'lua, Vector3>;
        type ArgsLook<'lua> = (
            LuaUserDataRef<'lua, Vector3>,
            LuaUserDataRef<'lua, Vector3>,
            Option<LuaUserDataRef<'lua, Vector3>>,
        );

        type ArgsPosXYZ = (f32, f32, f32);
        type ArgsPosXYZQuat = (f32, f32, f32, f32, f32, f32, f32);
        type ArgsMatrix = (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32);

        let cframe_new = |lua, args: LuaMultiValue| match args.len() {
            0 => Ok(CFrame(Mat4::IDENTITY)),

            1 => match ArgsPos::from_lua_multi(args, lua) {
                Ok(pos) => Ok(CFrame(Mat4::from_translation(pos.0))),
                Err(err) => Err(err),
            },

            3 => {
                if let Ok((from, to, up)) = ArgsLook::from_lua_multi(args.clone(), lua) {
                    Ok(CFrame(look_at(
                        from.0,
                        to.0,
                        up.as_deref().unwrap_or(&Vector3(Vec3::Y)).0,
                    )))
                } else if let Ok((x, y, z)) = ArgsPosXYZ::from_lua_multi(args, lua) {
                    Ok(CFrame(Mat4::from_translation(Vec3::new(x, y, z))))
                } else {
                    // TODO: Make this error message better
                    Err(LuaError::RuntimeError(
                        "Invalid arguments to constructor".to_string(),
                    ))
                }
            }

            7 => match ArgsPosXYZQuat::from_lua_multi(args, lua) {
                Ok((x, y, z, qx, qy, qz, qw)) => Ok(CFrame(Mat4::from_rotation_translation(
                    Quat::from_array([qx, qy, qz, qw]),
                    Vec3::new(x, y, z),
                ))),
                Err(err) => Err(err),
            },

            12 => match ArgsMatrix::from_lua_multi(args, lua) {
                Ok((x, y, z, r00, r01, r02, r10, r11, r12, r20, r21, r22)) => {
                    Ok(CFrame(Mat4::from_cols_array_2d(&[
                        [r00, r10, r20, 0.0],
                        [r01, r11, r21, 0.0],
                        [r02, r12, r22, 0.0],
                        [x, y, z, 1.0],
                    ])))
                }
                Err(err) => Err(err),
            },

            _ => Err(LuaError::RuntimeError(format!(
                "Invalid number of arguments: expected 0, 1, 3, 7, or 12, got {}",
                args.len()
            ))),
        };

        TableBuilder::new(lua)?
            .with_function("Angles", cframe_angles)?
            .with_value("identity", CFrame(Mat4::IDENTITY))?
            .with_function("fromAxisAngle", cframe_from_axis_angle)?
            .with_function("fromEulerAnglesXYZ", cframe_from_euler_angles_xyz)?
            .with_function("fromEulerAnglesYXZ", cframe_from_euler_angles_yxz)?
            .with_function("fromMatrix", cframe_from_matrix)?
            .with_function("fromOrientation", cframe_from_orientation)?
            .with_function("lookAt", cframe_look_at)?
            .with_function("new", cframe_new)?
            .build_readonly()
    }
}

impl LuaUserData for CFrame {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Position", |_, this| Ok(Vector3(this.position())));
        fields.add_field_method_get("Rotation", |_, this| {
            Ok(CFrame(Mat4::from_cols(
                this.0.x_axis,
                this.0.y_axis,
                this.0.z_axis,
                Vec3::ZERO.extend(1.0),
            )))
        });
        fields.add_field_method_get("X", |_, this| Ok(this.position().x));
        fields.add_field_method_get("Y", |_, this| Ok(this.position().y));
        fields.add_field_method_get("Z", |_, this| Ok(this.position().z));
        fields.add_field_method_get("XVector", |_, this| Ok(Vector3(this.orientation().x_axis)));
        fields.add_field_method_get("YVector", |_, this| Ok(Vector3(this.orientation().y_axis)));
        fields.add_field_method_get("ZVector", |_, this| Ok(Vector3(this.orientation().z_axis)));
        fields.add_field_method_get("RightVector", |_, this| {
            Ok(Vector3(this.orientation().x_axis))
        });
        fields.add_field_method_get("UpVector", |_, this| Ok(Vector3(this.orientation().y_axis)));
        fields.add_field_method_get("LookVector", |_, this| {
            Ok(Vector3(-this.orientation().z_axis))
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Methods
        methods.add_method("Inverse", |_, this, ()| Ok(this.inverse()));
        methods.add_method(
            "Lerp",
            |_, this, (goal, alpha): (LuaUserDataRef<CFrame>, f32)| {
                let quat_this = Quat::from_mat4(&this.0);
                let quat_goal = Quat::from_mat4(&goal.0);
                let translation = this
                    .0
                    .w_axis
                    .truncate()
                    .lerp(goal.0.w_axis.truncate(), alpha);
                let rotation = quat_this.slerp(quat_goal, alpha);
                Ok(CFrame(Mat4::from_rotation_translation(
                    rotation,
                    translation,
                )))
            },
        );
        methods.add_method("Orthonormalize", |_, this, ()| {
            let rotation = Quat::from_mat4(&this.0);
            let translation = this.0.w_axis.truncate();
            Ok(CFrame(Mat4::from_rotation_translation(
                rotation.normalize(),
                translation,
            )))
        });
        methods.add_method(
            "ToWorldSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<CFrame>>| {
                Ok(Variadic::from_iter(rhs.into_iter().map(|cf| *this * *cf)))
            },
        );
        methods.add_method(
            "ToObjectSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<CFrame>>| {
                let inverse = this.inverse();
                Ok(Variadic::from_iter(rhs.into_iter().map(|cf| inverse * *cf)))
            },
        );
        methods.add_method(
            "PointToWorldSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                Ok(Variadic::from_iter(rhs.into_iter().map(|v3| *this * *v3)))
            },
        );
        methods.add_method(
            "PointToObjectSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                let inverse = this.inverse();
                Ok(Variadic::from_iter(rhs.into_iter().map(|v3| inverse * *v3)))
            },
        );
        methods.add_method(
            "VectorToWorldSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                let result = *this - Vector3(this.position());
                Ok(Variadic::from_iter(rhs.into_iter().map(|v3| result * *v3)))
            },
        );
        methods.add_method(
            "VectorToObjectSpace",
            |_, this, rhs: Variadic<LuaUserDataRef<Vector3>>| {
                let inverse = this.inverse();
                let result = inverse - Vector3(inverse.position());

                Ok(Variadic::from_iter(rhs.into_iter().map(|v3| result * *v3)))
            },
        );
        #[rustfmt::skip]
        methods.add_method("GetComponents", |_, this, ()| {
            let pos = this.position();
            let transposed = this.orientation().transpose();
            Ok((
                pos.x, pos.y, pos.z,
				 transposed.x_axis.x, transposed.x_axis.y,   transposed.x_axis.z,
				 transposed.y_axis.x, transposed.y_axis.y,   transposed.y_axis.z,
				 transposed.z_axis.x, transposed.z_axis.y,   transposed.z_axis.z,
            ))
        });
        methods.add_method("ToEulerAnglesXYZ", |_, this, ()| {
            Ok(Quat::from_mat4(&this.0).to_euler(EulerRot::XYZ))
        });
        methods.add_method("ToEulerAnglesYXZ", |_, this, ()| {
            let (ry, rx, rz) = Quat::from_mat4(&this.0).to_euler(EulerRot::YXZ);
            Ok((rx, ry, rz))
        });
        methods.add_method("ToOrientation", |_, this, ()| {
            let (ry, rx, rz) = Quat::from_mat4(&this.0).to_euler(EulerRot::YXZ);
            Ok((rx, ry, rz))
        });
        methods.add_method("ToAxisAngle", |_, this, ()| {
            let (axis, angle) = Quat::from_mat4(&this.0).to_axis_angle();
            Ok((Vector3(axis), angle))
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Mul, |lua, this, rhs: LuaValue| {
            if let LuaValue::UserData(ud) = &rhs {
                if let Ok(cf) = ud.borrow::<CFrame>() {
                    return lua.create_userdata(*this * *cf);
                } else if let Ok(vec) = ud.borrow::<Vector3>() {
                    return lua.create_userdata(*this * *vec);
                }
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "userdata",
                message: Some(format!(
                    "Expected CFrame or Vector3, got {}",
                    rhs.type_name()
                )),
            })
        });
        methods.add_meta_method(
            LuaMetaMethod::Add,
            |_, this, vec: LuaUserDataRef<Vector3>| Ok(*this + *vec),
        );
        methods.add_meta_method(
            LuaMetaMethod::Sub,
            |_, this, vec: LuaUserDataRef<Vector3>| Ok(*this - *vec),
        );
    }
}

impl fmt::Display for CFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos = self.position();
        let transposed = self.orientation().transpose();
        write!(
            f,
            "{}, {}, {}, {}",
            Vector3(pos),
            Vector3(transposed.x_axis),
            Vector3(transposed.y_axis),
            Vector3(transposed.z_axis)
        )
    }
}

impl ops::Mul for CFrame {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        CFrame(self.0 * rhs.0)
    }
}

impl ops::Mul<Vector3> for CFrame {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Self::Output {
        Vector3(self.0.project_point3(rhs.0))
    }
}

impl ops::Add<Vector3> for CFrame {
    type Output = Self;
    fn add(self, rhs: Vector3) -> Self::Output {
        CFrame(Mat4::from_cols(
            self.0.x_axis,
            self.0.y_axis,
            self.0.z_axis,
            self.0.w_axis + rhs.0.extend(0.0),
        ))
    }
}

impl ops::Sub<Vector3> for CFrame {
    type Output = Self;
    fn sub(self, rhs: Vector3) -> Self::Output {
        CFrame(Mat4::from_cols(
            self.0.x_axis,
            self.0.y_axis,
            self.0.z_axis,
            self.0.w_axis - rhs.0.extend(0.0),
        ))
    }
}

impl From<DomCFrame> for CFrame {
    fn from(v: DomCFrame) -> Self {
        CFrame(Mat4::from_cols(
            Vector3::from(v.orientation.x).0.extend(0.0),
            Vector3::from(v.orientation.y).0.extend(0.0),
            Vector3::from(v.orientation.z).0.extend(0.0),
            Vector3::from(v.position).0.extend(1.0),
        ))
    }
}

impl From<CFrame> for DomCFrame {
    fn from(v: CFrame) -> Self {
        let orientation = v.orientation();
        DomCFrame {
            position: DomVector3::from(Vector3(v.position())),
            orientation: DomMatrix3::new(
                DomVector3::from(Vector3(orientation.x_axis)),
                DomVector3::from(Vector3(orientation.y_axis)),
                DomVector3::from(Vector3(orientation.z_axis)),
            ),
        }
    }
}

/**
    Creates a matrix at the position `from`, looking towards `to`.

    [`glam`] does provide functions such as [`look_at_lh`], [`look_at_rh`] and more but
    they all create view matrices for camera transforms which is not what we want here.
*/
fn look_at(from: Vec3, to: Vec3, up: Vec3) -> Mat4 {
    let dir = (to - from).normalize();
    let xaxis = up.cross(dir).normalize();
    let yaxis = dir.cross(xaxis).normalize();

    Mat4::from_cols(
        Vec3::new(xaxis.x, yaxis.x, dir.x).extend(0.0),
        Vec3::new(xaxis.y, yaxis.y, dir.y).extend(0.0),
        Vec3::new(xaxis.z, yaxis.z, dir.z).extend(0.0),
        from.extend(1.0),
    )
}

#[cfg(test)]
mod cframe_test {
    use glam::{Mat4, Vec3};
    use rbx_dom_weak::types::{CFrame as DomCFrame, Matrix3 as DomMatrix3, Vector3 as DomVector3};

    use super::CFrame;

    #[test]
    fn dom_cframe_from_cframe() {
        let dom_cframe = DomCFrame::new(
            DomVector3::new(1.0, 2.0, 3.0),
            DomMatrix3::new(
                DomVector3::new(1.0, 2.0, 3.0),
                DomVector3::new(1.0, 2.0, 3.0),
                DomVector3::new(1.0, 2.0, 3.0),
            ),
        );

        let cframe = CFrame(Mat4::from_cols(
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(1.0),
        ));

        assert_eq!(CFrame::from(dom_cframe), cframe)
    }

    #[test]
    fn cframe_from_dom_cframe() {
        let cframe = CFrame(Mat4::from_cols(
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(0.0),
            Vec3::new(1.0, 2.0, 3.0).extend(1.0),
        ));

        let dom_cframe = DomCFrame::new(
            DomVector3::new(1.0, 2.0, 3.0),
            DomMatrix3::new(
                DomVector3::new(1.0, 2.0, 3.0),
                DomVector3::new(1.0, 2.0, 3.0),
                DomVector3::new(1.0, 2.0, 3.0),
            ),
        );

        assert_eq!(DomCFrame::from(cframe), dom_cframe)
    }
}
