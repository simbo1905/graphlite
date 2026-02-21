-- luacheck configuration for GraphLite Lua SDK
-- Warnings are treated as errors: the build MUST pass with zero warnings.

std = "lua51+luajit"
max_line_length = 120

-- The ffi module is provided by LuaJIT at runtime; not a regular Lua module.
-- Suppress "accessing undefined variable 'ffi'" globals from ffi.cdef usage.
globals = {}
read_globals = {
  "ffi",
}

-- Allow common patterns
allow_defined_top = true

-- Unused variable prefixed with _ is intentional (e.g. _, err_name = func())
unused_args = true
unused_secondaries = true

-- Per-file overrides for FFI-heavy modules
files["lua-sdk/src/graphlite_ffi.lua"] = {
  -- ffi is used via require("ffi") and all its members are needed for cdef/load/new/string
  ignore = { "212" },  -- unused argument (ffi callbacks may have unused params)
}
