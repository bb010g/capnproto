#[cxx::bridge(namespace = "kj")]
mod r#async {
    unsafe extern "C++" {
        include!(<kj/async.h>);

        type EventLoop;
    }
}
