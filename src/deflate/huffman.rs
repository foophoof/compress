// Copyright 2016 compress Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tree {
    Node(u16),
    Tree(Box<Tree>, Box<Tree>), // (0, 1)
}

#[derive(Clone, Debug)]
pub struct Table {
    pub bits: u8,
    pub contents: Vec<TableEntry>,
}

#[derive(Clone, Debug)]
pub enum TableEntry {
    Symbol(u16, u8),
    Table(Vec<TableEntry>),
    None,
}

fn insert_in_table(contents: &mut Vec<TableEntry>, bits: u8, code: u16, symbol: u16, length: u8) {
    if length <= bits {
        for i in 0..(1 << (bits - length)) {
            contents[((code << (bits - length)) + i) as usize] = TableEntry::Symbol(symbol as u16, length as u8);
        }
    } else {
        let prefix = code >> (length - bits);
        match contents[prefix as usize] {
            TableEntry::None => {
                let subcontents = vec![TableEntry::None; 1 << bits];
                contents[prefix as usize] = TableEntry::Table(subcontents);
            },
            TableEntry::Table(_) => {},
            _ => unreachable!(),
        };
        let mut subcontents = match contents[prefix as usize] {
            TableEntry::Table(ref mut subcontents) => subcontents,
            _ => unreachable!(),
        };
        insert_in_table(&mut subcontents, bits, code & ((1 << (length - bits)) - 1), symbol, length - bits);
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
        code = (code + bl_count[bits-1]) << 1;
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

    // for bits in 1..16 {
    //     for (sym, &length) in lengths.iter().enumerate() {
    //         if length == bits {
    //             for _ in 0..(1 << (16 - length)) {
    //                 contents.push(TableEntry::Symbol(sym as u16, length as u8));
    //             }
    //         }
    //     }
    // }

    Table {
        bits: bits,
        contents: contents,
    }
}

#[test]
fn test_make_table() {
    let table = make_table(2, &vec![2, 1, 3, 3][..]);
    println!("table: {:?}", table);
    assert!(false);
    // assert_eq!(table[0b0000_0000_0000_0000], TableEntry { symbol: 1, length: 1 });
    // assert_eq!(table[0b0100_0000_0000_0000], TableEntry { symbol: 1, length: 1 });
    // assert_eq!(table[0b1000_0000_0000_0000], TableEntry { symbol: 0, length: 2 });
    // assert_eq!(table[0b1100_0000_0000_0000], TableEntry { symbol: 2, length: 3 });
    // assert_eq!(table[0b1110_0000_0000_0000], TableEntry { symbol: 3, length: 3 });
}

pub fn make_tree(lengths: &[u8]) -> Tree {
    let mut nodes: Vec<Tree> = Vec::new();
    let mut i = 16;
    while i > 0 {
        i -= 1;
        if nodes.len() % 2 != 0 {
            // println!("nodes: {:?}, input: {:?}", nodes, lengths);
            panic!("canonical code does not represent huffman code tree");
        }
        let mut new_nodes = Vec::new();

        if i > 0 {
            for (symbol, &length) in lengths.iter().enumerate() {
                if length == i {
                    new_nodes.push(Tree::Node(symbol as u16))
                }
            }
        }

        let mut j = 0;
        while j < nodes.len() {
            new_nodes.push(Tree::Tree(Box::new(nodes[j].clone()), Box::new(nodes[j+1].clone())));
            j += 2;
        }

        nodes = new_nodes;
    }

    if nodes.len() != 1 {
        panic!("canonical code does not represent huffman code tree")
    }

    nodes[0].clone()



    // let mut bl_count: [u16; 16] = [0; 16];

    // for &length in lengths {
    //     if length == 0 {
    //         continue;
    //     }
    //     assert!(length <= 15);
    //     bl_count[length as usize] += 1;
    // }

    // let mut next_code: [u16; 16] = [0; 16];
    // let mut code = 0;
    // for bits in 1..16 {
    //     code = (code + bl_count[bits - 1]) << 1;
    //     next_code[bits] = code;
    // }

    // let mut codes = Vec::with_capacity(lengths.len());
    // for n in 0..lengths.len() {
    //     let len = lengths[n] as usize;
    //     if len != 0 {
    //         codes.push(Some(next_code[len]));
    //         next_code[len] += 1;
    //     } else {
    //         codes.push(None);
    //     }
    // }

    // make_tree_prefix(codes.as_slice(), lengths, 0, 0)
}

// // lengths: [2, 0, 3, 3]
// // codes: 00, x, 010, 011,
// fn make_tree_prefix(codes: &[Option<u16>], lengths: &[u8], prefix: u16, prefix_mask: u16) -> Tree {
//     if prefix_mask == 65535 {
//         return Tree::None;
//         // println!("lengths: {:?}", lengths);
//         // print!("codes: ");
//         // for n in 0..(codes.len()) {
//         //     match codes[n] {
//         //         Some(code) => print!("{0:01$b} ", code, lengths[n] as usize),
//         //         None => print!("x "),
//         //     }
//         // }
//         // println!("");
//         // panic!("uhhh…");
//     }

//     for n in 0..codes.len() {
//         if lengths[n] == 0 || codes[n] == None {
//             continue;
//         }

//         let mask = (1 << (lengths[n] as u16)) - 1;
//         if mask == prefix_mask && codes[n] == Some(prefix) {
//             return Tree::Node(n as u16)
//         }
//     }

//     let left = make_tree_prefix(codes, lengths, prefix << 1, (prefix_mask << 1) | 1);
//     let right = make_tree_prefix(codes, lengths, (prefix << 1) | 1, (prefix_mask << 1) | 1);

//     Tree::Tree(Box::new(left), Box::new(right))
// }

#[test]
fn test_make_tree() {
    let tree = make_tree(&vec![2, 1, 3, 3][..]);
    assert_eq!(
        tree,
        Tree::Tree(
            Box::new(Tree::Node(1)),
            Box::new(Tree::Tree(
                Box::new(Tree::Node(0)),
                Box::new(Tree::Tree(
                    Box::new(Tree::Node(2)),
                    Box::new(Tree::Node(3)),
                ))
            ))
        )
    );

    // println!("{:?}", make_tree(&vec![3, 3, 3, 3, 3, 2, 4, 4][..]));
    // make_tree(&vec![4, 0, 0, 6, 3, 4, 2, 4, 2, 5, 4, 5, 5, 0, 7, 0, 7][..]);
}
