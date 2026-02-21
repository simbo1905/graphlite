#!/usr/bin/env lua5.4
--[[
  GraphLite Lua 5.4 Basic Usage Demo

  Minimal demo using the graphlite_lua C module.
  Follows the same flow as examples/java/bindings/BasicUsage.java.

  Run: lua5.4 basic_usage.lua
  (Ensure graphlite_lua.so is in package.cpath and libgraphlite_ffi is in library path)
]]

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
local result = db:query(session, "MATCH (p:Person) RETURN p.name as name, p.age as age")
print("   Found " .. result.row_count .. " persons:")
for _, row in ipairs(result.rows) do
  print("   - " .. tostring(row.name) .. ": " .. tostring(row.age) .. " years old")
end
print()

-- 6. Filter with WHERE
print("6. Filtering: Persons older than 25 years in the ascending order of age")
result = db:query(session,
  "MATCH (p:Person) WHERE p.age > 25 " ..
  "RETURN p.name as name, p.age as age ORDER BY p.age ASC")
print("   Found " .. result.row_count .. " persons over 25:")
for _, row in ipairs(result.rows) do
  print("   - " .. tostring(row.name) .. ": " .. tostring(row.age) .. " years old")
end
print()

-- 7. Aggregation
print("7. Aggregation query...")
result = db:query(session, "MATCH (p:Person) RETURN count(p) as total, avg(p.age) as avg_age")
if result.row_count > 0 then
  local row = result.rows[1]
  print("   Total persons: " .. tostring(row.total))
  local avg_age = row.avg_age
  if type(avg_age) == "number" then
    print("   Average age: " .. string.format("%.2f", avg_age))
  else
    print("   Average age: " .. tostring(avg_age))
  end
end
print()

-- 8. Get column values
print("8. Extracting column values...")
result = db:query(session, "MATCH (p:Person) RETURN p.name as name")
local names = {}
for _, row in ipairs(result.rows) do
  table.insert(names, tostring(row.name))
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
