--[[
  GraphLite FFI - Thin adapter over LuaJIT ffi for the GraphLite C FFI library.
  Binds against libgraphlite_ffi (same symbols as Python/Java bindings).
  All ffi.cdef and ffi.load logic lives here.
]]

local ffi = require("ffi")

-- Platform-aware library loading
local function find_library()
  local lib_name
  if ffi.os == "OSX" then
    lib_name = "libgraphlite_ffi.dylib"
  elseif ffi.os == "Windows" then
    lib_name = "graphlite_ffi.dll"
  else
    lib_name = "libgraphlite_ffi.so"
  end

  -- Infer GraphLite repo root from this file's location (lua-sdk/src/graphlite_ffi.lua)
  -- Repo root is parent of lua-sdk, so target/ is at repo_root/target/
  local repo_root
  local src = debug.getinfo(1, "S").source
  if src:sub(1, 1) == "@" then
    src = src:sub(2)
  end
  local src_dir = src:match("(.+)/[^/]+$") or "."
  if src_dir:match("/src$") then
    local lua_sdk_root = src_dir:match("(.+)/src$")
    if lua_sdk_root and lua_sdk_root:match("/lua%-sdk$") then
      repo_root = lua_sdk_root:match("(.+)/lua%-sdk$")
    elseif lua_sdk_root then
      repo_root = lua_sdk_root  -- lua-sdk at repo root
    end
  end
  if repo_root and repo_root ~= "" then
    repo_root = repo_root:gsub("/+$", "")
  end

  local search_paths = {}
  if repo_root then
    search_paths[#search_paths + 1] = repo_root .. "/target/release/" .. lib_name
    search_paths[#search_paths + 1] = repo_root .. "/target/debug/" .. lib_name
  end
  for _, p in ipairs({
    "graphlite_ffi", "libgraphlite_ffi", lib_name,
    "target/release/" .. lib_name, "target/debug/" .. lib_name,
    "../target/release/" .. lib_name, "../target/debug/" .. lib_name,
    "../../target/release/" .. lib_name, "../../target/debug/" .. lib_name,
    "../../../target/release/" .. lib_name, "../../../target/debug/" .. lib_name,
    "/usr/local/lib/" .. lib_name, "/usr/lib/" .. lib_name,
  }) do
    search_paths[#search_paths + 1] = p
  end

  for _, path in ipairs(search_paths) do
    local ok, lib = pcall(ffi.load, path)
    if ok and lib then
      return lib
    end
  end

  return nil, "Could not find GraphLite library (" .. lib_name .. "). " ..
    "Build first: cargo build --release -p graphlite-ffi"
end

-- C FFI definitions (match graphlite-ffi/src/lib.rs)
ffi.cdef([[
  typedef enum {
    GraphLiteErrorCode_Success = 0,
    GraphLiteErrorCode_NullPointer = 1,
    GraphLiteErrorCode_InvalidUtf8 = 2,
    GraphLiteErrorCode_DatabaseOpenError = 3,
    GraphLiteErrorCode_SessionError = 4,
    GraphLiteErrorCode_QueryError = 5,
    GraphLiteErrorCode_PanicError = 6,
    GraphLiteErrorCode_JsonError = 7
  } GraphLiteErrorCode;

  typedef struct GraphLiteDB GraphLiteDB;

  GraphLiteDB* graphlite_open(const char* path, GraphLiteErrorCode* error_out);
  char* graphlite_create_session(GraphLiteDB* db, const char* username, GraphLiteErrorCode* error_out);
  char* graphlite_query(GraphLiteDB* db, const char* session_id, const char* query, GraphLiteErrorCode* error_out);
  GraphLiteErrorCode graphlite_close_session(GraphLiteDB* db, const char* session_id, GraphLiteErrorCode* error_out);
  void graphlite_free_string(char* s);
  void graphlite_close(GraphLiteDB* db);
  const char* graphlite_version(void);
]])

local lib, err = find_library()
if not lib then
  error(err)
end

-- Error code names for Lua
local ERROR_NAMES = {
  [0] = "Success",
  [1] = "NullPointer",
  [2] = "InvalidUtf8",
  [3] = "DatabaseOpenError",
  [4] = "SessionError",
  [5] = "QueryError",
  [6] = "PanicError",
  [7] = "JsonError",
}

local M = {}

function M.open(path)
  local error_out = ffi.new("GraphLiteErrorCode[1]")
  error_out[0] = 0
  local db = lib.graphlite_open(path, error_out)
  if db == nil then
    return nil, error_out[0], ERROR_NAMES[error_out[0]] or "Unknown"
  end
  return db
end

function M.create_session(db, username)
  local error_out = ffi.new("GraphLiteErrorCode[1]")
  error_out[0] = 0
  local session_id_ptr = lib.graphlite_create_session(db, username, error_out)
  if session_id_ptr == nil then
    return nil, error_out[0], ERROR_NAMES[error_out[0]] or "Unknown"
  end
  local session_id = ffi.string(session_id_ptr)
  lib.graphlite_free_string(session_id_ptr)
  return session_id
end

function M.query(db, session_id, query_str)
  local error_out = ffi.new("GraphLiteErrorCode[1]")
  error_out[0] = 0
  local result_ptr = lib.graphlite_query(db, session_id, query_str, error_out)
  if result_ptr == nil then
    return nil, error_out[0], ERROR_NAMES[error_out[0]] or "Unknown"
  end
  local json_str = ffi.string(result_ptr)
  lib.graphlite_free_string(result_ptr)
  return json_str
end

function M.close_session(db, session_id)
  local error_out = ffi.new("GraphLiteErrorCode[1]")
  error_out[0] = 0
  local code = lib.graphlite_close_session(db, session_id, error_out)
  return code == 0, error_out[0]
end

function M.close(db)
  if db and db ~= nil then
    lib.graphlite_close(db)
  end
end

function M.version()
  local v = lib.graphlite_version()
  if v and v ~= nil then
    return ffi.string(v)
  end
  return "unknown"
end

M.ERROR_NAMES = ERROR_NAMES
M.lib = lib

return M
