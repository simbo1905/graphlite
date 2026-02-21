#!/usr/bin/env luajit
--[[
  GraphLite OpenResty demo setup script (LuaJIT)

  Creates schema/graph and inserts sample data if missing.
  Idempotent and verifies each write via read queries.

  Usage:
    luajit setup.lua
    luajit setup.lua check
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
    "./" .. lib_name,
    "../" .. lib_name,
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

  error("Failed to load GraphLite library. Set GRAPHLITE_LIB.")
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

local function rows_empty(json)
  return json:match("\"rows\"%s*:%s*%[%s*%]") ~= nil
end

local function ensure_schema_and_graph(db, session)
  local ok, err

  ok, err = query(db, session, "CREATE SCHEMA IF NOT EXISTS /app")
  if not ok then
    return nil, err
  end

  ok, err = query(db, session, "SESSION SET SCHEMA /app")
  if not ok then
    return nil, err
  end

  ok, err = query(db, session, "CREATE GRAPH IF NOT EXISTS social")
  if not ok then
    -- Ignore if it already exists; verify by setting graph
  end

  ok, err = query(db, session, "SESSION SET GRAPH /app/social")
  if not ok then
    -- Fallback to relative if absolute fails
    ok, err = query(db, session, "SESSION SET GRAPH social")
    if not ok then
      return nil, err
    end
  end

  return true
end

local function ensure_data(db, session)
  local result, err = query(db, session, "MATCH (p:Person) RETURN p.name as name LIMIT 1")
  if not result then
    return nil, err
  end

  if rows_empty(result) then
    local inserts = {
      "CREATE (p:Person {name: 'Alice', age: 30})",
      "CREATE (p:Person {name: 'Bob', age: 25})",
      "CREATE (p:Person {name: 'Charlie', age: 35})",
    }

    for _, q in ipairs(inserts) do
      local ok, qerr = query(db, session, q)
      if not ok then
        return nil, qerr
      end
    end
  end

  local verify, verr = query(db, session, "MATCH (p:Person) RETURN p.name as name LIMIT 1")
  if not verify then
    return nil, verr
  end

  if rows_empty(verify) then
    return nil, "verification failed: no data"
  end

  return true
end

local function main()
  local mode = arg[1] or "setup"

  local db_path = os.getenv("GRAPHLITE_DB_PATH")
  if not db_path or db_path == "" then
    print("GRAPHLITE_DB_PATH is not set")
    return 1
  end

  -- Safely escape path by replacing single quotes with '\'', and wrapping in single quotes
  local escaped_db_path = db_path:gsub("'", "'\\''")
  os.execute("mkdir -p '" .. escaped_db_path .. "'")

  local db, err = graphlite_open(db_path)
  if not db then
    print("Failed to open database: " .. err)
    return 1
  end

  local session, err = create_session(db, "admin")
  if not session then
    gl.graphlite_close(db)
    print("Failed to create session: " .. err)
    return 1
  end

  local ok, qerr = ensure_schema_and_graph(db, session)
  if not ok then
    gl.graphlite_close_session(db, session, nil)
    gl.graphlite_close(db)
    print("Setup failed: " .. qerr)
    return 1
  end

  if mode ~= "check" then
    ok, qerr = ensure_data(db, session)
    if not ok then
      gl.graphlite_close_session(db, session, nil)
      gl.graphlite_close(db)
      print("Data setup failed: " .. qerr)
      return 1
    end
  else
    local verify, verr = query(db, session, "MATCH (p:Person) RETURN p.name as name LIMIT 1")
    if not verify then
      gl.graphlite_close_session(db, session, nil)
      gl.graphlite_close(db)
      print("Check failed: " .. verr)
      return 1
    end
    if rows_empty(verify) then
      gl.graphlite_close_session(db, session, nil)
      gl.graphlite_close(db)
      print("Check failed: no data")
      return 1
    end
  end

  gl.graphlite_close_session(db, session, nil)
  gl.graphlite_close(db)

  print("Setup complete")
  return 0
end

os.exit(main())
