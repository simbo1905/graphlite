#!/usr/bin/env luajit
--[[
  Basic Usage - Minimal sanity test for GraphLite LuaJIT SDK.
  Run: luajit basic_usage.lua
]]

local src = debug.getinfo(1, "S").source
if src:sub(1, 1) == "@" then src = src:sub(2) end
local script_dir = src:match("(.+)/[^/]+$") or "."
package.path = script_dir .. "/?.lua;" .. package.path

require("bootstrap")

local connection = require("src.connection")
local GraphLite = connection.GraphLite

local tmp = os.getenv("TMPDIR") or os.getenv("TMP") or "/tmp"
local db_path = tmp .. "/graphlite_lua_basic_" .. os.time()

print("=== GraphLite LuaJIT SDK Basic Usage ===\n")

local db = GraphLite.open(db_path)
local session = db:session("admin")

session:execute("CREATE SCHEMA IF NOT EXISTS /example")
session:execute("SESSION SET SCHEMA /example")
session:execute("CREATE GRAPH IF NOT EXISTS social")
session:execute("SESSION SET GRAPH social")
session:execute("INSERT (p:Person {name: 'Alice', age: 30})")
session:execute("INSERT (p:Person {name: 'Bob', age: 25})")

local result = session:query("MATCH (p:Person) RETURN p.name, p.age ORDER BY p.name")
print("Persons:", #result.rows)
for _, row in ipairs(result.rows) do
  print("  -", row["p.name"], row["p.age"])
end

session:close()
db:close()

os.execute("rm -rf " .. db_path)
print("\n✓ Basic usage completed successfully")
