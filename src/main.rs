// Copyright 2016 compress Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![feature(alloc_system)]
extern crate alloc_system;

extern crate deflate;

use std::io::{self, copy, Read};

use deflate::Inflater;

fn main() {
    let stdin = io::stdin();
    let mut rf = stdin.lock();

    let mut header: [u8; 10] = [0; 10];
    rf.read(&mut header).expect("couldn't read header");
    assert_eq!((header[0], header[1]), (0x1F, 0x8B));
    assert_eq!(header[2], 8);

    let mut inflater = Inflater::new(&mut rf);

    let mut wf = io::stdout();
    copy(&mut inflater, &mut wf).expect("couldn't copy data");
}
