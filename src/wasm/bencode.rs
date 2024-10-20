
use std::vec;

use crate::lang::ast::{Program, Stmt, Entry};

use super::{console_log, highlight::{Fragment, Highlight, Line}};

fn bencode_str(out: &mut Vec<u8>, bytes: &mut Vec<u8>) {
    let mut len: Vec<u8> = bytes.len().to_string().bytes().collect();
    
    out.append(&mut len);
    out.push(b':');
    out.append(bytes);
}


fn bencode_fragment(out: &mut Vec<u8>, fragment: Fragment) {
    let mut buffer = fragment.value.as_bytes();
    let mut len = (buffer.len() + 2).to_string().bytes().collect();
    
    out.append(&mut len);
    out.push(b':');
    out.push(fragment.kind);
    out.push(fragment.lint);
    out.extend_from_slice(&buffer);
}


fn bencode_line(out: &mut Vec<u8>, line: Line) {
    out.push(b'l');

    for fragment in line {
        bencode_fragment(out, fragment);
    }

    out.push(b'e');
}

pub fn benchode_highlight(out: &mut Vec<u8>, highlight: Highlight) {
    out.push(b'l');

    for line in highlight {
        bencode_line(out, line); 
    } 

    out.push(b'e');
}

