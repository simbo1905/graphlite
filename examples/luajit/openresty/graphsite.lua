--[[
  GraphLite OpenResty read-only endpoint

  Query via: /graphsite.lua?query=...
  If query is omitted, returns a default list of Person nodes.
--]]

local ffi = require("ffi")
local json = require("cjson")

local function ensure_cdef()
  local ok = pcall(ffi.cdef, [[
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
  ]])
  return ok
end

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

ensure_cdef()
local graphlite_lib = load_graphlite()

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
  local db = graphlite_lib.graphlite_open(path, error_code)
  if db == nil then
    return nil, "failed to open database: " .. error_to_string(tonumber(error_code[0]))
  end
  return db
end

local function create_session(db, username)
  local error_code = ffi.new("GraphLiteErrorCode[1]")
  local session_ptr = graphlite_lib.graphlite_create_session(db, username, error_code)
  if session_ptr == nil then
    return nil, "failed to create session: " .. error_to_string(tonumber(error_code[0]))
  end
  local session_id = ffi.string(session_ptr)
  graphlite_lib.graphlite_free_string(session_ptr)
  return session_id
end

local function gql_query(db, session_id, query_str)
  local error_code = ffi.new("GraphLiteErrorCode[1]")
  local result_ptr = graphlite_lib.graphlite_query(db, session_id, query_str, error_code)
  if result_ptr == nil then
    return nil, "query failed: " .. error_to_string(tonumber(error_code[0]))
  end
  local json_str = ffi.string(result_ptr)
  graphlite_lib.graphlite_free_string(result_ptr)
  return json_str
end

local function close_session(db, session_id)
  local error_code = ffi.new("GraphLiteErrorCode[1]")
  local code = graphlite_lib.graphlite_close_session(db, session_id, error_code)
  if tonumber(code) ~= 0 then
    return false, "failed to close session: " .. error_to_string(tonumber(code))
  end
  return true
end

local function respond_error(status, msg)
  ngx.status = status
  ngx.header.content_type = "application/json"
  ngx.say(json.encode({ error = msg }))
end

local function open_db()
  local db_path = os.getenv("GRAPHLITE_DB_PATH")
  if not db_path or db_path == "" then
    return nil, "GRAPHLITE_DB_PATH is not set"
  end

  local db, err = graphlite_open(db_path)
  if not db then
    return nil, err
  end

  return db
end

local function prepare_session(db, session)
  local ok, err
  ok, err = gql_query(db, session, "SESSION SET SCHEMA /app")
  if not ok then
    return nil, "setup missing: run setup.sh"
  end
  ok, err = gql_query(db, session, "SESSION SET GRAPH /app/social")
  if not ok then
    ok, err = gql_query(db, session, "SESSION SET GRAPH social")
    if not ok then
      return nil, "setup missing: run setup.sh"
    end
  end
  return true
end

local function main()
  local args = ngx.req.get_uri_args()
  local gql = args.query
  if not gql or gql == "" then
    gql = "MATCH (p:Person) RETURN p.name as name, p.age as age"
  end

  -- SECURITY: Enforce read-only access (only allow MATCH queries)
  -- Remove leading whitespace and check for 'MATCH' followed by space or parenthesis
  if not gql:match("^%s*[mM][aA][tT][cC][hH][%s%(]") then
    return respond_error(ngx.HTTP_FORBIDDEN, "Security error: Only MATCH queries are allowed in this read-only endpoint.")
  end

  local db, err = open_db()
  if not db then
    return respond_error(ngx.HTTP_INTERNAL_SERVER_ERROR, err)
  end

  local session, err = create_session(db, "api_user")
  if not session then
    graphlite_lib.graphlite_close(db)
    return respond_error(ngx.HTTP_INTERNAL_SERVER_ERROR, err)
  end

  local ok, perr = prepare_session(db, session)
  if not ok then
    close_session(db, session)
    graphlite_lib.graphlite_close(db)
    return respond_error(ngx.HTTP_INTERNAL_SERVER_ERROR, perr)
  end

  local result, qerr = gql_query(db, session, gql)

  close_session(db, session)
  graphlite_lib.graphlite_close(db)

  if not result then
    return respond_error(ngx.HTTP_INTERNAL_SERVER_ERROR, qerr)
  end

  ngx.header.content_type = "application/json"
  ngx.say(result)
end

main()
