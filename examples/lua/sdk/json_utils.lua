local SEP = package.config:sub(1, 1)

local function prepend_once(path_value, existing)
    if existing:find(path_value, 1, true) then
        return existing
    end
    return path_value .. ";" .. existing
end

local function append_rocks_paths(example_dir)
    local roots = {
        example_dir .. SEP .. ".luarocks" .. SEP .. "share" .. SEP .. "lua",
        example_dir .. SEP .. ".luarocks" .. SEP .. "lib" .. SEP .. "lua",
    }
    local versions = { "5.4", "5.3", "5.2", "5.1" }

    local path_value = package.path
    local cpath_value = package.cpath

    for _, root in ipairs(roots) do
        for _, version in ipairs(versions) do
            path_value = prepend_once(root .. SEP .. version .. SEP .. "?.lua", path_value)
            path_value = prepend_once(root .. SEP .. version .. SEP .. "?" .. SEP .. "init.lua", path_value)
            cpath_value = prepend_once(root .. SEP .. version .. SEP .. "?.so", cpath_value)
            cpath_value = prepend_once(root .. SEP .. version .. SEP .. "?.dylib", cpath_value)
            cpath_value = prepend_once(root .. SEP .. version .. SEP .. "?.dll", cpath_value)
        end
    end

    package.path = path_value
    package.cpath = cpath_value
end

local function unwrap_value(value)
    if type(value) ~= "table" then
        return value
    end

    if value.String ~= nil then
        return value.String
    elseif value.Number ~= nil then
        return value.Number
    elseif value.Boolean ~= nil then
        return value.Boolean
    elseif value.Null ~= nil then
        return nil
    elseif value.List ~= nil then
        local out = {}
        for i, item in ipairs(value.List) do
            out[i] = unwrap_value(item)
        end
        return out
    elseif value.Map ~= nil then
        local out = {}
        for k, item in pairs(value.Map) do
            out[k] = unwrap_value(item)
        end
        return out
    end

    return value
end

local function decode_query_json(raw_json, dkjson)
    local decoded, _, err = dkjson.decode(raw_json, 1, nil)
    if err then
        return nil, err
    end

    local rows = {}
    local raw_rows = (type(decoded) == "table" and decoded.rows) or {}
    for i, row in ipairs(raw_rows) do
        local values = row.values or row
        local out = {}
        for key, value in pairs(values) do
            out[key] = unwrap_value(value)
        end
        rows[i] = out
    end

    local variables = (type(decoded) == "table" and decoded.variables) or {}
    return {
        variables = variables,
        rows = rows,
        row_count = #rows,
        raw = decoded,
    }
end

local function require_dkjson(example_dir)
    append_rocks_paths(example_dir)
    local ok, dkjson_or_err = pcall(require, "dkjson")
    if not ok then
        return nil, "dkjson not found. Run ./setup.sh first (" .. tostring(dkjson_or_err) .. ")"
    end
    return dkjson_or_err
end

return {
    require_dkjson = require_dkjson,
    decode_query_json = decode_query_json,
}
