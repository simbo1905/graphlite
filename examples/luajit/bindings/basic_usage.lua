#!/usr/bin/env luajit
--[[
  GraphLite LuaJIT FFI Example - Basic Usage

  This example demonstrates how to use GraphLite from LuaJIT using the FFI API.
  It mirrors the Python bindings example but runs directly via LuaJIT.

  Prerequisites:
    - LuaJIT installed
    - GraphLite FFI library built: cargo build --release -p graphlite-ffi

  Usage:
    luajit basic_usage.lua
--]]

local ffi = require("ffi")

ffi.cdef[[
  typedef enum {
    Success = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    DatabaseOpenError = 3,
    SessionError = 4,
    QueryError = 5,
    PanicError = 6,
    JsonError = 7
  } GraphLiteErrorCode;

  typedef struct GraphLiteDB GraphLiteDB;

  GraphLiteDB *graphlite_open(const char *path, GraphLiteErrorCode *error_out);
  char *graphlite_create_session(GraphLiteDB *db, const char *username, GraphLiteErrorCode *error_out);
  char *graphlite_query(GraphLiteDB *db, const char *session_id, const char *query, GraphLiteErrorCode *error_out);
  GraphLiteErrorCode graphlite_close_session(GraphLiteDB *db, const char *session_id, GraphLiteErrorCode *error_out);
  void graphlite_free_string(char *s);
  void graphlite_close(GraphLiteDB *db);
  const char *graphlite_version(void);
]]

local function load_graphlite()
  local lib_name
  if ffi.os == "OSX" then
    lib_name = "libgraphlite_ffi.dylib"
  else
    lib_name = "libgraphlite_ffi.so"
  end

  local env_path = os.getenv("GRAPHLITE_LIB")
  if env_path then
    return ffi.load(env_path)
  end

  local paths = {
    "./target/release/" .. lib_name,
    "../target/release/" .. lib_name,
    "../../target/release/" .. lib_name,
    "../../../target/release/" .. lib_name,
    lib_name,
  }

  for _, path in ipairs(paths) do
    local ok, lib = pcall(ffi.load, path)
    if ok then
      return lib
    end
  end

  error("Failed to load GraphLite library. Set GRAPHLITE_LIB to the full path.")
end

local gl = load_graphlite()

local function error_to_string(code)
  local errors = {
    [0] = "Success",
    [1] = "NullPointer",
    [2] = "InvalidUtf8",
    [3] = "DatabaseOpenError",
    [4] = "SessionError",
    [5] = "QueryError",
    [6] = "PanicError",
    [7] = "JsonError",
  }
  return errors[code] or string.format("UnknownError(%d)", code)
end

local function graphlite_open(path)
  local error_code = ffi.new("GraphLiteErrorCode[1]")
  local db = gl.graphlite_open(path, error_code)
  if db == nil then
    return nil, error_to_string(tonumber(error_code[0]))
  end
  return db
end

local function create_session(db, username)
  local error_code = ffi.new("GraphLiteErrorCode[1]")
  local session_ptr = gl.graphlite_create_session(db, username, error_code)
  if session_ptr == nil then
    return nil, error_to_string(tonumber(error_code[0]))
  end
  local session_id = ffi.string(session_ptr)
  gl.graphlite_free_string(session_ptr)
  return session_id
end

local function query(db, session_id, gql)
  local error_code = ffi.new("GraphLiteErrorCode[1]")
  local result_ptr = gl.graphlite_query(db, session_id, gql, error_code)
  if result_ptr == nil then
    return nil, error_to_string(tonumber(error_code[0]))
  end
  local json_str = ffi.string(result_ptr)
  gl.graphlite_free_string(result_ptr)
  return json_str
end

local function close_session(db, session_id)
  local error_code = ffi.new("GraphLiteErrorCode[1]")
  local code = gl.graphlite_close_session(db, session_id, error_code)
  if tonumber(code) ~= 0 then
    return false, error_to_string(tonumber(code))
  end
  return true
end

local function main()
  print("=== GraphLite LuaJIT FFI Example ===")
  print("GraphLite version: " .. ffi.string(gl.graphlite_version()))
  print("")

  local db_path = os.tmpname() .. "_graphlite_db"
  local db, err = graphlite_open(db_path)
  if not db then
    print("Failed to open database: " .. err)
    return 1
  end

  local session, err = create_session(db, "admin")
  if not session then
    print("Failed to create session: " .. err)
    gl.graphlite_close(db)
    return 1
  end

  local ok, qerr
  ok, qerr = query(db, session, "CREATE SCHEMA IF NOT EXISTS /example")
  ok, qerr = query(db, session, "SESSION SET SCHEMA /example")
  ok, qerr = query(db, session, "CREATE GRAPH IF NOT EXISTS social")
  ok, qerr = query(db, session, "SESSION SET GRAPH social")

  query(db, session, "CREATE (p:Person {name: 'Alice', age: 30})")
  query(db, session, "CREATE (p:Person {name: 'Bob', age: 25})")
  query(db, session, "CREATE (p:Person {name: 'Charlie', age: 35})")

  local result, err = query(db, session, "MATCH (p:Person) RETURN p.name as name, p.age as age")
  if not result then
    print("Query failed: " .. err)
  else
    print("Result JSON:")
    print(result)
  end

  close_session(db, session)
  gl.graphlite_close(db)

  os.execute(string.format("rm -rf %s", db_path))
  print("=== Done ===")
  return 0
end

os.exit(main())
