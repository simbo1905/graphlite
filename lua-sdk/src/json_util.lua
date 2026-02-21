--[[
  JSON parsing for GraphLite query results.
  Uses dkjson (install via: luarocks install dkjson)
  Engine returns JSON as bytes; we parse with dkjson.
]]

local dkjson = require("dkjson")

return {
  decode = function(str)
    return dkjson.decode(str)
  end,
}
