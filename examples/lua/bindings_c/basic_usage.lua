math.randomseed(os.time())

local function path_separator()
    return package.config:sub(1, 1)
end

local function script_dir()
    local source = debug.getinfo(1, "S").source
    if type(source) == "string" and source:sub(1, 1) == "@" then
        source = source:sub(2)
    end

    if type(source) ~= "string" then
        return "."
    end

    return source:match("^(.*)[/\\].-$") or "."
end

local function add_local_rocks_paths(base_dir)
    local sep = path_separator()
    local lua_version = _VERSION:match("(%d+%.%d+)") or "5.4"
    local rocks_share = table.concat({ base_dir, ".rocks", "share", "lua", lua_version }, sep)

    package.path =
        rocks_share .. sep .. "?.lua;" ..
        rocks_share .. sep .. "?" .. sep .. "init.lua;" ..
        package.path
end

local function load_dkjson()
    add_local_rocks_paths(script_dir())

    local ok, mod = pcall(require, "dkjson")
    if not ok then
        error(
            "dkjson is not installed. Run ./setup.sh first.\n" ..
            "Original error: " .. tostring(mod)
        )
    end
    return mod
end

local json = load_dkjson()
local gl = require("graphlite_lua")

local VALUE_VARIANTS = {
    String = true,
    Number = true,
    Boolean = true,
    DateTime = true,
    DateTimeWithFixedOffset = true,
    DateTimeWithNamedTz = true,
    TimeWindow = true,
    Array = true,
    List = true,
    Vector = true,
    Path = true,
    Node = true,
    Edge = true,
    Temporal = true,
    Map = true,
    Null = true,
}

local function single_variant_tag(value)
    local first_key
    local key_count = 0

    for key, _ in pairs(value) do
        key_count = key_count + 1
        if key_count == 1 then
            first_key = key
        else
            return nil
        end
    end

    if key_count == 1 and type(first_key) == "string" and VALUE_VARIANTS[first_key] then
        return first_key
    end

    return nil
end

local function unwrap_value(value)
    if type(value) ~= "table" then
        return value
    end

    local variant = single_variant_tag(value)
    if variant then
        if variant == "Null" then
            return nil
        end
        return unwrap_value(value[variant])
    end

    local out = {}
    local n = #value

    if n > 0 then
        for i = 1, n do
            out[i] = unwrap_value(value[i])
        end
    end

    for key, inner in pairs(value) do
        local is_array_key =
            type(key) == "number" and
            key >= 1 and
            key <= n and
            key % 1 == 0

        if not is_array_key then
            out[key] = unwrap_value(inner)
        end
    end

    return out
end

local function decode_query_result(json_bytes)
    local parsed, pos, err = json.decode(json_bytes, 1, nil)
    if err then
        error(("Failed to decode GraphLite JSON at byte %s: %s"):format(tostring(pos), tostring(err)))
    end
    if type(parsed) ~= "table" then
        error("GraphLite query returned non-object JSON")
    end

    local rows = {}
    for _, raw_row in ipairs(parsed.rows or {}) do
        local source_values = raw_row
        if type(raw_row) == "table" and type(raw_row.values) == "table" then
            source_values = raw_row.values
        end

        local row = {}
        if type(source_values) == "table" then
            for key, inner in pairs(source_values) do
                row[key] = unwrap_value(inner)
            end
        end
        rows[#rows + 1] = row
    end

    return {
        rows = rows,
        row_count = #rows,
        variables = parsed.variables or {},
    }
end

local function query_rows(db, session_id, gql)
    local json_bytes = db:query(session_id, gql)
    return decode_query_result(json_bytes)
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
    local result = query_rows(db, session, "MATCH (p:Person) RETURN p.name as name, p.age as age")
    print(("   Found %d persons:"):format(result.row_count))
    for _, row in ipairs(result.rows) do
        print(("   - %s: %s years old"):format(tostring(row.name), tostring(row.age)))
    end
    print()

    print("7. Querying: persons with age > 25 ordered by age...")
    result = query_rows(
        db,
        session,
        "MATCH (p:Person) WHERE p.age > 25 RETURN p.name as name, p.age as age ORDER BY p.age ASC"
    )
    print(("   Found %d persons over 25:"):format(result.row_count))
    for _, row in ipairs(result.rows) do
        print(("   - %s: %s years old"):format(tostring(row.name), tostring(row.age)))
    end
    print()

    print("8. Aggregation: count + avg(age)...")
    result = query_rows(db, session, "MATCH (p:Person) RETURN count(p) as total, avg(p.age) as avg_age")
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
