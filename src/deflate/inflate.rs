// Copyright 2016 compress Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io::{self, Read};
use std::collections::VecDeque;

use deflate::huffman;
use reader::BitReader;

#[derive(Clone, Copy, Debug)]
struct Block {
    bfinal: bool,
    btype: BlockType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum BlockType {
    NoCompression(u16),
    FixedHuffman,
    DynamicHuffman,
    Reserved,
}

pub struct Inflater<R> {
    reader: BitReader<R>,
    lookback: VecDeque<u8>,
    block: Option<Block>,
    litlentable: Option<huffman::Table>,
    disttable: Option<huffman::Table>,
    backread: Option<(usize, usize)>,
    done: bool,
}

impl<R: Read> Inflater<R> {
    pub fn new(reader: R) -> Self {
        Inflater {
            reader: BitReader::new(reader),
            lookback: VecDeque::with_capacity(32 * 1024),
            block: None,
            litlentable: None,
            disttable: None,
            backread: None,
            done: false,
        }
    }

    fn read_header(&mut self) -> io::Result<()> {
        let bfinal = try!(self.reader.read_u8(1)) == 1;

        let btype = match try!(self.reader.read_u8(2)) {
            0b00 => {
                self.reader.jump_to_byte_boundary();
                let len = try!(self.reader.read_u16(16));
                let nlen = try!(self.reader.read_u16(16));

                if !len != nlen {
                    panic!("len ({:016b}) and nlen ({:016b}) don't match", len, nlen);
                }

                BlockType::NoCompression(len)
            },
            0b01 => BlockType::FixedHuffman,
            0b10 => BlockType::DynamicHuffman,
            0b11 => panic!("reserved block type"), // TODO: shouldn't panic
            _ => unreachable!(), // TODO: actually unreachable?
        };

        // println!("beginning block of type {:?}", btype);
        
        self.block = Some(Block {
            bfinal: bfinal,
            btype: btype,
        });

        Ok(())
    }

    fn read_codelengths(&mut self, lengths: usize, codelengthtable: &huffman::Table) -> io::Result<Vec<u8>> {
        let mut codelengths = vec![0; lengths];
        let mut run_val = None;
        let mut run_len = 0;
        let mut i = 0;
        while i < lengths {
            if run_len > 0 {
                codelengths[i] = run_val.unwrap();
                run_len -= 1;
                i += 1;
            } else {
                let sym = try!(self.reader.read_table(&codelengthtable));
                match sym {
                    0...15 => {
                        codelengths[i] = sym as u8;
                        run_val = Some(sym as u8);
                        i += 1;
                    }
                    16 => {
                        assert!(run_val.is_some());
                        run_len = try!(self.reader.read_u8(2)) + 3;
                    }
                    17 => {
                        run_val = Some(0);
                        run_len = try!(self.reader.read_u8(3)) + 3;
                    }
                    18 => {
                        run_val = Some(0);
                        run_len = try!(self.reader.read_u8(7)) + 11;
                    }
                    _ => unreachable!(),
                }
            }
        }
        assert_eq!(run_len, 0);

        Ok(codelengths)
    }

    fn read_dynamic_trees(&mut self) -> io::Result<()> {
        let lit_len_code_count = (try!(self.reader.read_u8(5)) as usize) + 257;
        let distance_code_count = (try!(self.reader.read_u8(5)) as usize) + 1;

        // Read the code length code lengths
        let code_len_code_count = (try!(self.reader.read_u8(4)) as usize) + 4;

        // println!("beginning to read code lengths at bit {}", self.reader.bit_position());

        let mut codelencodelen = vec![0; 19];
        codelencodelen[16] = try!(self.reader.read_u8(3));
        codelencodelen[17] = try!(self.reader.read_u8(3));
        codelencodelen[18] = try!(self.reader.read_u8(3));
        codelencodelen[0] = try!(self.reader.read_u8(3));
        for i in 0..(code_len_code_count - 4) {
            if i % 2 == 0 {
                codelencodelen[8 + i / 2] = try!(self.reader.read_u8(3));
            } else {
                codelencodelen[7 - i / 2] = try!(self.reader.read_u8(3));
            }
        }

        let codelentable = huffman::make_table(7, codelencodelen.as_slice());
        // println!("made codelen table: {:?}", codelentable);
        
        let codelengths = try!(self.read_codelengths(lit_len_code_count + distance_code_count, &codelentable));
        let (litlenlengths, distlengths) = codelengths.split_at(lit_len_code_count);

        self.litlentable = Some(huffman::make_table(10, litlenlengths));
        // println!("made lit/len table: {:?}", self.litlentable);

        self.disttable = if distance_code_count == 1 && distlengths[0] == 0 {
            None
        } else {
            let mut one_count = 0;
            let mut other_positive_count = 0;
            for &x in distlengths {
                if x == 1 {
                    one_count += 1
                } else if x > 1 {
                    other_positive_count += 1;
                }
            }

            // println!("make distance tree");
            if one_count == 1 && other_positive_count == 0 {
                let mut dummy_dist_lengths = distlengths.to_vec();
                dummy_dist_lengths.resize(32, 0);
                dummy_dist_lengths[31] = 1;
                Some(huffman::make_table(8, dummy_dist_lengths.as_slice()))
            } else {
                Some(huffman::make_table(8, distlengths))
            }
        };

        Ok(())
    }

    fn decode_run_length(&mut self, sym: u16) -> io::Result<u16> {
        Ok(match sym {
            257...264 => sym - 254,
            265...284 => {
                let extra_bits = (sym - 261) / 4;
                (((sym - 265) % 4 + 4) << extra_bits) + 3 + try!(self.reader.read_u16(extra_bits as usize))
            }
            285 => 258,
            _ => unreachable!(),
        })
    }

    fn decode_distance(&mut self, sym: u16) -> io::Result<u16> {
        Ok(match sym {
            0...3 => sym + 1,
            4...29 => {
                let extra_bits = sym / 2 - 1;
                ((sym % 2 + 2) << extra_bits) + 1 + try!(self.reader.read_u16(extra_bits as usize))
            }
            30...31 => panic!("reserved distance symbol: {}", sym),
            _ => unreachable!(),
        })
    }

    fn read_literal_or_length(&mut self) -> io::Result<u16> {
        self.reader.read_table(self.litlentable.as_ref().unwrap())
    }

    fn next_byte(&mut self) -> io::Result<Option<u8>> {
        if let Some((dist, run)) = self.backread {
            let b = *self.lookback.get(dist-1).expect("lookback too small");
            if run == 1 {
                self.backread = None;
            } else {
                self.backread = Some((dist, run - 1));
            }
            self.lookback.truncate(32 * 1024 - 1);
            self.lookback.push_front(b);
            return Ok(Some(b));
        }

        if self.block.is_none() {
            try!(self.read_header());
        }

        match self.block.expect("no block?").btype {
            BlockType::DynamicHuffman => {
                if self.litlentable.is_none() {
                    try!(self.read_dynamic_trees());
                }

                match try!(self.read_literal_or_length()) {
                    lit @ 0...255 => {
                        self.lookback.truncate(32 * 1024 - 1);
                        self.lookback.push_front(lit as u8);
                        return Ok(Some(lit as u8));
                    }
                    256 => {
                        // println!("end of block");
                        if self.block.as_ref().expect("no table?").bfinal {
                            // println!("Done!");
                            self.done = true;
                        }
                        self.block = None;
                        self.litlentable = None;
                        self.disttable = None;

                        if self.done {
                            return Ok(None);
                        }

                        return self.next_byte();
                    },
                    len @ 257...285 => {
                        if self.disttable.is_none() {
                            panic!("distance length encountered, but no distance tree");
                        }

                        let run = try!(self.decode_run_length(len));
                        assert!(run >= 3 && run <= 258);
                        let dist_code = try!(self.reader.read_table(self.disttable.as_ref().expect("dist table didn't exist")));
                        let dist = try!(self.decode_distance(dist_code));

                        self.backread = Some((dist as usize, run as usize));
                        return self.next_byte();
                    }
                    other => panic!("literal/length code {} shouldn't happen", other),
                }
            }
            _ => unimplemented!(),
        }

        unreachable!();
    }
}

impl<R: Read> Read for Inflater<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done {
            return Ok(0);
        }

        let mut i = 0;
        for b in buf.iter_mut() {
            match try!(self.next_byte()) {
                Some(data) => {
                    *b = data;
                    i += 1;
                }
                None => return Ok(i),
            }
        }

        Ok(i)
    }
}
