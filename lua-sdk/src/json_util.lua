--[[
  JSON parsing for GraphLite query results.
  Tries cjson (common in LuaJIT/OpenResty) first, then cjson.safe.
  Fallback: minimal recursive descent decoder for GraphLite result format.
]]

local function try_require(name)
  local ok, mod = pcall(require, name)
  if ok and mod and type(mod.decode) == "function" then
    return mod.decode
  end
  return nil
end

local json_decode = try_require("cjson") or try_require("cjson.safe")

if not json_decode then
  -- Minimal recursive descent JSON decoder (handles GraphLite result format)
  json_decode = function(str)
    local pos = 1
    local function skip_ws()
      while pos <= #str and str:sub(pos, pos):match("%s") do pos = pos + 1 end
    end
    local function parse_value()
      skip_ws()
      local c = str:sub(pos, pos)
      if c == "{" then
        pos = pos + 1
        local obj = {}
        while true do
          skip_ws()
          if str:sub(pos, pos) == "}" then pos = pos + 1 return obj end
          local key = parse_value()
          skip_ws()
          if str:sub(pos, pos) ~= ":" then error("expected : at " .. pos) end
          pos = pos + 1
          obj[key] = parse_value()
          skip_ws()
          c = str:sub(pos, pos)
          if c == "}" then pos = pos + 1 return obj end
          if c == "," then pos = pos + 1 end
        end
      elseif c == "[" then
        pos = pos + 1
        local arr = {}
        while true do
          skip_ws()
          if str:sub(pos, pos) == "]" then pos = pos + 1 return arr end
          table.insert(arr, parse_value())
          skip_ws()
          c = str:sub(pos, pos)
          if c == "]" then pos = pos + 1 return arr end
          if c == "," then pos = pos + 1 end
        end
      elseif c == '"' then
        pos = pos + 1
        local s = ""
        while pos <= #str do
          local ch = str:sub(pos, pos)
          if ch == '\\' then
            pos = pos + 1
            ch = str:sub(pos, pos)
            if ch == 'n' then s = s .. '\n'
            elseif ch == 't' then s = s .. '\t'
            elseif ch == 'r' then s = s .. '\r'
            elseif ch == '"' then s = s .. '"'
            else s = s .. ch end
          elseif ch == '"' then pos = pos + 1 return s end
          s = s .. ch
          pos = pos + 1
        end
        return s
      elseif str:sub(pos, pos + 3) == "true" then pos = pos + 4 return true
      elseif str:sub(pos, pos + 4) == "false" then pos = pos + 5 return false
      elseif str:sub(pos, pos + 3) == "null" then pos = pos + 4 return nil
      elseif c == "-" or c:match("%d") then
        local s = ""
        while pos <= #str and str:sub(pos, pos):match("[%d%+%-%.eE]") do
          s = s .. str:sub(pos, pos)
          pos = pos + 1
        end
        return tonumber(s)
      else
        error("unexpected char at " .. pos .. ": " .. c)
      end
    end
    return parse_value()
  end
end

return { decode = json_decode }
