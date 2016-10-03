// Copyright 2016 compress Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io::{self, Read};

use deflate::huffman;

pub struct BitReader<R> {
    reader: R,
    bitbuf: u32,
    bitbuf_remaining: usize,
    bytes_read: usize,
}

impl<R: Read> BitReader<R> {
    pub fn new(reader: R) -> Self {
        BitReader {
            reader: reader,
            bitbuf: 0,
            bitbuf_remaining: 0,
            bytes_read: 0,
        }
    }

    pub fn bit_position(&self) -> usize {
        self.bytes_read * 8 - self.bitbuf_remaining
    }

    pub fn read_u8(&mut self, bits: usize) -> io::Result<u8> {
        // let mut out = 0;

        // for i in 0..bits {
        //     out |= try!(self.read_bit()) << i;
        // }

        // Ok(out)

        self.read_u16(bits).map(|c| c as u8)
    }

    pub fn read_u16(&mut self, bits: usize) -> io::Result<u16> {
        let num = try!(self.peek_u16(bits));
        self.bitbuf >>= bits;
        self.bitbuf_remaining -= bits;

        Ok(num)
    }

    fn peek_u16(&mut self, bits: usize) -> io::Result<u16> {
        try!(self.need(bits));
        Ok((self.bitbuf & ((1 << bits) - 1)) as u16)
    }

    // fn read_bit(&mut self) -> io::Result<u8> {
    //     if self.bitbuf_remaining == 0 {
    //         self.bitbuf = try!(self.read_buf_byte());
    //         self.bitbuf_remaining = 8;
    //     }
    //     self.bitbuf_remaining -= 1;
    //     Ok((self.bitbuf >> (7 - self.bitbuf_remaining)) & 1)
    // }

    fn need(&mut self, bits: usize) -> io::Result<()> {
        assert!(bits <= 24); // TODO: Can this be 25?

        while self.bitbuf_remaining < bits {
            self.bitbuf |= (try!(self.read_buf_byte()) as u32) << self.bitbuf_remaining;
            self.bitbuf_remaining += 8;
        }

        Ok(())
    }
    
    fn read_buf_byte(&mut self) -> io::Result<u8> {
        let mut bytebuf: [u8; 1] = [0];
        let nread = try!(self.reader.read(&mut bytebuf));
        assert_eq!(nread, 1);
        self.bytes_read += 1;
        Ok(bytebuf[0])
    }

    pub fn jump_to_byte_boundary(&mut self) {
        let skip_bits = self.bitbuf_remaining % 8;
        self.bitbuf >>= skip_bits;
        self.bitbuf_remaining -= skip_bits;
    }

    pub fn read_table(&mut self, table: &huffman::Table) -> io::Result<u16> {
        self.read_table_contents(table.bits, table.contents.as_slice())
    }

    fn read_table_contents(&mut self, bits: u8, contents: &[huffman::TableEntry]) -> io::Result<u16> {
        // let mut lookup_rev = try!(self.peek_u16(bits as usize));
        // let mut lookup = 0;
        // for _ in 0..bits {
        //     lookup <<= 1;
        //     lookup |= lookup_rev & 1;
        //     lookup_rev >>= 1;
        // }
        let mut lookup = try!(self.peek_u16(bits as usize));
        lookup = ((lookup >> 1) & 0x5555) | ((lookup & 0x5555) << 1);
        lookup = ((lookup >> 2) & 0x3333) | ((lookup & 0x3333) << 2);
        lookup = ((lookup >> 4) & 0x0F0F) | ((lookup & 0x0F0F) << 4);
        lookup = ((lookup >> 8) & 0x00FF) | ((lookup & 0x00FF) << 8);
        lookup >>= 16 - bits;
        // println!("looking for {0:01$b} ({0}) in {2:?}", lookup, bits as usize, contents);
        match contents[lookup as usize] {
            huffman::TableEntry::Symbol(code, len) => {
                self.bitbuf >>= len;
                self.bitbuf_remaining -= len as usize;
                Ok(code)
            }
            huffman::TableEntry::Table(ref subcontents) => {
                self.bitbuf >>= bits;
                self.bitbuf_remaining -= bits as usize;
                self.read_table_contents(bits, subcontents.as_slice())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BitReader;

    #[test]
    fn test_read_u8() {
        let data = &[0b1001_0110, 0b0101_1010][..];
        let mut reader = BitReader::new(data);
        assert_eq!(reader.read_u8(3).unwrap(), 0b110);
        assert_eq!(reader.read_u8(3).unwrap(), 0b010);
    }
}
