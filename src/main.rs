mod list;
mod util;
mod fmt;
mod explorer;
mod api;

use api::{get_formats, bash};
use list::List;
use util::*;

use std::{
    collections::HashMap, env::home_dir, fs::File, io::Read, path::PathBuf, process::exit, sync::{LazyLock, Mutex, OnceLock}
};


use clap::Parser;
use anyhow::{anyhow, Result};
use mlua::{Function, Lua};

pub static MAP : LazyLock<Mutex< HashMap<FileType, mlua::Function>>> = LazyLock::new(||{ 
    use FileType as FT;
    let lua = LUA.lock().unwrap();

    let def_file: Function = lua.load("function(name, path, tick) return '󰈔 ' .. name end").eval().unwrap();
    let def_dir: Function = lua.load("function(name, path, tick) return ' ' .. name .. '/' end").eval().unwrap();
    let def_link: Function = lua.load("function(name, path, tick) return ' ' .. name .. '@' end").eval().unwrap();

    return Mutex::new(HashMap::from([
        (FT::GenericFile, def_file),
        (FT::GenericDir , def_dir),
        (FT::GenericSymLink, def_link),
    ]));
});

pub static LUA: LazyLock<Mutex<Lua>> = LazyLock::new( || Mutex::new(Lua::new()) );

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

#[allow(dead_code)]
fn curr_dir() -> PathBuf {
    use std::env::current_dir;
    return match current_dir(){
        Ok(s) => s,
        Err(_) => {
            exit(-1);
        },
    };
}

#[derive(Parser)]
enum Mode{
    List(List),
    Explorer,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Options {
    #[arg(long, short, default_value = config_dir().display().to_string())]
    config: PathBuf,

    #[arg(default_value = curr_dir().display().to_string())]
    path: PathBuf,

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

pub fn get_options<'a>() -> &'a Options{
    OPTIONS.get_or_init( Options::parse )
}

fn init_lua() -> Result<()> {
    let lua = LUA.lock().map_err(|err| anyhow!(err.to_string()) )?;
    let path = &get_options().config;

    let load_format_function = lua.create_function(get_formats)?;
    lua.globals().set("load_formats", load_format_function)?;
    let bash_function = lua.create_function(bash)?;
    lua.globals().set("bash", bash_function)?;

    if let Ok(mut file) = File::open(path){
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        lua.load(&buf).exec()?;
    }

    Ok(())
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
