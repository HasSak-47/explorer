local utf = require 'utf8'

local types = {
    lua  = {sy='', cl={0x99, 0x99, 0xff}},
    sh   = {sy='󱆃', cl={0x99, 0xff, 0x99}},
    rs   = {sy='󱘗', cl={0x9B, 0x52, 0x31}},
    c    = {sy='', cl={0x66, 0x99, 0xD2}},
    h    = {sy='', cl={0x66, 0x99, 0xD2}},
    cpp  = {sy='', cl={}},
    toml = {sy='', cl={0xaB, 0x52, 0x31}},
    txt  = {sy='󰦨'},
    md   = {sy=''},
    zip  = {sy = ''},
    pdf  = {sy = ''},
    svg  = {sy = '󰜡'},
    csv  = {sy = ''},
    png  = {sy = ''}, jpeg = {sy = ''}, jpg  = {sy = ''},
    lock = {sy = ''},
    nix  = {sy = '', cl={0x5B, 0xa2, 0xf1}},
    so   = {sy = '', cl={0xf1, 0x91, 0x11}},
    o    = {sy = '', cl={0xf1, 0x91, 0x11}},
    py   = {sy = '', cl={0xff, 0xbf, 0x11}},
}
local function parse_env(var)
    local result = {}
    for part in string.gmatch(var, "([^:]+)") do
        table.insert(result, part)
    end
    return result
end

local data_dirs = parse_env(bash 'echo $XDG_DATA_DIRS')
local conf_dirs = parse_env(bash 'echo $XDG_CONFIG_DIRS')

local home = bash 'echo $HOME'
home = string.sub(home, 1, string.len(home) - 1)
local special_path = {
    [home]                 = {sy = '󱂵'},
    [home .. '/Documents'] = {sy = '󱧶'},
    [home .. '/Pictures' ] = {sy = '󰉏'},
    [home .. '/Downloads'] = {sy = '󰉍'},
    [home .. '/Videos'   ] = {sy = '󱧺'},
    [home .. '/Music'    ] = {sy = '󱍙'},
    [home .. '/Desktop'  ] = {sy = ''},
}

for _, dir in ipairs(conf_dirs) do
    special_path[dir] = {sy = ''}
end

local function into_cells(s, col)
    if not col then
        col = {0xff, 0xff, 0xff, }
    end
    local t = {}
    for i = 1, utf.len(s), 1 do
        local s_beg = utf8.offset(s, i);
        local s_end = utf8.offset(s, i + 1);
        local c = ''

        if s_end then
            c = string.sub(s, s_beg, s_end - 1)
        else
            c = string.sub(s, s_beg)
        end

        table.insert(t, {
            chr = c,
            col = col
        })
    end

    return t
end

local formats = {
    file = {
        function (name, _, _)
            local fmt = {}
            local str = '󰈔 ' .. name
            if name == '.gitignore' then
                fmt = into_cells(' ' .. name, {0x99, 0x99, 0x99})
                fmt[1].col = {0xaB, 0x52, 0x31}
            elseif string.lower(name) == 'make' or string.lower(name) == 'makefile' then
                fmt = into_cells(' ' .. name)
                fmt[1].col = {0xaa, 0x33, 0x11}
            elseif string.sub(name, 1,1) == '.' then
                fmt = into_cells(str, {0x99, 0x99, 0x99})
            else
                fmt = into_cells(str)
            end

            return fmt
        end,
    },
    dirs = {
        function (name, path, _)
            local has_git = bash('ls ' .. path .. ' | grep git')
            if has_git then
                print('git!')
            end
            local sp = special_path[path]
            if sp ~= nil then
                local fmt = into_cells(sp.sy .. ' ' .. name .. '/')
                if sp.col then
                    fmt[1].col = sp.col
                else
                fmt[1].col = {0x77, 0x77, 0xff}
                end
                return fmt
            end

            if name == '.git' then
                local fmt = into_cells(' ' .. name .. '/', {0x99, 0x99, 0x99})
                fmt[1].col = {0xaB, 0x52, 0x31}
                return fmt
            end

            local str = ' ' .. name .. '/'
            local fmt = {}
            if string.sub(name, 1,1) == '.' then
                fmt = into_cells(str, {0x99, 0x99, 0x99})
                fmt[1].col = {0x55, 0x55, 0x99}
            else
                fmt = into_cells(str)
                fmt[1].col = {0x77, 0x77, 0xff}
            end

            return fmt
        end,
    },
}

local function register_generic(sy, cl)
    return function (name, _, _)
        local cells = into_cells(sy .. ' ' .. name)
        cells[1].col = cl
        return cells
    end
end

for type, sym in pairs(types) do
    local cl = sym.cl
    local sy = sym.sy
    formats.file[type] = register_generic(sy, cl)
end

load_formats(formats)
