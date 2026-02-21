--[[
  GraphLite SDK - Session object.
  Wraps session ID and provides execute/query methods.
]]

local graphlite_ffi = require("src.graphlite_ffi")
local errors = require("src.errors")
local json_util = require("src.json_util")
local result_module = require("src.result")

local QueryError = errors.QueryError
local JsonError = errors.JsonError

local Session = {}
Session.__index = Session

function Session.new(connection, session_id, username)
  return setmetatable({
    _conn = connection,
    _session_id = session_id,
    _username = username,
    _closed = false,
  }, Session)
end

function Session:query(query_str)
  if self._closed then
    error(QueryError.new("Session is closed"))
  end
  local db = self._conn._db
  if not db then
    error(QueryError.new("Database is closed"))
  end
  local json_str = self:query_raw(query_str)
  local ok, data = pcall(json_util.decode, json_str)
  if not ok then
    error(JsonError.new("Failed to parse query result: " .. tostring(data)))
  end
  return result_module.new(data)
end

function Session:query_raw(query_str)
  if self._closed then
    error(QueryError.new("Session is closed"))
  end
  local db = self._conn._db
  if not db then
    error(QueryError.new("Database is closed"))
  end
  local json_str, _, err_name = graphlite_ffi.query(db, self._session_id, query_str)
  if not json_str then
    error(QueryError.new("Query failed: " .. (err_name or "unknown")))
  end
  return json_str
end

function Session:execute(statement)
  self:query_raw(statement)
end

function Session:close()
  if not self._closed and self._conn and self._conn._db then
    local ok, err_code = graphlite_ffi.close_session(self._conn._db, self._session_id)
    -- Best effort - don't error on close
    self._closed = true
  end
end

return {
  Session = Session,
}
