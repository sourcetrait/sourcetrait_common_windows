#[cfg(feature = "crossplat")]
pub(crate) mod crossplat {
    pub(crate) mod component {
        pub(crate) mod net {
            pub(crate) mod net;
        }
        pub(crate) mod cmd;
        pub(crate) mod ui;
    }
    pub(crate) mod consts;
}
pub(crate) mod winsys;

pub use crate::{
    crossplat::{
        component::{
            net::{
                net::*,
            },
            cmd::*,
            ui::*,
        },
        consts::*,
    },
    winsys::*,
};

pub(crate) use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    ffi::c_void,
    os::windows::ffi::OsStrExt,
};

pub(crate) use sourcetrait_twostr::*;
pub(crate) use sourcetrait_crossplat_bridge::{
    self as cross,
    prelude::driver::*
};
pub(crate) use widestring::{U16CString, U16CStr};
