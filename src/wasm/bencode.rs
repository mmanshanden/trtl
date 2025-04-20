
use std::vec;

use super::highlight::{Fragment, Highlight, Line};

fn bencode_str(out: &mut Vec<u8>, bytes: &[u8]) {
    let mut len: Vec<u8> = bytes.len().to_string().bytes().collect();
    
    out.append(&mut len);
    out.push(b':');
    out.extend_from_slice(bytes);
}

fn bencode_fragment(out: &mut Vec<u8>, fragment: Fragment) {
    let mut buffer = fragment.value.as_bytes();
    let mut len = (buffer.len() + 2).to_string().bytes().collect();
    
    out.append(&mut len);
    out.push(b':');
    out.push(fragment.kind);
    out.push(fragment.hint);
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
    out.push(b'l');

    for line in highlight.lines {
        bencode_line(out, line); 
    } 

    out.push(b'e');
    out.push(b'l');

    for hint in highlight.hints {
        bencode_str(out, hint.as_bytes()); 
    } 

    out.push(b'e');
    out.push(b'e');
}

