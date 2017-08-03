// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use ffi;
use err::PyErr;
use typeob::PyTypeObject;
use conversion::ToPyObject;
use python::{Python, IntoPyPointer};

/// Defines rust type for exception defined in Python code.
///
/// # Syntax
/// `import_exception!(module, MyError)`
///
/// * `module` is the name of the containing module.
/// * `MyError` is the name of the new exception type.
///
/// # Example
/// ```
/// #[macro_use]
/// extern crate pyo3;
///
/// use pyo3::{Python, PyDict};
///
/// import_exception!(socket, gaierror);
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let ctx = PyDict::new(py);
///
///     ctx.set_item("gaierror", py.get_type::<gaierror>()).unwrap();
///     py.run("import socket; assert gaierror is socket.gaierror", None, Some(ctx)).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! import_exception {
    ($module: ident, $name: ident) => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl $crate::std::convert::From<$name> for $crate::PyErr {
            fn from(_err: $name) -> $crate::PyErr {
                $crate::PyErr::new::<$name, _>(())
            }
        }

        impl $name {
            pub fn new<T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyErr
                where Self: $crate::typeob::PyTypeObject + Sized
            {
                $crate::PyErr::new::<Self, T>(args)
            }
        }

        impl $crate::PyNativeException for $name {
            const MOD: &'static str = stringify!($module);
            const NAME: &'static str = stringify!($name);
        }

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type() {}

            #[inline]
            fn type_object() -> $crate::Py<$crate::PyType> {
                use $crate::IntoPyPointer;
                static mut TYPE_OBJECT: *mut $crate::ffi::PyTypeObject =
                    $crate::std::ptr::null_mut();

                unsafe {
                    if TYPE_OBJECT.is_null() {
                        let gil = $crate::Python::acquire_gil();
                        let py = gil.python();

                        let imp = py.import(stringify!($module))
                            .expect(concat!(
                                "Can not import module: ", stringify!($module)));
                        let cls = imp.get(stringify!($name))
                            .expect(concat!(
                                "Can not load exception class: {}.{}", stringify!($module),
                                ".", stringify!($name)));
                        TYPE_OBJECT = cls.into_ptr() as *mut $crate::ffi::PyTypeObject;
                    }

                    $crate::Py::from_borrowed_ptr(
                        TYPE_OBJECT as *const _ as *mut $crate::ffi::PyObject)
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use {PyErr, Python};
    use objects::PyDict;

    import_exception!(socket, gaierror);

    #[test]
    fn test_check_exception() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let err: PyErr = gaierror.into();

        let d = PyDict::new(py);
        d.set_item("socket", py.import("socket").unwrap()).unwrap();
        d.set_item("exc", err).unwrap();

        py.run("assert isinstance(exc, socket.gaierror)", None, Some(d)).unwrap();
    }
}
