use std::{io::Read, process::{Command, Stdio}};

use crate::{util::FileType, LUA, MAP};
use mlua::{Function, Lua, Table};

pub fn get_formats(_: &Lua, tb: mlua::Table) -> mlua::Result<()>{
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

pub fn bash(_: &Lua, s: String) -> mlua::Result<String>{
    let mut cmd = Command::new("bash");
    cmd.arg("-c");
    cmd.arg(s);
    cmd.stdout(Stdio::piped());
    let result = cmd.spawn()?;
    return Ok(match result.stdout{
        Some(mut s) => {
            let mut buf = String::new();
            s.read_to_string(&mut buf)?;
            buf
        },
        _ => {
            String::new()
        }
    });
}
