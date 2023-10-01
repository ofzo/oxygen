use oxygen::runtime::OxygenRuntime;
use std::{env, fs::read, path::Path};

#[test]
fn test_elem_2_wasm() {
    let mut rt = OxygenRuntime::default();

    let url = Path::new("./testsuite/valid/elem.2.wasm");
    let root = env::current_dir().unwrap();
    // let root = root.parent().unwrap().parent().unwrap();
    let url = root.join(url);
    let url = url.canonicalize().unwrap();
    let buf = read(url).unwrap();
    let r = rt.load(buf);
    let r = if let Some(v) = r.err() {
        println!("error: {v}");
        false
    } else {
        true
    };
    assert!(r, "Failed to load wasm elem.2.wasm");
}
