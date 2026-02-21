#!/usr/bin/env lua5.4
--
-- GraphLite Lua 5.4 Bindings - Basic Usage Example
--
-- Lua 5.4 has no built-in FFI; this demo uses a tiny C module
-- (graphlite_lua.c) that calls the GraphLite FFI shared library.
--
-- Run:  lua5.4 basic_usage.lua
--

local gl = require("graphlite_lua")

-- Portable temp-dir helper (os.tmpname gives a file; we need a directory).
local function make_temp_dir()
    local tmp = os.tmpname()          -- e.g. /tmp/lua_XXXXXX
    os.remove(tmp)                    -- remove the file …
    local dir = tmp .. "_graphlite"
    os.execute("mkdir -p " .. dir)    -- … and create a directory instead
    return dir
end

-- Remove a directory tree (best-effort, non-portable but fine for demos).
local function remove_tree(dir)
    os.execute("rm -rf " .. dir .. " 2>/dev/null")
end

-- Format a value for display: show round floats as integers.
local function fmtval(v)
    if type(v) == "number" then
        if v == math.floor(v) then return string.format("%d", v) end
        return tostring(v)
    end
    return tostring(v)
end

-- Pretty-print helper for result rows.
local function print_rows(rows, fmt)
    if not rows then return end
    for _, row in ipairs(rows) do
        print(fmt(row))
    end
end

-- ===================================================================
local function main()
    print("=== GraphLite Lua 5.4 Bindings Example ===\n")

    local db_path = make_temp_dir()
    print("Using temporary database: " .. db_path .. "\n")

    local ok, err = pcall(function()

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

        -- 5. Query all persons
        print("5. Querying: All persons (name, age)")
        local result = db:query(session,
            "MATCH (p:Person) RETURN p.name as name, p.age as age")
        print("   Found " .. (result.row_count or #result.rows) .. " persons:")
        print_rows(result.rows, function(r)
            return "   - " .. fmtval(r.name) .. ": " .. fmtval(r.age) .. " years old"
        end)
        print()

        -- 6. Filter with WHERE + ORDER BY
        print("6. Filtering: Persons older than 25 (ascending by age)")
        result = db:query(session,
            "MATCH (p:Person) WHERE p.age > 25 " ..
            "RETURN p.name as name, p.age as age ORDER BY p.age ASC")
        print("   Found " .. (result.row_count or #result.rows) ..
              " persons over 25:")
        print_rows(result.rows, function(r)
            return "   - " .. fmtval(r.name) .. ": " .. fmtval(r.age) .. " years old"
        end)
        print()

        -- 7. Aggregation
        print("7. Aggregation query...")
        result = db:query(session,
            "MATCH (p:Person) RETURN count(p) as total, avg(p.age) as avg_age")
        if result.rows and #result.rows > 0 then
            local row = result.rows[1]
            print("   Total persons: " .. fmtval(row.total))
            print("   Average age: " .. fmtval(row.avg_age))
        end
        print()

        -- 8. Close session
        print("8. Closing session...")
        db:close_session(session)
        print("   [OK] Session closed\n")

        -- 9. Close database
        print("9. Closing database...")
        db:close()
        print("   [OK] Database closed\n")

    end)

    -- Cleanup temp directory regardless of outcome.
    remove_tree(db_path)

    if ok then
        print("=== Example completed successfully ===")
    else
        io.stderr:write("\n[ERROR] " .. tostring(err) .. "\n")
        os.exit(1)
    end
end

main()
