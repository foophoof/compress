// Copyright 2016 compress Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![feature(alloc_system)]
extern crate alloc_system;

extern crate deflate;

use std::fs::File;
use std::io::{copy, Read, BufReader, BufRead};

use deflate::Inflater;

// static OS_NAMES: [&'static str; 14] = [
//     "FAT",
//     "Amiga",
//     "VMS",
//     "Unix",
//     "VM/CMS",
//     "Atari TOS",
//     "HPFS",
//     "Macintosh",
//     "Z-System",
//     "CP/M",
//     "TOPS-20",
//     "NTFS",
//     "QDOS",
//     "Acorn RISCOS",
// ];

fn main() {
    for _ in 0..1 {
        do_thing()
    }
}

fn do_thing() {
    let mut rf = BufReader::new(File::open("xargo-in.gz").expect("couldn't open file"));

    // let mut total_header_length = 10;

    let mut header: [u8; 10] = [0; 10];
    rf.read(&mut header).expect("couldn't read header");
    assert_eq!((header[0], header[1]), (0x1F, 0x8B));
    assert_eq!(header[2], 8);

    // let flags = header[3] & 0xFF;

    // let mtime = (header[4] as u32) | (header[5] as u32) << 8 | (header[6] as u32) << 16 | (header[7] as u32) << 24;
    // if mtime != 0 {
    //     println!("last modified: {}", mtime);
    // }

    // match header[8] {
    //     2 => println!("maximum compression"),
    //     4 => println!("fastest compression"),
    //     c => println!("unknown compression: {}", c),
    // }

    // if header[9] < 14 {
    //     println!("Operating system: {}", OS_NAMES[header[9] as usize]);
    // } else if header[9] == 255 {
    //     println!("Operating system: unknown");
    // } else {
    //     println!("Operating system: really unknown");
    // };

    // let mut extra = None;
    // if flags & (1 << 2) != 0 {
    //     let mut xlenbuf: [u8; 2] = [0; 2];
    //     rf.read(&mut xlenbuf).expect("couldn't read xlen");
    //     let xlen = xlenbuf[0] as u16 | ((xlenbuf[1] as u16) << 8);
    //     extra = Some(vec![0; xlen as usize]);
    //     rf.read(&mut extra.as_mut().unwrap()).expect("couldn't read extra");
    //     total_header_length += 2 + xlen as usize;
    // }
    // println!("extra: {:?}", extra);

    // let name = if flags & (1 << 3) != 0 {
    //     let mut namev = Vec::new();
    //     rf.read_until(0, &mut namev).expect("couldn't read filename");
    //     total_header_length += namev.len();

    //     let name_str = std::str::from_utf8(&namev.as_slice()[0..(namev.len() - 1)]).expect("filename couldn't be parsed");

    //     Some(name_str.to_owned())
    // } else {
    //     None
    // };
    // println!("name: {:?}", name);

    // let comment = if flags & (1 << 4) != 0 {
    //     let mut comment = Vec::new();
    //     rf.read_until(0, &mut comment).expect("couldn't read comment");
    //     total_header_length += comment.len();
    //     Some(comment)
    // } else {
    //     None
    // };
    // println!("comment: {:?}", comment);

    // let crc16 = if flags & (1 << 1) != 0 {
    //     let mut crc16buf: [u8; 2] = [0; 2];
    //     rf.read(&mut crc16buf).expect("couldn't read xlen");
    //     total_header_length += 2;
    //     Some(crc16buf[0] as u16 | ((crc16buf[1] as u16) << 8))
    // } else {
    //     None
    // };
    // println!("crc16: {:?}", crc16);

    // println!("read {} bytes of header", total_header_length);

    let mut inflater = Inflater::new(&mut rf);

    let mut wf = File::create("xargo-out").expect("couldn't create file");
    copy(&mut inflater, &mut wf).expect("couldn't copy data");
}
