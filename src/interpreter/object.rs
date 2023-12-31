/// Object pointer.
pub struct Obj(*mut ObjCommon);

impl Obj {
    pub fn marked(&self) -> bool {
        unsafe { (*self.0).marked }
    }

    pub fn set_marked(&mut self, marked: bool) {
        unsafe {
            (*self.0).marked = marked;
        }
    }

    unsafe fn downcast<T>(&mut self) -> &mut T {
        let ptr = self.0 as *mut T;
        &mut (*ptr)
    }

    pub fn obj_as_string(&mut self) -> &mut ObjString {
        unsafe {
            assert!((*self.0).kind == ObjKind::String);
            self.downcast()
        }
    }

    pub fn obj_as_function(&mut self) -> &mut ObjFunction {
        unsafe {
            assert!((*self.0).kind == ObjKind::Function);
            self.downcast()
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
            }
        }
    }
}

#[repr(C)]
struct ObjCommon {
    marked: bool,
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

#[derive(Debug, PartialEq, Eq)]
pub enum ObjKind {
    String,
    Function,
}

#[repr(C)]
pub struct ObjString {
    obj: ObjCommon,
    data: String,
}

impl ObjString {
    pub fn new(data: String) -> Self {
        Self {
            obj: ObjCommon::new(ObjKind::String),
            data,
        }
    }
}

impl From<ObjString> for Obj {
    fn from(value: ObjString) -> Self {
        let b = Box::new(value);
        let ptr = Box::into_raw(b);
        let c = ptr as *mut ObjCommon;
        Obj(c)
    }
}

#[repr(C)]
pub struct ObjFunction {
    obj: ObjCommon,
    arity: u8,
}

impl ObjFunction {
    pub fn new(arity: u8) -> Self {
        Self {
            obj: ObjCommon::new(ObjKind::Function),
            arity,
        }
    }
}

impl From<ObjFunction> for Obj {
    fn from(value: ObjFunction) -> Self {
        let b = Box::new(value);
        let ptr = Box::into_raw(b);
        let c = ptr as *mut ObjCommon;
        Obj(c)
    }
}
