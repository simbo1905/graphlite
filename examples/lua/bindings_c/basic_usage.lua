local gl = require("graphlite_lua")

math.randomseed(os.time())

local function path_separator()
    return package.config:sub(1, 1)
end

local function make_temp_dir()
    local sep = path_separator()
    local base = os.getenv("TMPDIR") or os.getenv("TEMP") or (sep == "\\" and "." or "/tmp")
    local dir = base .. sep .. string.format("graphlite_lua_%d_%d", os.time(), math.random(1000, 9999))

    if sep == "\\" then
        os.execute(('mkdir "%s" >NUL 2>NUL'):format(dir))
    else
        os.execute(('mkdir -p "%s" >/dev/null 2>&1'):format(dir))
    end

    return dir
end

local function remove_dir(path)
    if not path or path == "" then
        return
    end

    if path_separator() == "\\" then
        os.execute(('rmdir /S /Q "%s" >NUL 2>NUL'):format(path))
    else
        os.execute(('rm -rf "%s" >/dev/null 2>&1'):format(path))
    end
end

local function safe_close(db, session)
    if db and session then
        pcall(function()
            db:close_session(session)
        end)
    end

    if db then
        pcall(function()
            db:close()
        end)
    end
end

print("=== GraphLite Lua 5.4 C Module Example ===\n")

print("1. Creating temporary database directory...")
local db_path = make_temp_dir()
print("   [OK] Using temporary database: " .. db_path .. "\n")

local db
local session

local ok, err = xpcall(function()
    print("2. Opening database...")
    db = gl.open(db_path)
    print("   [OK] GraphLite version: " .. gl.version() .. "\n")

    print("3. Creating session...")
    session = db:create_session("admin")
    print(("   [OK] Session created: %s...\n"):format(session:sub(1, 20)))

    print("4. Setting up schema and graph...")
    db:execute(session, "CREATE SCHEMA IF NOT EXISTS /example")
    db:execute(session, "SESSION SET SCHEMA /example")
    db:execute(session, "CREATE GRAPH IF NOT EXISTS social")
    db:execute(session, "SESSION SET GRAPH social")
    print("   [OK] Schema and graph created\n")

    print("5. Inserting data...")
    db:execute(session, "INSERT (:Person {name: 'Alice', age: 30})")
    db:execute(session, "INSERT (:Person {name: 'Bob', age: 25})")
    db:execute(session, "INSERT (:Person {name: 'Charlie', age: 35})")
    print("   [OK] Inserted 3 persons\n")

    print("6. Querying: all persons (name, age)...")
    local result = db:query(session, "MATCH (p:Person) RETURN p.name as name, p.age as age")
    print(("   Found %d persons:"):format(result.row_count))
    for _, row in ipairs(result.rows) do
        print(("   - %s: %s years old"):format(tostring(row.name), tostring(row.age)))
    end
    print()

    print("7. Querying: persons with age > 25 ordered by age...")
    result = db:query(
        session,
        "MATCH (p:Person) WHERE p.age > 25 RETURN p.name as name, p.age as age ORDER BY p.age ASC"
    )
    print(("   Found %d persons over 25:"):format(result.row_count))
    for _, row in ipairs(result.rows) do
        print(("   - %s: %s years old"):format(tostring(row.name), tostring(row.age)))
    end
    print()

    print("8. Aggregation: count + avg(age)...")
    result = db:query(session, "MATCH (p:Person) RETURN count(p) as total, avg(p.age) as avg_age")
    if result.row_count > 0 then
        local stats = result.rows[1]
        print("   Total persons: " .. tostring(stats.total))

        local avg_age = tonumber(stats.avg_age)
        if avg_age then
            print(("   Average age: %.1f"):format(avg_age))
        else
            print("   Average age: " .. tostring(stats.avg_age))
        end
    end
    print()

    print("9. Closing session and database...")
    db:close_session(session)
    db:close()
    session = nil
    db = nil
    print("   [OK] Session and database closed")
end, debug.traceback)

if not ok then
    io.stderr:write("\n[ERROR] " .. tostring(err) .. "\n")
    safe_close(db, session)
    remove_dir(db_path)
    os.exit(1)
end

remove_dir(db_path)
print("\n=== Example completed successfully ===")
