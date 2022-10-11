#[cxx::bridge(namespace = "capnp")]
mod common {
    unsafe extern "C++" {
        include!(<capnp/common.h>);

        type uint;

        type Void;

        type Text;
        type Data;
    }
}
