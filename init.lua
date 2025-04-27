
local function set_color(r, g, b)
    if type(r) == "number" then
        return '\x1b[38;2;' .. r .. ';' .. g .. ';' .. b .. 'm'
    else
        local rc = tonumber(string.sub(r, 1, 2), 16)
        local gc = tonumber(string.sub(r, 3, 4), 16) or 0
        local bc = tonumber(string.sub(r, 5, 6), 16) or 0

        return set_color(rc, gc, bc)
    end
end

local function reset_col()
    return '\x1b[0m'
end

local types = {
    lua  = set_color('00b3d7') .. '',
    rust = '󱘗',
    cpp  = '',
    toml = '',
}

local formats = {
    file = {},
    dirs = {},
}

for type, sym in pairs(types) do
    ---@param name string
    ---@param _full_path string
    ---@param _tick number
    ---@return unknown
    formats.file[type] = function (name, _full_path, _tick)
        return sym .. ' ' .. name .. reset_col()
    end
end

load_formats(formats)
