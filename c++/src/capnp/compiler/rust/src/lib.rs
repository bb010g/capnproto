mod ffi {
    #[repr(transparent)]
    pub struct CInt(pub core::ffi::c_int);

    unsafe impl cxx::ExternType for CInt {
        type Id = cxx::type_id!("capnp::compiler::cxx_int");
        type Kind = cxx::kind::Trivial;
    }

    #[cxx::bridge(namespace = "capnp::compiler")]
    pub mod capnp_compiler {
        unsafe extern "C++" {
            include!(<capnp/compiler/rust/cxx.h>);

            type cxx_int = crate::ffi::CInt;

            // type Module;

            // type Compiler;
            unsafe fn main(argc: cxx_int, argv: *mut *mut c_char) -> cxx_int;
        }
    }
}

pub mod capnpc {
    use std::{ffi, ptr};

    pub fn main(args: &[&ffi::CStr]) -> ffi::c_int {
        let argc = args.len();
        let mut argv: Vec<*mut ffi::c_char> = Vec::with_capacity(argc + 1);
        for arg in args {
            argv.push(arg.as_ptr() as *mut ffi::c_char);
        }
        argv.push(ptr::null_mut()); // Nul terminator.
        unsafe {
            crate::ffi::capnp_compiler::main(
                crate::ffi::CInt(argc as ffi::c_int),
                argv.as_mut_ptr(),
            )
        }
        .0
    }
}
