#![crate_name = "russimp"]
#![crate_type = "lib"]

pub extern crate russimp_sys as sys;

#[cfg(feature = "mint")]
mod impl_mint;
#[cfg(feature = "mint")]
pub use impl_mint::*;

use derivative::Derivative;
use glam::{vec4, Mat4, Quat, Vec2, Vec3};
use std::{
    ffi::IntoStringError,
    fmt,
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use sys::{aiAABB, aiColor3D, aiColor4D, aiMatrix4x4, aiQuaternion, aiVector2D, aiVector3D};

#[macro_use]
extern crate num_derive;

pub mod animation;
pub mod bone;
pub mod camera;
pub mod face;
pub mod light;
pub mod material;
pub mod mesh;
pub mod metadata;
pub mod node;
pub mod scene;

#[derive(Derivative)]
#[derivative(Debug)]
pub enum RussimpError {
    Import(String),
    MetadataError(String),
    MaterialError(String),
    Primitive(String),
    TextureNotFound,
}

impl Display for RussimpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RussimpError::Import(content) => {
                write!(f, "{}", content)
            }
            _ => {
                write!(f, "unknown error")
            }
        }
    }
}

#[derive(Clone, Copy, Default, Derivative)]
#[derivative(Debug)]
#[repr(C)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl From<&aiAABB> for AABB {
    fn from(aabb: &aiAABB) -> Self {
        Self {
            max: (&aabb.mMax).convert_into(),
            min: (&aabb.mMin).convert_into(),
        }
    }
}

#[derive(Clone, Copy, Default, Derivative)]
#[derivative(Debug)]
#[repr(C)]
pub struct Color4D {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<&aiColor4D> for Color4D {
    fn from(color: &aiColor4D) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

#[derive(Clone, Copy, Default, Derivative)]
#[derivative(Debug)]
#[repr(C)]
pub struct Color3D {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<&aiColor3D> for Color3D {
    fn from(color: &aiColor3D) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

pub trait ConvertFrom<T>: Sized {
    /// Converts to this type from the input type.
    fn convert_from(value: T) -> Self;
}

// local replacement for From trait for foreign types
trait ConvertInto<T>: Sized {
    fn convert_into(&self) -> T;
}

impl ConvertFrom<&aiVector3D> for Vec3 {
    fn convert_from(value: &aiVector3D) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl ConvertInto<Mat4> for &aiMatrix4x4 {
    fn convert_into(&self) -> Mat4 {
        Mat4::from_cols(
            vec4(self.a1, self.b1, self.c1, self.d1), // m00, m01, m02, m03
            vec4(self.a2, self.b2, self.c2, self.d2), // m10, m11, m12, m13
            vec4(self.a3, self.b3, self.c3, self.d3), // m20, m21, m22, m23
            vec4(self.a4, self.b4, self.c4, self.d4), // m30, m31, m32, m33
        )
    }
}

impl ConvertInto<Vec2> for &aiVector2D {
    fn convert_into(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl ConvertInto<Vec3> for &aiVector3D {
    fn convert_into(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl ConvertInto<Vec3> for aiVector3D {
    fn convert_into(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl ConvertInto<Quat> for aiQuaternion {
    fn convert_into(&self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

#[derive(Clone, Copy, Default, Derivative)]
#[derivative(Debug)]
#[repr(C)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl From<Utf8Error> for RussimpError {
    fn from(val: Utf8Error) -> Self {
        RussimpError::Primitive(val.to_string())
    }
}

impl From<IntoStringError> for RussimpError {
    fn from(val: IntoStringError) -> Self {
        RussimpError::Primitive(val.to_string())
    }
}

pub type Russult<T> = Result<T, RussimpError>;

pub mod utils {
    use crate::{ConvertFrom, ConvertInto};
    use std::{os::raw::c_uint, ptr::slice_from_raw_parts};

    pub(crate) fn get_base_type_vec_from_raw<'a, TRaw: 'a>(
        data: *mut *mut TRaw,
        len: u32,
    ) -> Vec<&'a TRaw> {
        let slice = slice_from_raw_parts(data, len as usize);
        if slice.is_null() {
            return vec![];
        }

        let raw = unsafe { slice.as_ref() }.unwrap();
        raw.iter().map(|x| unsafe { x.as_ref() }.unwrap()).collect()
    }

    #[allow(dead_code)]
    pub(crate) fn get_model(relative_path_from_root: &str) -> String {
        if let Ok(mut github_root) = std::env::var("GITHUB_WORKSPACE") {
            github_root.push('/');
            github_root.push_str(relative_path_from_root);

            github_root
        } else {
            relative_path_from_root.into()
        }
    }

    pub(crate) fn get_raw<'a, TRaw: 'a, TComponent: From<&'a TRaw>>(
        raw: *mut TRaw,
    ) -> Option<TComponent> {
        unsafe { raw.as_ref() }.map(|x| x.into())
    }

    pub(crate) fn get_vec<'a, TRaw: 'a, TComponent: From<&'a TRaw>>(
        raw: *mut TRaw,
        len: c_uint,
    ) -> Vec<TComponent> {
        let slice = slice_from_raw_parts(raw as *const TRaw, len as usize);
        if slice.is_null() {
            return vec![];
        }

        let raw = unsafe { slice.as_ref() }.unwrap();
        raw.iter().map(|x| x.into()).collect()
    }

    pub(crate) fn get_vec_2<'a, TRaw: 'a, TComponent: ConvertFrom<&'a TRaw>, T>(
        raw: *mut TRaw,
        len: c_uint,
    ) -> Vec<TComponent>
    where
        TRaw: ConvertInto<T>,
        Vec<TComponent>: FromIterator<T>,
    {
        let slice = slice_from_raw_parts(raw as *const TRaw, len as usize);
        if slice.is_null() {
            return vec![];
        }

        let raw = unsafe { slice.as_ref() }.unwrap();
        raw.iter().map(|x| x.convert_into()).collect()
    }

    pub fn get_raw_vec<TRaw>(raw: *mut TRaw, len: c_uint) -> Vec<TRaw>
    where
        TRaw: Clone,
    {
        let slice = slice_from_raw_parts(raw as *const TRaw, len as usize);
        if slice.is_null() {
            return vec![];
        }

        let raw = unsafe { slice.as_ref() }.unwrap();
        raw.to_vec()
    }

    pub fn get_vec_from_raw<'a, TComponent: From<&'a TRaw>, TRaw: 'a>(
        raw_source: *mut *mut TRaw,
        num_raw_items: c_uint,
    ) -> Vec<TComponent> {
        let slice = slice_from_raw_parts(raw_source, num_raw_items as usize);
        if slice.is_null() {
            return vec![];
        }

        let raw = unsafe { slice.as_ref() }.unwrap();
        raw.iter()
            .map(|x| (unsafe { x.as_ref() }.unwrap()).into())
            .collect()
    }

    pub(crate) fn get_vec_of_vecs_from_raw<'a, TRaw: 'a, TComponent: From<&'a TRaw>>(
        raw: [*mut TRaw; 8usize],
        len: c_uint,
    ) -> Vec<Option<Vec<TComponent>>> {
        raw.iter()
            .map(|head| unsafe { head.as_mut() }.map(|head| get_vec(head, len)))
            .collect()
    }

    pub(crate) fn get_vec_of_vecs_from_raw_2<'a, TRaw: 'a, TComponent: ConvertFrom<&'a TRaw>, T>(
        raw: [*mut TRaw; 8usize],
        len: c_uint,
    ) -> Vec<Option<Vec<TComponent>>>
    where
        TRaw: ConvertInto<T>,
        Vec<TComponent>: FromIterator<T>,
    {
        raw.iter()
            .map(|head| unsafe { head.as_mut() }.map(|head| get_vec_2(head, len)))
            .collect()
    }
}
