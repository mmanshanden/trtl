
use crate::lang::ast::{Program, Stmt, Entry};

fn bencode_str(out: &mut Vec<u8>, value: String) {
    let mut bytes: Vec<u8> = value.bytes().collect();
    let mut len: Vec<u8> = bytes.len().to_string().bytes().collect();

    out.append(&mut len);
    out.push(b':');
    out.append(&mut bytes);
}

fn bencode_entry(out: &mut Vec<u8>, entry: Entry) {
    match entry {
        Entry::Func(name, args, body) => {
            out.push(b'd');
            bencode_str(out, "name".to_string());
            bencode_str(out, name);
            out.push(b'l');
            for arg in args {
                bencode_str(out, arg);
            }
            out.push(b'e');
            out.push(b'e');
        },
        Entry::Stmt(stmt) => {
            out.push(b'l');
            out.push(b'e');
        }
    }
}

pub fn bencode_program(out: &mut Vec<u8>, program: Program) {
    out.push(b'l');

    for entry in program {
        bencode_entry(out, entry); 
    }

    out.push(b'e');
}

