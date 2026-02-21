local SEP = package.config:sub(1, 1)

local function join_path(...)
    local parts = { ... }
    local out = {}
    for i = 1, #parts do
        local part = parts[i]
        if part and part ~= "" then
            local value = part
            if #value > 1 and value:sub(-1) == SEP then
                value = value:sub(1, -2)
            end
            table.insert(out, value)
        end
    end
    return table.concat(out, SEP)
end

local function file_exists(path)
    local file = io.open(path, "rb")
    if file then
        file:close()
        return true
    end
    return false
end

local function sdk_layout_ok(sdk_root)
    return file_exists(join_path(sdk_root, "src", "connection.lua"))
        and file_exists(join_path(sdk_root, "src", "session.lua"))
        and file_exists(join_path(sdk_root, "src", "errors.lua"))
        and file_exists(join_path(sdk_root, "src", "graphlite_ffi.lua"))
end

local function default_candidates()
    local candidates = {}
    local home = os.getenv("HOME") or os.getenv("USERPROFILE")
    if home and home ~= "" then
        candidates[#candidates + 1] = join_path(
            home,
            "github",
            "deepgraphai",
            "GraphLite",
            "lua-sdk"
        )
    end
    return candidates
end

local function locate_sdk()
    local checked = {}

    local env_sdk = os.getenv("GRAPH_LITE_LUA_SDK")
    if env_sdk and env_sdk ~= "" then
        checked[#checked + 1] = env_sdk
        if sdk_layout_ok(env_sdk) then
            return env_sdk
        end
    end

    local defaults = default_candidates()
    for i = 1, #defaults do
        checked[#checked + 1] = defaults[i]
        if sdk_layout_ok(defaults[i]) then
            return defaults[i]
        end
    end

    local lines = {
        "LuaJIT SDK not found.",
        "",
        "Expected either:",
        "  1) GRAPH_LITE_LUA_SDK to point at a lua-sdk directory",
        "  2) default path: ~/github/deepgraphai/GraphLite/lua-sdk",
        "",
        "Checked paths:",
    }

    for i = 1, #checked do
        lines[#lines + 1] = "  - " .. checked[i]
    end

    lines[#lines + 1] = ""
    lines[#lines + 1] = "To install the SDK:"
    lines[#lines + 1] = "  cd ~/github/deepgraphai"
    lines[#lines + 1] = "  git clone https://github.com/deepgraphai/GraphLite.git"
    lines[#lines + 1] = "  cd GraphLite"
    lines[#lines + 1] = "  git checkout luajit-sdk"
    lines[#lines + 1] = ""
    lines[#lines + 1] = "Then rerun this example, or set GRAPH_LITE_LUA_SDK explicitly."

    if SEP == "\\" then
        lines[#lines + 1] = ""
        lines[#lines + 1] = "Windows hint: set GRAPH_LITE_LUA_SDK with:"
        lines[#lines + 1] = "  set GRAPH_LITE_LUA_SDK=C:\\path\\to\\GraphLite\\lua-sdk"
    end

    return nil, table.concat(lines, "\n")
end

local function add_to_package_path(sdk_root)
    local lua_path = join_path(sdk_root, "?.lua")
    local init_path = join_path(sdk_root, "?", "init.lua")

    if not package.path:find(lua_path, 1, true) then
        package.path = lua_path .. ";" .. init_path .. ";" .. package.path
    end
end

return {
    locate_sdk = locate_sdk,
    add_to_package_path = add_to_package_path,
}
