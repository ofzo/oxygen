use anyhow::Context;
use oxygen::runtime::{
    decoder::{ImportKind, WasmModule, WasmValue},
    OxygenRuntime,
};
use std::{collections::HashMap, fs::read, path::Path, process};

use clap::{Args, Parser, Subcommand};

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    Run(RunArgs),
    Inspect(RunArgs),
}

#[derive(Debug, Args)]
struct RunArgs {
    url: String,
}

fn main() -> anyhow::Result<()> {
    let cmd = Arguments::parse();

    match cmd.command {
        Command::Run(args) => {
            let url = Path::new(&args.url);
            let buf = read(url).context(format!("can't read file {:?}", url))?;

            let mut rt = OxygenRuntime::default();
            rt.load(buf)?;
            for wasm in &mut rt.modes {
                let mut import_object = HashMap::new();
                let mut wasi_snapshot_preview1 = HashMap::new();
                wasi_snapshot_preview1.insert(
                    format!("fd_write"),
                    ImportKind::Func(wasi_snapshot_preview1_fd_write),
                );
                wasi_snapshot_preview1.insert(
                    format!("proc_exit"),
                    ImportKind::Func(wasi_snapshot_preview1_proc_exit),
                );
                import_object.insert(format!("wasi_snapshot_preview1"), wasi_snapshot_preview1);

                wasm.instance(Some(import_object))?;
                wasm.start()?;
            }
        }
        Command::Inspect(args) => {
            let url = Path::new(&args.url);
            let buf = read(url).context(format!("can't read file {:?}", url))?;

            let mut rt = OxygenRuntime::default();
            rt.load(buf)?;
            for wasm in &mut rt.modes {
                println!("{:?}", url.display());
                println!("{}", wasm);
            }
        }
    };

    Ok(())
}

pub fn wasi_snapshot_preview1_fd_write(
    wasm: &mut WasmModule,
    arg: &Vec<WasmValue>,
) -> Vec<WasmValue> {
    let arg = (arg[0], arg[1], arg[2], arg[3]);
    let mem = &mut wasm.mem[0];
    match arg {
        (
            WasmValue::I32(_fd),
            WasmValue::I32(offset),
            WasmValue::I32(len),
            WasmValue::I32(nwritten),
        ) => {
            let mut offset = offset;
            let mut data = vec![];
            let mut num = 0;
            for _ in 0..len {
                // let oft = offset >> 2;
                let mut ptr = [0; 4];
                for k in 0..4 {
                    ptr[k] = mem[offset as usize + k];
                }
                let ptr = u32::from_le_bytes(ptr);
                let mut l = [0; 4];
                for k in 4..8 {
                    l[k - 4] = mem[offset as usize + k];
                }
                let l = u32::from_le_bytes(l);
                offset += 8;
                for j in 0..l {
                    let p = ptr + j;
                    data.push(mem[p as usize]);
                }
                num += l;
            }
            let num = num.to_le_bytes();
            for (i, v) in num.iter().enumerate() {
                mem[nwritten as usize + i] = *v;
            }
            let s = String::from_utf8(data).unwrap();
            println!("{s}");
        }
        _ => {}
    }
    return vec![WasmValue::I32(0)];
}

pub fn wasi_snapshot_preview1_proc_exit(
    _wasm: &mut WasmModule,
    arg: &Vec<WasmValue>,
) -> Vec<WasmValue> {
    let code = arg[0];
    match code {
        WasmValue::I32(code) => process::exit(code),
        _ => {}
    }
    return vec![WasmValue::I32(0)];
}

#[test]
fn test_run() {
    use std::collections::HashMap;
    use std::{env, fs::read, path::Path};

    let mut rt = OxygenRuntime::default();

    let url = Path::new("./examples/fib.c.wasm");
    let root = env::current_dir().unwrap();
    let url = root.join(url);
    let url = url.canonicalize().unwrap();
    let buf = read(url).unwrap();
    let r = rt.load(buf);

    for wasm in &mut rt.modes {
        // println!("{}", wasm);
        let mut import_object = HashMap::new();
        let mut wasi_snapshot_preview1 = HashMap::new();
        wasi_snapshot_preview1.insert(
            format!("fd_write"),
            ImportKind::Func(wasi_snapshot_preview1_fd_write),
        );
        wasi_snapshot_preview1.insert(
            format!("proc_exit"),
            ImportKind::Func(wasi_snapshot_preview1_proc_exit),
        );
        import_object.insert(format!("wasi_snapshot_preview1"), wasi_snapshot_preview1);
        wasm.instance(Some(import_object)).unwrap();

        let _ = wasm.start();
    }
    let r = if let Some(v) = r.err() {
        println!("error: {v}");
        false
    } else {
        true
    };
    assert!(r, "Failed to load wasm elem.2.wasm");
}
