//! WARNING: this is not part of the crate's public API and is subject to change at any time

use self::sealed::KVs;
use crate::{Level, Metadata, Record};
use std::fmt::Arguments;
pub use std::{file, format_args, line, module_path, stringify};

#[cfg(feature = "kv_unstable")]
pub type Value<'a> = dyn crate::kv::value::ToValue + 'a;

#[cfg(not(feature = "kv_unstable"))]
pub type Value<'a> = str;

mod sealed {
    /// Types for the `kv` argument.
    pub trait KVs<'a> {
        fn into_kvs(self) -> Option<&'a [(&'a str, &'a super::Value<'a>)]>;
    }
}

// Types for the `kv` argument.

impl<'a> KVs<'a> for &'a [(&'a str, &'a Value<'a>)] {
    #[inline]
    fn into_kvs(self) -> Option<&'a [(&'a str, &'a Value<'a>)]> {
        Some(self)
    }
}

impl<'a> KVs<'a> for () {
    #[inline]
    fn into_kvs(self) -> Option<&'a [(&'a str, &'a Value<'a>)]> {
        None
    }
}

// Log implementation.

fn log_impl(
    args: Arguments,
    level: Level,
    &(target, module_path, file): &(&str, &'static str, &'static str),
    line: u32,
    kvs: Option<&[(&str, &Value)]>,
) {
    #[cfg(not(feature = "kv_unstable"))]
    if kvs.is_some() {
        panic!(
            "key-value support is experimental and must be enabled using the `kv_unstable` feature"
        )
    }

    let mut builder = Record::builder();

    builder
        .args(args)
        .level(level)
        .target(target)
        .module_path_static(Some(module_path))
        .file_static(Some(file))
        .line(Some(line));

    #[cfg(feature = "kv_unstable")]
    builder.key_values(&kvs);

    crate::logger().log(&builder.build());
}

pub fn log<'a, K>(
    args: Arguments,
    level: Level,
    target_module_path_and_file: &(&str, &'static str, &'static str),
    line: u32,
    kvs: K,
) where
    K: KVs<'a>,
{
    log_impl(
        args,
        level,
        target_module_path_and_file,
        line,
        kvs.into_kvs(),
    )
}

pub fn enabled(level: Level, target: &str) -> bool {
    crate::logger().enabled(&Metadata::builder().level(level).target(target).build())
}

/// This function is not intended to be actually called by anyone. The intention of this function is for forcing the
/// compiler to generate monomorphizations of the `log` function, so that they can be shared by different downstream
/// crates.
///
/// The idea is that with [`share-generics`](https://github.com/rust-lang/rust/pull/48779), downstream crates can reuse
/// generic monomorphizations from upstream crates, but not siblings crates. So it is best to instantiate these
/// monomorphizations in the `log` crate, so downstream crates are guaranteed to be able to share them.
pub fn instantiate_log_function() -> usize {
    struct State(usize);

    impl State {
        fn add<'a, K>(&mut self)
        where
            K: KVs<'a>,
        {
            self.0 ^= log::<K> as usize;
        }
    }

    let mut state = State(0);

    state.add::<&[(&str, &Value)]>();
    state.add::<()>();

    state.0
}
