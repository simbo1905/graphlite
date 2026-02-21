--[[
  Bootstrap: locate LuaJIT SDK and add to package.path.
  Use: require("bootstrap") before requiring src.connection etc.

  Search order:
  1. GRAPH_LITE_LUA_SDK env var
  2. ~/github/simbo1905/graphlite/lua-sdk/
  3. ~/github/deepgraphai/GraphLite/lua-sdk/
]]

local function get_sdk_path()
  local env = os.getenv("GRAPH_LITE_LUA_SDK")
  if env and env ~= "" then
    return env
  end

  local home = os.getenv("HOME") or os.getenv("USERPROFILE") or ""
  if home == "" then
    return nil
  end

  local candidates = {
    home .. "/github/simbo1905/graphlite/lua-sdk",
    home .. "/github/deepgraphai/GraphLite/lua-sdk",
  }

  for _, p in ipairs(candidates) do
    -- Check for src/connection.lua
    local f = io.open(p .. "/src/connection.lua", "r")
    if f then
      f:close()
      return p
    end
  end

  return nil
end

local sdk_path = get_sdk_path()
if not sdk_path then
  io.stderr:write([[
ERROR: GraphLite LuaJIT SDK not found.

Please either:
  1. Set GRAPH_LITE_LUA_SDK to the path of your lua-sdk directory, or
  2. Clone/checkout the luajit-sdk branch and place lua-sdk at one of:
     ~/github/simbo1905/graphlite/lua-sdk/
     ~/github/deepgraphai/GraphLite/lua-sdk/

Example:
  git fetch origin luajit-sdk
  git checkout luajit-sdk -- lua-sdk

Then run: luajit drug_discovery.lua
]])
  os.exit(1)
end

package.path = sdk_path .. "/?.lua;" .. package.path
return sdk_path
