#!/usr/bin/env lua5.4
--[[
  GraphLite Lua 5.4 Basic Usage Demo

  Minimal demo using the graphlite_lua C module and dkjson.
  Follows the same flow as examples/java/bindings/BasicUsage.java.

  Run: make run
]]

-- Add the locally installed luarocks directory to package.path
local script_dir = debug.getinfo(1, "S").source:sub(2):match("(.*[/\\])") or "./"
package.path = package.path .. ";" .. script_dir .. "lua_modules/share/lua/5.4/?.lua"

local dkjson = require("dkjson")

print("=== GraphLite Lua 5.4 Bindings Example ===\n")

-- Create temp dir for demo (Unix-style)
local function get_temp_db_path()
  local tmp = os.getenv("TMPDIR") or os.getenv("TEMP") or "/tmp"
  return tmp .. "/graphlite_lua_demo_" .. (os.getenv("USER") or "user")
end

local db_path = get_temp_db_path()
os.execute("rm -rf " .. db_path)
print("Using temporary database: " .. db_path .. "\n")

-- Load module (requires graphlite_lua.so in package.cpath)
local gl = require("graphlite_lua")

-- Helper to execute a query, parse JSON, and return a clean table
local function query_parsed(db, session_id, query_str)
  local raw_json = db:query(session_id, query_str)
  local result, pos, err = dkjson.decode(raw_json, 1, nil)
  if err then
    error("JSON Parse Error: " .. tostring(err) .. "\nRaw JSON: " .. raw_json)
  end
  return result
end

-- Helper to unwrap GraphLite's nested value types (e.g. {"String":"Alice"})
local function unwrap_value(v)
  if type(v) == "table" then
    if v.String then return v.String end
    if v.Number then return v.Number end
    if type(v.Boolean) ~= "nil" then return v.Boolean end
    if type(v.Null) ~= "nil" then return nil end
  end
  return v
end

-- 1. Open database
print("1. Opening database...")
local db = gl.open(db_path)
print("   [OK] GraphLite version: " .. gl.version() .. "\n")

-- 2. Create session
print("2. Creating session...")
local session = db:create_session("admin")
print("   [OK] Session created: " .. session:sub(1, 20) .. "...\n")

-- 3. Create schema and graph
print("3. Setting up schema and graph...")
db:execute(session, "CREATE SCHEMA IF NOT EXISTS /example")
db:execute(session, "SESSION SET SCHEMA /example")
db:execute(session, "CREATE GRAPH IF NOT EXISTS social")
db:execute(session, "SESSION SET GRAPH social")
print("   [OK] Schema and graph created\n")

-- 4. Insert data
print("4. Inserting data...")
db:execute(session, "INSERT (:Person {name: 'Alice', age: 30})")
db:execute(session, "INSERT (:Person {name: 'Bob', age: 25})")
db:execute(session, "INSERT (:Person {name: 'Charlie', age: 35})")
print("   [OK] Inserted 3 persons\n")

-- 5. Query data
print("5. Querying: All persons' age and name")
local result = query_parsed(db, session, "MATCH (p:Person) RETURN p.name as name, p.age as age")
local rows = result.rows or {}
print("   Found " .. #rows .. " persons:")
for _, row in ipairs(rows) do
  local name = unwrap_value(row.values.name)
  local age = unwrap_value(row.values.age)
  print("   - " .. tostring(name) .. ": " .. tostring(age) .. " years old")
end
print()

-- 6. Filter with WHERE
print("6. Filtering: Persons older than 25 years in the ascending order of age")
result = query_parsed(db, session,
  "MATCH (p:Person) WHERE p.age > 25 " ..
  "RETURN p.name as name, p.age as age ORDER BY p.age ASC")
rows = result.rows or {}
print("   Found " .. #rows .. " persons over 25:")
for _, row in ipairs(rows) do
  local name = unwrap_value(row.values.name)
  local age = unwrap_value(row.values.age)
  print("   - " .. tostring(name) .. ": " .. tostring(age) .. " years old")
end
print()

-- 7. Aggregation
print("7. Aggregation query...")
result = query_parsed(db, session, "MATCH (p:Person) RETURN count(p) as total, avg(p.age) as avg_age")
rows = result.rows or {}
if #rows > 0 then
  local row = rows[1]
  local total = unwrap_value(row.values.total)
  local avg_age = unwrap_value(row.values.avg_age)
  print("   Total persons: " .. tostring(total))
  if type(avg_age) == "number" then
    print("   Average age: " .. string.format("%.2f", avg_age))
  else
    print("   Average age: " .. tostring(avg_age))
  end
end
print()

-- 8. Get column values
print("8. Extracting column values...")
result = query_parsed(db, session, "MATCH (p:Person) RETURN p.name as name")
rows = result.rows or {}
local names = {}
for _, row in ipairs(rows) do
  table.insert(names, tostring(unwrap_value(row.values.name)))
end
print("   All names: " .. table.concat(names, ", ") .. "\n")

-- 9. Close session
print("9. Closing session...")
db:close_session(session)
print("   [OK] Session closed\n")

-- 10. Close database
print("10. Closing database...")
db:close()
print("   [OK] Database closed\n")

print("=== Example completed successfully ===")
