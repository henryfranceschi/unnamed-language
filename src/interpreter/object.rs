use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use super::value::Value;

/// Object pointer.
pub struct Obj(*mut ObjCommon);

impl<T: SubObject> From<Box<T>> for Obj {
    fn from(value: Box<T>) -> Self {
        Obj(Box::into_raw(value) as *mut ObjCommon)
    }
}

impl Deref for Obj {
    type Target = ObjCommon;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for Obj {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl<T: SubObject> AsRef<T> for Obj {
    fn as_ref(&self) -> &T {
        unsafe {
            assert_eq!((*self.0).kind, T::KIND);

            let ptr = self.0 as *mut T;
            &mut *ptr
        }
    }
}

impl<T: SubObject> AsMut<T> for Obj {
    fn as_mut(&mut self) -> &mut T {
        unsafe {
            assert_eq!((*self.0).kind, T::KIND);

            let ptr = self.0 as *mut T;
            &mut *ptr
        }
    }
}

impl Drop for Obj {
    fn drop(&mut self) {
        unsafe {
            match (*self.0).kind {
                ObjKind::String => {
                    let _ = Box::from_raw(self.0 as *mut ObjString);
                }
                ObjKind::Function => {
                    let _ = Box::from_raw(self.0 as *mut ObjFunction);
                }
                ObjKind::Instance => {
                    let _ = Box::from_raw(self.0 as *mut ObjInstance);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ObjKind {
    String,
    Function,
    Instance,
}

trait SubObject {
    const KIND: ObjKind;
}

#[repr(C)]
pub struct ObjCommon {
    pub marked: bool,
    kind: ObjKind,
}

impl ObjCommon {
    fn new(kind: ObjKind) -> Self {
        Self {
            marked: false,
            kind,
        }
    }
}

#[repr(C)]
pub struct ObjString {
    pub obj: ObjCommon,
    data: String,
}

impl SubObject for ObjString {
    const KIND: ObjKind = ObjKind::String;
}

impl ObjString {
    pub fn obj(data: String) -> Obj {
        Box::new(Self {
            obj: ObjCommon::new(Self::KIND),
            data,
        })
        .into()
    }
}

#[repr(C)]
pub struct ObjFunction {
    pub obj: ObjCommon,
    arity: u8,
}

impl SubObject for ObjFunction {
    const KIND: ObjKind = ObjKind::Function;
}

impl ObjFunction {
    pub fn obj(arity: u8) -> Obj {
        Box::new(Self {
            obj: ObjCommon::new(Self::KIND),
            arity,
        })
        .into()
    }
}

#[repr(C)]
pub struct ObjInstance {
    pub obj: ObjCommon,
    fields: HashMap<String, Value>,
}

impl SubObject for ObjInstance {
    const KIND: ObjKind = ObjKind::Instance;
}

impl ObjInstance {
    pub fn obj() -> Obj {
        Box::new(Self {
            obj: ObjCommon::new(Self::KIND),
            fields: HashMap::new(),
        })
        .into()
    }
}
