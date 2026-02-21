--[[
  GraphLite SDK - Connection and session management.
  Session-centric API mirroring the Python SDK.
]]

local graphlite_ffi = require("src.graphlite_ffi")
local errors = require("src.errors")
local session_module = require("src.session")

local ConnectionError = errors.ConnectionError
local SessionError = errors.SessionError

local GraphLite = {}
GraphLite.__index = GraphLite

function GraphLite.open(path)
  local db = graphlite_ffi.open(path)
  if not db then
    error(ConnectionError.new("Failed to open database at " .. tostring(path)))
  end
  return setmetatable({
    _db = db,
    _path = path,
    _closed = false,
  }, GraphLite)
end

function GraphLite:session(username)
  if self._closed or not self._db then
    error(ConnectionError.new("Database is closed"))
  end
  local session_id, err_code, err_name = graphlite_ffi.create_session(self._db, username)
  if not session_id then
    error(SessionError.new("Failed to create session for user '" .. tostring(username) .. "': " .. (err_name or "unknown")))
  end
  return session_module.Session.new(self, session_id, username)
end

function GraphLite:close()
  if not self._closed and self._db then
    graphlite_ffi.close(self._db)
    self._db = nil
    self._closed = true
  end
end

function GraphLite:__gc()
  self:close()
end

-- Best-effort finalizer (Lua 5.1/LuaJIT)
local mt = getmetatable(GraphLite) or {}
mt.__gc = GraphLite.__gc
setmetatable(GraphLite, mt)

return {
  GraphLite = GraphLite,
}
