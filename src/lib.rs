// Copyright 2016 compress Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![feature(deque_extras)]
#![feature(alloc_system)]
extern crate alloc_system;

mod deflate;
mod reader;

pub use deflate::inflate::Inflater;
