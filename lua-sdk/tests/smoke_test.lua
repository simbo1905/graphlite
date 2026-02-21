--[[
  Minimal smoke test for GraphLite LuaJIT SDK.
  Run from repo root: luajit lua-sdk/tests/smoke_test.lua
  Or from lua-sdk: luajit tests/smoke_test.lua
]]

local function get_sdk_path()
  local src = debug.getinfo(1, "S").source
  if src:sub(1, 1) == "@" then src = src:sub(2) end
  local dir = src:match("(.+)/[^/]+$") or "."
  if dir:match("/tests$") then
    return dir:match("(.+)/tests$")
  end
  return "."
end

local sdk_path = get_sdk_path()
package.path = sdk_path .. "/?.lua;" .. package.path

local connection = require("src.connection")
local GraphLite = connection.GraphLite

local tmp_dir = os.getenv("TMPDIR") or os.getenv("TMP") or "/tmp"
local db_path = tmp_dir .. "/graphlite_lua_sdk_smoke_" .. os.time()

local function cleanup()
  pcall(function()
    if db_path and db_path ~= "" then
      os.execute("rm -rf " .. db_path)
    end
  end)
end

local ok, err = pcall(function()
  print("=== GraphLite LuaJIT SDK Smoke Test ===")

  print("1. Opening database...")
  local db = GraphLite.open(db_path)
  assert(db, "db should exist")

  print("2. Creating session...")
  local session = db:session("admin")
  assert(session, "session should exist")

  print("3. Creating schema and graph...")
  session:execute("CREATE SCHEMA IF NOT EXISTS /smoke")
  session:execute("SESSION SET SCHEMA /smoke")
  session:execute("CREATE GRAPH IF NOT EXISTS test")
  session:execute("SESSION SET GRAPH test")

  print("4. Inserting nodes...")
  session:execute("INSERT (n:Node {id: 1, name: 'A'})")
  session:execute("INSERT (n:Node {id: 2, name: 'B'})")

  print("5. Running query...")
  local result = session:query("MATCH (n:Node) RETURN n.id, n.name ORDER BY n.id")
  assert(result, "result should exist")
  assert(#result.rows >= 2, "should have at least 2 rows")
  print("   Rows:", #result.rows)
  for i, row in ipairs(result.rows) do
    print("   ", i, row["n.id"], row["n.name"])
  end

  print("6. Closing session and db...")
  session:close()
  db:close()

  print("=== Smoke test completed successfully ===")
end)

cleanup()

if not ok then
  print("FAILED:", err)
  if type(err) == "table" and err.message then
    print("Error:", err.message)
  end
  os.exit(1)
end

os.exit(0)
