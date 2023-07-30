rm -rf ./crates/decode/tests/*
if [[ ! -e  ./crates/decode/tests ]];
then
    mkdir ./crates/decode/tests
fi
touch ./crates/decode/tests/wasm.rs

echo "use oxygen::runtime::AsmccRuntime;" >> ./crates/decode/tests/wasm.rs
echo "use std::{env, fs::read, path::Path};" >> ./crates/decode/tests/wasm.rs

function gene(){
    name=$(echo $1 | sed 's/[\.-]/_/g')
    echo "
#[test]
fn test_$name() {
    let mut rt = AsmccRuntime::default();

    let url = Path::new(\"./testsuite/valid/$1\");
    let root = env::current_dir().unwrap();
    let root = root.parent().unwrap().parent().unwrap();
    let url = root.join(url);
    let url = url.canonicalize().unwrap();
    let buf = read(url).unwrap();
    let r = rt.load(buf);
    let r = if let Some(v) = r.err() {
        println!(\"error: {v}\");
        false
    } else {
        true
    };
    assert!(r, \"Failed to load wasm $1\");
}
" >> ./crates/decode/tests/wasm.rs
echo "Info: Generate $1 test success"
}


if [[ -z "$1" ]]
then
    cd testsuite
    bash ./extract-parts.sh
    cd ..
    rm -rf ./testsuite/valid/*.wat
    for file in $(ls ./testsuite/valid/)
    do
       gene $file
    done
else
    gene "$1"
fi
