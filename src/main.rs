mod list;
mod util;

use list::List;
use util::*;

use std::{
    collections::HashMap, env::{current_dir, home_dir}, fs::{read_dir, File}, io::Read, path::PathBuf, process::exit, sync::{LazyLock, Mutex, OnceLock}, usize
};

use clap::{Parser, ValueEnum};
use anyhow::{anyhow, Result};
use mlua::{Either, Function, Lua, Table};

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
    let lua = LUA.lock().unwrap();

    let def_file: Function = lua.load("function(name, path, tick) return '󰈔 ' .. name end").eval().unwrap();
    let def_dir: Function = lua.load("function(name, path, tick) return ' ' .. name .. '/' end").eval().unwrap();

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

#[derive(Parser)]
enum Mode{
    List(List),
    Explorer,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Options {
    #[arg(long, short, default_value = "config_dir")]
    config: PathBuf,

    #[arg(long, short, default_value_t=false)]
    debug: bool,

    #[arg(long, short, default_value_t=false)]
    verbose: bool,

    #[arg(value_enum, long, short, default_value_t=SortBy::Name)]
    sort_by: SortBy,


    #[command(subcommand)]
    mode: Mode,
}

static OPTIONS : OnceLock<Options> = OnceLock::new();

fn _format_file<'a>(map: &'a HashMap<FileType, Function>, path: &PathBuf) -> Option<&'a Function>{
    let extension = path.extension()?.to_str()?.to_string();
    let ft = FileType::OtherFile(extension.clone());

    return map.get(&ft);
}

fn format_file(path: &PathBuf) -> Function{
    let map = MAP.lock().unwrap();
    let format = _format_file(&map, path)
        .unwrap_or(map.get(&FileType::GenericFile).unwrap());

    return format.clone();
}

fn format_dir(_: &PathBuf) -> Function{
    let map = MAP.lock().unwrap();
    let format = map.get(&FileType::GenericDir).unwrap();

    return format.clone();
}

fn get_options<'a>() -> &'a Options{
    OPTIONS.get_or_init( Options::parse )
}

fn get_formats(_: &Lua, tb: mlua::Table) -> mlua::Result<()>{
    let mut map = MAP.lock().map_err(|err| mlua::Error::RuntimeError(err.to_string()) )?;
    let file_formats : Table = tb.get("file")?;

    match file_formats.get(1){
        Ok(default) => {
            *map.get_mut(&FileType::GenericFile).unwrap() = default;
        }
        Err(_) => {},
    }
    for kv in file_formats.pairs(){
        let (k, v) : (String, Function) = kv?;
        map.insert(FileType::OtherFile(k), v);
    }

    let dirs_format : Table = tb.get("dirs")?;
    match dirs_format.get(1){
        Ok(default) => {
            *map.get_mut(&FileType::GenericDir).unwrap() = default;
        }
        Err(_) => {},
    }
    for kv in file_formats.pairs(){
        let (k, v) : (String, Function) = kv?;
        map.insert(FileType::OtherDir(k), v);
    }

    Ok(())
}

fn init_lua() -> Result<()> {
    let lua = LUA.lock().map_err(|err| anyhow!(err.to_string()) )?;
    let path = &get_options().config;

    let load_format_function = lua.create_function(get_formats)?;
    lua.globals().set("load_formats", load_format_function)?;

    if let Ok(mut file) = File::open(path){
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        lua.load(&buf).exec()?;
    }

    Ok(())
}

fn get_cells(t: Either<Table, String>) -> Result<Format>{
    match t{
        Either::Left(l) => Ok(Format::try_from(l)?),
        Either::Right(r) => Ok(Format::from(r.as_str())),
    }
}

fn print_data() -> Result<()>{
    let path = current_dir()?;
    let mut v : Vec<Format> = Vec::new();

    let mut dirs : Vec<_> = read_dir(path)?
        .filter(|e| e.is_ok())
        .map(|e|{
            let path  = e.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();

            return (name, path);
        })
        .collect();

    dirs.sort_by(|a, b|{
        let a = a.0.chars().next().unwrap().to_ascii_lowercase();
        let b = b.0.chars().next().unwrap().to_ascii_lowercase();
        return a.cmp(&b);
    });

    for (name, path) in dirs{
        if path.is_file() {
            let formatter = format_file(&path);
            v.push( get_cells( formatter.call((name.clone(), path, 0))?)? );

        }
        else if path.is_dir() {
            let formatter = format_dir(&path);
            v.push( get_cells( formatter.call((name.clone(), path, 0))?)? );
        }
    }
    for e in v{
        println!("{}", e);
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
    match &get_options().mode{
        Mode::List(ls) => ls.ls()?,//print_data()? ,
        Mode::Explorer => {},
    }

    Ok(())
}
