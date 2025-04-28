local utf = require 'utf8'

local types = {
    lua  = {sy='', cl={0x99, 0x99, 0xff}},
    rs   = {sy='󱘗', cl={0x9B, 0x52, 0x31}},
    c    = {sy='', cl={0x66, 0x99, 0xD2}},
    h    = {sy='', cl={0x66, 0x99, 0xD2}},
    cpp  = {sy='', cl={}},
    toml = {sy='', cl={0xaB, 0x52, 0x31}},
    txt  = {sy='󰦨'},
    md   = {sy=''},
}

---@class Color
---@field [1] number
---@field [2] number
---@field [3] number

---@class Cell
---@field chr string
---@field col Color

---@param s string
---@return Cell[]
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
            if string.sub(name, 1,1) == '.' then
                fmt = into_cells('󰈔 ' .. name, {0x99, 0x99, 0x99})
            else
                fmt = into_cells('󰈔 ' .. name, {0xff, 0xff, 0xff})
            end

            return fmt
        end,
    },
    dirs = {
        function (name, _, _)
            local fmt = {}
            local str = ' ' .. name .. '/'
            if string.sub(name, 1,1) == '.' then
                fmt = into_cells(str, {0x55, 0x55, 0x99})
            else
                fmt = into_cells(str, {0x77, 0x77, 0xff})
            end

            return fmt
        end,
    },
}

for type, sym in pairs(types) do
    local cl = sym.cl
    local sy = sym.sy
    formats.file[type] = function (name, _, _)
        local cells = into_cells(sy .. ' ' .. name)
        cells[1].col = cl
        return cells
    end
end



load_formats(formats)
