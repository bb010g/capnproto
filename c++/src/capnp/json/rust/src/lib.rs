mod ffi {
    #[cxx::bridge(namespace = "capnp")]
    pub mod capnp {
        unsafe extern "C++" {
            include!(<capnp/compat/json.h>);
        }
    }
}
