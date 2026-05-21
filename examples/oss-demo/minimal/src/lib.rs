//! Tiny crate for pinned structural drift demo output.

pub mod api {
    pub fn handle() {}
}

pub mod store {
    use crate::api;

    pub fn load() {
        api::handle();
    }
}
