#!/usr/bin/env lua
--- Basic Usage Example for GraphLite High-Level Lua SDK
--
-- Minimal example: open database, create session, insert nodes,
-- query rows, close session, close database.
--
-- Requires Lua 5.4+ and dkjson (install via: ./setup.sh)
--
-- Run with: lua basic_usage.lua

------------------------------------------------------------------------
-- SDK path bootstrapper (identical logic to drug_discovery.lua)
------------------------------------------------------------------------

local function resolve_sdk_path()
  local env = os.getenv("GRAPHLITE_LUA_SDK")
  if env and env ~= "" then return env end

  local home = os.getenv("HOME") or os.getenv("USERPROFILE") or ""
  local candidates = {
    home .. "/github/simbo1905/graphlite/lua-sdk",
  }
  for _, path in ipairs(candidates) do
    local f = io.open(path .. "/src/connection.lua", "r")
    if f then f:close(); return path end
  end
  return nil
end

local sdk_path = resolve_sdk_path()
if not sdk_path then
  io.stderr:write([[
ERROR: GraphLite Lua SDK not found.

Set GRAPHLITE_LUA_SDK or checkout the luajit-sdk branch at:
  ~/github/simbo1905/graphlite/
Then run: cd lua-sdk && ./setup.sh

See examples/lua/sdk/README.md for full setup instructions.
]])
  os.exit(1)
end

package.path = sdk_path .. "/?.lua;" .. sdk_path .. "/?/init.lua;" .. package.path

------------------------------------------------------------------------
-- Example
------------------------------------------------------------------------
local GraphLite = require("src.connection").GraphLite

local function main()
  print("=== GraphLite Lua SDK -- Basic Usage ===\n")

  local db_path = "./basic_usage_lua_sdk_db"
  os.execute("rm -rf " .. db_path .. " 2>/dev/null")

  local ok, err = xpcall(function()
    -- Open database
    local db = GraphLite.open(db_path)
    print("Opened database at " .. db_path)
    print("GraphLite version: " .. GraphLite.version())

    -- Create session
    local session = db:session("admin")
    print("Session created for user: " .. session:username())

    -- Schema / graph
    session:execute("CREATE SCHEMA IF NOT EXISTS /basic")
    session:execute("SESSION SET SCHEMA /basic")
    session:execute("CREATE GRAPH IF NOT EXISTS demo")
    session:execute("SESSION SET GRAPH demo")
    print("Schema and graph ready")

    -- Insert some nodes
    session:execute([[INSERT
      (:Person {name: 'Alice', age: 30}),
      (:Person {name: 'Bob',   age: 25}),
      (:Person {name: 'Carol', age: 35})
    ]])
    print("Inserted 3 Person nodes")

    -- Insert a relationship
    session:execute([[
      MATCH (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'})
      INSERT (a)-[:KNOWS {since: '2020-01-01'}]->(b)
    ]])
    print("Created KNOWS relationship")

    -- Query all persons
    print("\nAll persons (ordered by age):")
    local result = session:query(
      "MATCH (p:Person) RETURN p.name AS Name, p.age AS Age ORDER BY p.age")
    print("  Columns: " .. table.concat(result.variables, ", "))
    for _, row in ipairs(result.rows) do
      print(string.format("  %s (age %s)", tostring(row.Name), tostring(row.Age)))
    end

    -- Query relationship
    print("\nKNOWS relationships:")
    result = session:query([[
      MATCH (a:Person)-[k:KNOWS]->(b:Person)
      RETURN a.name AS From, b.name AS To, k.since AS Since
    ]])
    for _, row in ipairs(result.rows) do
      print(string.format("  %s -> %s (since %s)",
        tostring(row.From), tostring(row.To), tostring(row.Since)))
    end

    -- Error handling demo
    print("\nError handling demo:")
    local eok, eerr = pcall(function()
      session:execute("THIS IS NOT VALID GQL")
    end)
    if not eok then
      print("  Caught expected error: " .. tostring(eerr))
    end

    -- Cleanup
    session:close()
    db:close()
    print("\nSession and database closed.")
    print("To clean up: rm -rf " .. db_path)

  end, function(e)
    return tostring(e) .. "\n" .. debug.traceback()
  end)

  if not ok then
    io.stderr:write("\nError: " .. tostring(err) .. "\n")
    os.exit(1)
  end
end

main()
