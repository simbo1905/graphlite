--[[
  GraphLite SDK - Query result wrapper.
  Parses JSON from FFI and provides Lua-friendly rows with flattened values.
]]

local function extract_value(val)
  if type(val) ~= "table" then
    return val
  end
  if val.String then return val.String end
  if val.Number then
    local n = val.Number
    if type(n) == "number" and n == math.floor(n) then
      return math.floor(n)
    end
    return n
  end
  if val.Boolean then return val.Boolean end
  if val.Null then return nil end
  if val.List then
    local out = {}
    for i, v in ipairs(val.List) do
      out[i] = extract_value(v)
    end
    return out
  end
  if val.Array then
    local out = {}
    for i, v in ipairs(val.Array) do
      out[i] = extract_value(v)
    end
    return out
  end
  if val.Map then
    local out = {}
    for k, v in pairs(val.Map) do
      out[k] = extract_value(v)
    end
    return out
  end
  -- Node, Edge, Path - return as-is
  if val.Node or val.Edge or val.Path then
    return val
  end
  return val
end

local function flatten_row(row)
  if type(row) ~= "table" then return row end
  if not row.values then return row end
  local out = {}
  for k, v in pairs(row.values) do
    out[k] = extract_value(v)
  end
  return out
end

local QueryResult = {}
QueryResult.__index = QueryResult

function QueryResult.new(data)
  local variables = data.variables or {}
  local raw_rows = data.rows or {}
  local rows = {}
  for i, row in ipairs(raw_rows) do
    rows[i] = flatten_row(row)
  end
  return setmetatable({
    _data = data,
    variables = variables,
    rows = rows,
    row_count = #rows,
  }, QueryResult)
end

function QueryResult:first()
  return self.rows[1]
end

function QueryResult:column(name)
  local out = {}
  for _, row in ipairs(self.rows) do
    table.insert(out, row[name])
  end
  return out
end

function QueryResult:__tostring()
  return string.format("QueryResult(rows=%d, variables=%s)", self.row_count, table.concat(self.variables, ", "))
end

return QueryResult
