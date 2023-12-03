//! WARNING: this is not part of the crate's public API and is subject to change at any time

use self::sealed::{KVs, Level as LevelTrait, Target};
use crate::{Level, LevelFilter, Metadata, Record};
use std::cmp::Ordering;
use std::fmt::Arguments;
pub use std::{file, format_args, line, module_path, stringify};

#[cfg(feature = "kv_unstable")]
pub type Value<'a> = dyn crate::kv::value::ToValue + 'a;

#[cfg(not(feature = "kv_unstable"))]
pub type Value<'a> = str;

mod sealed {
    /// Types for the `target` argument.
    pub trait Target<'a>: Copy {
        fn into_target(self, module_path: &'a &'a str) -> &'a &'a str;
    }

    /// Types for the `level` argument.
    pub trait Level: Copy {
        fn into_level(self) -> crate::Level;
    }

    /// Types for the `kv` argument.
    pub trait KVs<'a> {
        fn into_kvs(self) -> Option<&'a &'a [(&'a str, &'a super::Value<'a>)]>;
    }
}

// Types for the `target` argument.

/// Caller specified target explicitly.
impl<'a> Target<'a> for &'a &'a str {
    #[inline]
    fn into_target(self, _module_path: &'a &'a str) -> &'a &'a str {
        self
    }
}

/// Caller did not specified target.
impl<'a> Target<'a> for () {
    #[inline]
    fn into_target(self, module_path: &'a &'a str) -> &'a &'a str {
        module_path
    }
}

// Types for the `kvs` argument.

/// Caller specified key-value data explicitly.
impl<'a> KVs<'a> for &'a &'a [(&'a str, &'a Value<'a>)] {
    #[inline]
    fn into_kvs(self) -> Option<&'a &'a [(&'a str, &'a Value<'a>)]> {
        Some(self)
    }
}

/// Caller did not specify key-value data.
impl<'a> KVs<'a> for () {
    #[inline]
    fn into_kvs(self) -> Option<&'a &'a [(&'a str, &'a Value<'a>)]> {
        None
    }
}

// Types for the `level` argument.

/// The log level is dynamically determined.
impl LevelTrait for Level {
    #[inline]
    fn into_level(self) -> Level {
        self
    }
}

macro_rules! define_static_levels {
    ($(($name:ident, $level:ident),)*) => {$(
        #[derive(Clone, Copy, Debug)]
        pub struct $name;

        /// The log level is statically determined.
        impl LevelTrait for $name {
            #[inline]
            fn into_level(self) -> Level {
                Level::$level
            }
        }

        impl PartialEq<LevelFilter> for $name {
            #[inline]
            fn eq(&self, other: &LevelFilter) -> bool {
                self.into_level().eq(other)
            }
        }

        impl PartialOrd<LevelFilter> for $name {
            #[inline]
            fn partial_cmp(&self, other: &LevelFilter) -> Option<Ordering> {
                self.into_level().partial_cmp(other)
            }
        }
    )*};
}

define_static_levels![
    (LevelError, Error),
    (LevelWarn, Warn),
    (LevelInfo, Info),
    (LevelDebug, Debug),
    (LevelTrace, Trace),
];

// Log implementation.

/// Log arguments that are not generic.
#[derive(Debug)]
pub struct NonGenericArgs<'a> {
    pub module_path_and_file: &'static (&'static str, &'static str),
    pub line: u32,
    pub args: Arguments<'a>,
}

// Note that all argument types are selected to have sizes less than or equal to a single pointer size, which allows
// tail call optimizations to be applied.
fn log_impl(
    non_generic_args: &NonGenericArgs,
    target: &&str,
    level: Level,
    kvs: Option<&&[(&str, &Value)]>,
) {
    #[cfg(not(feature = "kv_unstable"))]
    if kvs.is_some() {
        panic!(
            "key-value support is experimental and must be enabled using the `kv_unstable` feature"
        )
    }

    let mut builder = Record::builder();

    builder
        .args(non_generic_args.args)
        .level(level)
        .target(target)
        .module_path_static(Some(non_generic_args.module_path_and_file.0))
        .file_static(Some(non_generic_args.module_path_and_file.1))
        .line(Some(non_generic_args.line));

    #[cfg(feature = "kv_unstable")]
    builder.key_values(&kvs);

    crate::logger().log(&builder.build());
}

// `#[inline(never)]` is used to prevent compiler from inlining this function so that the binary size could be kept as
// small as possible. Also, the argument types are carefully selected so that the performance cost of this function
// could be kept as low as possible.
#[inline(never)]
pub fn log<'a, T, L, K>(non_generic_args: &NonGenericArgs, target: T, level: L, kvs: K)
where
    T: Target<'a>,
    L: LevelTrait,
    K: KVs<'a>,
{
    // For all possible generic arguments combinations, tail call optimization can be applied to this function call.
    log_impl(
        non_generic_args,
        target.into_target(&non_generic_args.module_path_and_file.0),
        level.into_level(),
        kvs.into_kvs(),
    )
}

pub fn enabled(level: Level, target: &str) -> bool {
    crate::logger().enabled(&Metadata::builder().level(level).target(target).build())
}
