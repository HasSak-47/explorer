use std::{
    collections::HashMap, env::{current_dir, home_dir}, fs::{read_dir, File}, io::Read, path::PathBuf, process::exit, sync::{LazyLock, Mutex, Once, OnceLock},
    thread,
};

use clap::Parser;
use anyhow::{anyhow, Result};
use mlua::{Function, Lua, Table};

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Hash, Clone)]
enum FileType {
    #[default]
    GenericFile,
    GenericDir,
    OtherDir(String),
    OtherFile(String),
}

static MAP : LazyLock<Mutex< HashMap<FileType, mlua::Function>>> = LazyLock::new(||{ 
    use FileType as FT;

    let def_file: Function = LUA.lock().unwrap().load("function(name, path, tick) return '󰈔 ' .. name end").eval().unwrap();
    let def_dir: Function = LUA.lock().unwrap().load("function(name, path, tick) return ' ' .. name end").eval().unwrap();

    return Mutex::new(HashMap::from([
        (FT::GenericFile, def_file),
        (FT::GenericDir, def_dir),
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

#[derive(Parser, Default, Clone)]
enum Mode{
    #[default]
    List,
    Explorer,
}

#[derive(Parser, Default, Clone)]
#[command(version, about, long_about = None)]
struct Options {
    #[arg(long, short, default_value = "config_dir")]
    config: PathBuf,

    #[arg(long, short, default_value_t=false)]
    debug: bool,

    #[arg(long, short, default_value_t=false)]
    verbose: bool,

    #[command(subcommand)]
    mode: Mode,
}

static OPTIONS : OnceLock<Options> = OnceLock::new();

fn _format_file(map: &HashMap<FileType, Function>, path: PathBuf) -> Option<&Function>{
    let extension = path.extension()?.to_str()?.to_string();
    let ft = FileType::OtherFile(extension.clone());

    return map.get(&ft);
}

fn format_file(path: PathBuf) -> Function{
    let map = MAP.lock().unwrap();
    let format = _format_file(&map, path)
        .unwrap_or(map.get(&FileType::GenericFile).unwrap());

    return format.clone();
}

fn get_options<'a>() -> &'a Options{
    OPTIONS.get_or_init( Options::parse )
}

fn load_formats(_: &Lua, tb: mlua::Table) -> mlua::Result<()>{
    let mut map = MAP.lock().map_err(|err| mlua::Error::RuntimeError(err.to_string()) )?;
    let file_formats : Table = tb.get("file")?;

    for kv in file_formats.pairs(){
        let (k, v) : (String, Function) = kv?;
        map.insert(FileType::OtherFile(k), v);
    }

    Ok(())
}

fn init_lua() -> Result<()> {
    let lua = LUA.lock().map_err(|err| anyhow!(err.to_string()) )?;
    let path = &get_options().config;

    let load_format_function = lua.create_function(load_formats)?;
    lua.globals().set("load_formats", load_format_function)?;

    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    lua.load(&buf).exec()?;

    Ok(())
}

fn print_data() -> Result<()>{
    let path = current_dir()?;
    let dirs = read_dir(path)?;
    for entry in dirs{
        if let Ok(entry) = entry {
            if entry.file_type()?.is_file() {
                let formatter = format_file(entry.path());

                let name = entry.file_name().into_string().unwrap();
                let path = entry.path().to_str().unwrap().to_string();

                let format : String = formatter.call((name, path, 0))?;
                println!("{format}");
            }
            else{
            }
        }
    }

    return Ok(());
}

fn setup_lua() {
    let _init_map = MAP.lock();
}

fn main() -> Result<()> {
    OPTIONS.get_or_init( Options::parse );

    setup_lua();
    init_lua()?;
    match get_options().mode{
        Mode::List => print_data()? ,
        Mode::Explorer => {},
    }

    Ok(())
}
