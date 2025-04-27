use std::{
    collections::HashMap, env::{current_dir, home_dir}, fs::{read_dir, File}, io::Read, path::PathBuf, process::exit, sync::{LazyLock, Mutex},
    thread,
};

use clap::Parser;
use anyhow::{anyhow, Result};
use mlua::Lua;

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Hash, Clone)]
enum FileType {
    #[default]
    GenericFile,
    GenericDir,
    OtherDir(String),
    OtherFile(String),
}

static MAP : LazyLock<Mutex< HashMap<FileType, String>>> = LazyLock::new(||{ 
    use FileType as FT;
    return Mutex::new(HashMap::from([
        (FT::GenericFile, "\u{f0214}".to_string()),
        (FT::GenericDir,  "\u{f4d3}".to_string()),
    ]));
});

static LUA: LazyLock<Mutex<Lua>> = LazyLock::new( || Mutex::new(Lua::new()) );

#[allow(dead_code)]
fn config_dir() -> PathBuf {
    let mut config_dir = match home_dir(){
        Some(s) => s,
        None => {
            exit(-1);
        },
    };

    config_dir.push(".config");
    config_dir.push("minexp");
    config_dir.push("init");
    config_dir.set_extension("lua");

    return config_dir;
}

#[derive(Parser, Default)]
#[command(version, about, long_about = None)]
struct Options {
    #[arg(default_value = "config_dir")]
    config: PathBuf,

    #[arg(long, short, default_value_t=false)]
    debug: bool,

    #[arg(long, short, default_value_t=false)]
    verbose: bool,
}

fn load_formats(_: &Lua, tb: mlua::Table) -> mlua::Result<()>{
    let mut map = MAP.lock().map_err(|err| mlua::Error::RuntimeError(err.to_string()) )?;

    let pairs = tb.pairs::<String, String>();

    for kv in pairs{
        let (k, mut v) = kv?;
        let pat : Vec<&str> = k.split(":").collect();
        if pat.len() > 2 {
            return Err(mlua::Error::RuntimeError("bad entry format".to_string()));
        }

        if pat.len() == 1 {
            match pat[0] {
                "file" => map.get_mut(&FileType::GenericFile),
                "dir" => map.get_mut(&FileType::GenericDir),
                _ => return Ok(()),
            }.replace(&mut v);
            continue;
        }

        let ty = pat[0];
        let ident = pat[1];

        map.insert( match ty {
            "file" => FileType::OtherFile(ident.to_string()),
            "dir" => FileType::OtherDir(ident.to_string()),
            _ => return Ok(()),
        }, v);
    }

    Ok(())
}

fn format_file(path: PathBuf) -> Option<String>{
    let map = MAP.lock().ok()?;

    let extension = path.extension()?.to_str()?.to_string();
    let ft = FileType::OtherFile(extension.clone());

    return map.get(&ft).and_then(|f| Some(f.clone()));
}

fn run_lua(path: PathBuf) -> Result<()> {
    let lua = LUA.lock().map_err(|err| anyhow!(err.to_string()) )?;

    let load_format_function = lua.create_function(load_formats)?;
    lua.globals().set("load_formats", load_format_function)?;
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    lua.load(&buf).exec()?;

    Ok(())
}

fn print_data(opts: &Options) -> Result<()>{

    if opts.debug {
        let map = MAP.lock().unwrap();
        println!("config: {}\n", opts.config.to_str().unwrap());

        println!("formats:");
        for (k, v) in map.iter() {
            println!("{v} {k:?}");
        }
        println!();
    }

    let path = current_dir()?;
    let dirs = read_dir(path)?;

    for entry in dirs{
        if let Ok(entry) = entry {
            if entry.file_type()?.is_file() {
                let format = match format_file(entry.path()){
                    Some(k) => k,
                    None => {
                        let map = MAP.lock().unwrap();
                        map.get(&FileType::GenericFile).unwrap().clone()
                    },
                };
                let name = entry.file_name().into_string().unwrap();
                println!("{name:20} {format}");
            }
            else{
            }
        }
    }

    return Ok(());
}


fn main() -> Result<()> {
    let opts = Options::parse();
    let pth_config = opts.config.clone();

    run_lua(pth_config)?;
    print_data(&opts)?;


    Ok(())
}
