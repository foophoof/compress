// Copyright 2016 compress Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[derive(Clone, Debug)]
pub struct Table {
    pub bits: u8,
    pub contents: Vec<TableEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TableEntry {
    Symbol(u16, u8),
    Table(Vec<TableEntry>),
    None,
}

fn insert_in_table(contents: &mut Vec<TableEntry>, bits: u8, code: u16, symbol: u16, length: u8) {
    if length <= bits {
        for i in 0..(1 << (bits - length)) {
            contents[((code << (bits - length)) + i) as usize] = TableEntry::Symbol(symbol as u16,
                                                                                    length as u8);
        }
    } else {
        let prefix = code >> (length - bits);
        match contents[prefix as usize] {
            TableEntry::None => {
                let subcontents = vec![TableEntry::None; 1 << bits];
                contents[prefix as usize] = TableEntry::Table(subcontents);
            }
            TableEntry::Table(_) => {}
            _ => unreachable!(),
        };
        let mut subcontents = match contents[prefix as usize] {
            TableEntry::Table(ref mut subcontents) => subcontents,
            _ => unreachable!(),
        };
        insert_in_table(&mut subcontents,
                        bits,
                        code & ((1 << (length - bits)) - 1),
                        symbol,
                        length - bits);
    }
}

pub fn make_table(bits: u8, lengths: &[u8]) -> Table {
    let mut bl_count = [0; 15];
    for &length in lengths {
        if length == 0 {
            continue;
        }
        bl_count[length as usize] += 1;
    }

    let mut next_code = [0; 16];
    let mut code = 0;
    for bits in 1..16 {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }

    let mut codes = Vec::with_capacity(lengths.len());
    for &length in lengths {
        if length != 0 {
            codes.push(Some(next_code[length as usize]));
            next_code[length as usize] += 1;
        } else {
            codes.push(None);
        }
    }

    let mut contents = vec![TableEntry::None; 1 << bits];

    for (symbol, &code_opt) in codes.iter().enumerate() {
        let code = match code_opt {
            Some(code) => code,
            None => continue,
        };

        let length = lengths[symbol];
        insert_in_table(&mut contents, bits, code, symbol as u16, length);
    }

    Table {
        bits: bits,
        contents: contents,
    }
}

#[test]
fn test_make_table() {
    let table = make_table(2, &vec![2, 1, 3, 3][..]);
    assert_eq!(table.bits, 2);
    assert_eq!(table.contents.len(), 4);
    assert_eq!(table.contents[0b00], TableEntry::Symbol(1, 1));
    assert_eq!(table.contents[0b01], TableEntry::Symbol(1, 1));
    assert_eq!(table.contents[0b10], TableEntry::Symbol(0, 2));

    match table.contents[0b11] {
        TableEntry::Table(ref subcontents) => {
            assert_eq!(subcontents.len(), 4);
            assert_eq!(subcontents[0b00], TableEntry::Symbol(2, 1));
            assert_eq!(subcontents[0b01], TableEntry::Symbol(2, 1));
            assert_eq!(subcontents[0b10], TableEntry::Symbol(3, 1));
            assert_eq!(subcontents[0b11], TableEntry::Symbol(3, 1));
        }
        _ => {
            panic!("expected subtable at table.contents[0b11], but was {:?}",
                   table.contents[0b11])
        }
    }
}
