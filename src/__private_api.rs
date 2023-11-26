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
    fmt_args: Arguments,
    const_args: &LikelyConstantArgs,
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
        .args(fmt_args)
        .level(const_args.level)
        .target(const_args.target)
        .module_path_static(Some(const_args.module_path))
        .file_static(Some(const_args.file))
        .line(Some(line));

    #[cfg(feature = "kv_unstable")]
    builder.key_values(&kvs);

    crate::logger().log(&builder.build());
}

// Group arguments that are likely constant together so that the compiler can reuse these arguments
// between different calls.
#[derive(Debug)]
pub struct LikelyConstantArgs<'a> {
    pub level: Level,
    pub target: &'a str,
    pub module_path: &'static str,
    pub file: &'static str,
}

pub fn log<'a, K>(fmt_args: Arguments, const_args: &LikelyConstantArgs, line: u32, kvs: K)
where
    K: KVs<'a>,
{
    log_impl(fmt_args, const_args, line, kvs.into_kvs())
}

pub fn enabled(level: Level, target: &str) -> bool {
    crate::logger().enabled(&Metadata::builder().level(level).target(target).build())
}
