cargo build --target wasm32-unknown-unknown --release 
move .\target\wasm32-unknown-unknown\release\trtl.wasm .\web\
basic-http-server -x .\web\
