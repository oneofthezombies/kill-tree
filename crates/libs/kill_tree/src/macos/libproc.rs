#[allow(clippy::all)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/libproc_bindings.rs"));
}

pub(crate) use bindings::*;
