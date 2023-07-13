//! # the totally sync variables.
//! ## features
//! - make any variable, pass it round!
//! - scoping? who needs it!
//! - only one mutable reference? pshaw!
//! - any numer of mutable references are allowed.
//
//! ## data races?
//! dont know what your talking about.
//!
//! its *fearless concurrency* remember? and theres no unsafe code!
//!
//! besides, miri says its fiine.
#![deny(unsafe_code)]
#![allow(dead_code)]
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::io::{BufReader, BufWriter};
use std::marker::PhantomData;
use std::ops::*;
use std::path::Path;

/// a variable stored on the filesystem (very secure 🔐)
#[doc(hidden)]
pub struct Var<T>(&'static str, PhantomData<T>);

impl<T> Var<T> {
    /// create a var
    pub fn new(name: &'static str) -> Self {
        Self(name, PhantomData::default())
    }
}

impl<T: Serialize> Var<T> {
    /// interior mutability!
    /// ```
    /// # use totally_sync_variable::*;
    /// let x = def!(x = 5u8);
    /// x.set(&3u8);
    /// ```
    pub fn set(&self, v: &T) {
        let f = BufWriter::new(
            fs::File::create(Path::new("/tmp/").join(self.0)).expect("become linux!"),
        );
        serde_json::to_writer(f, v).unwrap();
    }
}

impl<T: DeserializeOwned> Var<T> {
    /// procure the stored variable
    /// ```
    /// # use totally_sync_variable::*;
    /// let y = def!(y = 5u8);
    /// assert_eq!(y.get(), 5u8);
    /// ```
    pub fn get(&self) -> T {
        let f = BufReader::new(fs::File::open(Path::new("/tmp/").join(self.0)).expect("oh no"));
        serde_json::from_reader(f).unwrap()
    }
}

macro_rules! op {
    ($op: ident, $char: tt) => {paste::paste! {
        impl<T: [<$op>]<T, Output = T> + Clone + DeserializeOwned + Serialize> [<$op Assign>] for Var<T> {
            fn [<$op:lower _assign>](&mut self, rhs: Self) {
                self.set(&(self.get().clone() $char rhs.get().clone()));
            }
        }
    }};
}

op!(Add, +);
op!(Sub, -);
op!(Mul, *);
op!(Div, /);
op!(BitAnd, &);
op!(BitOr, |);
op!(BitXor, ^);
op!(Shl, <<);
op!(Shr, >>);

#[macro_export]
/// lil macro to define variables.
macro_rules! def {
    ($name:ident = $value:expr) => {{
        let v = $crate::Var::new(stringify!($name));
        v.set(&$value);
        v
    }};
}

#[macro_export]
/// macro to reference variables from anywhere.
macro_rules! refer {
    ($name:ident) => {
        $crate::Var::new(stringify!($name)).get()
    };
}

#[test]
fn it_works() {
    {
        let mut x = def!(x = 4u8);
        x += def!(tmp = 4u8);
        assert_eq!(x.get(), 8);
    }
    let v: u8 = refer!(x);
    assert_eq!(v, 8)
}
