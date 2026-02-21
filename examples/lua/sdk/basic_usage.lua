local SEP = package.config:sub(1, 1)

local function script_dir()
    local source = debug.getinfo(1, "S").source
    if source:sub(1, 1) ~= "@" then
        return "."
    end
    local path = source:sub(2)
    return path:match("^(.*)[/\\][^/\\]+$") or "."
end

local local_path = script_dir() .. SEP .. "?.lua"
if not package.path:find(local_path, 1, true) then
    package.path = local_path .. ";" .. package.path
end

local locator = require("sdk_locator")

local sdk_root, locate_err = locator.locate_sdk()
if not sdk_root then
    io.stderr:write(locate_err .. "\n")
    os.exit(1)
end
locator.add_to_package_path(sdk_root)

local GraphLite = require("src.connection").GraphLite

local function remove_tree(path)
    if SEP == "\\" then
        os.execute(string.format('if exist "%s" rmdir /s /q "%s"', path, path))
    else
        os.execute(string.format('rm -rf "%s"', path))
    end
end

local function make_temp_db_dir()
    local base
    if SEP == "\\" then
        base = os.getenv("TEMP") or "."
    else
        base = os.getenv("TMPDIR") or "/tmp"
    end
    local stamp = tostring(os.time()) .. "_" .. tostring(math.random(1000, 9999))
    return base .. SEP .. "graphlite_lua_sdk_example_" .. stamp
end

local function run()
    math.randomseed(os.time())
    local db_path = make_temp_db_dir()
    remove_tree(db_path)

    local db = nil
    local session = nil
    local ok, err = pcall(function()
        print("=== GraphLite LuaJIT SDK Basic Usage ===")
        print("Using SDK from: " .. sdk_root)
        print("Temporary DB: " .. db_path .. "\n")

        db = GraphLite.open(db_path)
        session = db:session("admin")

        session:execute("CREATE SCHEMA IF NOT EXISTS /example")
        session:execute("SESSION SET SCHEMA /example")
        session:execute("CREATE GRAPH IF NOT EXISTS social")
        session:execute("SESSION SET GRAPH social")

        session:execute("INSERT (:Person {name: 'Alice', age: 31})")
        session:execute("INSERT (:Person {name: 'Bob', age: 27})")

        local result = session:query(
            "MATCH (p:Person) RETURN p.name AS name, p.age AS age ORDER BY p.age DESC"
        )

        print("Rows returned: " .. tostring(result.row_count))
        for i = 1, #result.rows do
            local row = result.rows[i]
            print(string.format("  - %s (%s)", tostring(row.name), tostring(row.age)))
        end
        print("\nBasic usage example passed.")
    end)

    if session then
        pcall(function()
            session:close()
        end)
    end
    if db then
        pcall(function()
            db:close()
        end)
    end
    remove_tree(db_path)

    if not ok then
        error(err, 0)
    end
end

local success, err = pcall(run)
if not success then
    io.stderr:write("Basic usage failed: " .. tostring(err) .. "\n")
    os.exit(1)
end
