--[[
  GraphLite SDK - Typed error types.
  Maps FFI error codes to Lua tables with metatable for structured errors.
]]

local ERROR_CODES = {
  Success = 0,
  NullPointer = 1,
  InvalidUtf8 = 2,
  DatabaseOpenError = 3,
  SessionError = 4,
  QueryError = 5,
  PanicError = 6,
  JsonError = 7,
}

local GraphLiteError = {}
GraphLiteError.__index = GraphLiteError

function GraphLiteError.new(message, code, code_name)
  return setmetatable({
    message = message or "GraphLite error",
    code = code or 0,
    code_name = code_name or "Unknown",
  }, GraphLiteError)
end

function GraphLiteError:__tostring()
  return string.format("GraphLiteError (%s): %s", self.code_name, self.message)
end

local ConnectionError = setmetatable({}, { __index = GraphLiteError })
ConnectionError.__index = ConnectionError

function ConnectionError.new(message)
  return setmetatable({
    message = message or "Connection error",
    code = ERROR_CODES.DatabaseOpenError,
    code_name = "DatabaseOpenError",
  }, ConnectionError)
end

local SessionError = setmetatable({}, { __index = GraphLiteError })
SessionError.__index = SessionError

function SessionError.new(message)
  return setmetatable({
    message = message or "Session error",
    code = ERROR_CODES.SessionError,
    code_name = "SessionError",
  }, SessionError)
end

local QueryError = setmetatable({}, { __index = GraphLiteError })
QueryError.__index = QueryError

function QueryError.new(message)
  return setmetatable({
    message = message or "Query error",
    code = ERROR_CODES.QueryError,
    code_name = "QueryError",
  }, QueryError)
end

local JsonError = setmetatable({}, { __index = GraphLiteError })
JsonError.__index = JsonError

function JsonError.new(message)
  return setmetatable({
    message = message or "JSON error",
    code = ERROR_CODES.JsonError,
    code_name = "JsonError",
  }, JsonError)
end

-- Create error from FFI error code
local function from_code(code, code_name, default_message)
  local msg = default_message or (code_name or "Error")
  return GraphLiteError.new(msg, code, code_name)
end

return {
  GraphLiteError = GraphLiteError,
  ConnectionError = ConnectionError,
  SessionError = SessionError,
  QueryError = QueryError,
  JsonError = JsonError,
  ERROR_CODES = ERROR_CODES,
  from_code = from_code,
}
