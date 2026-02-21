-- xmake orchestration for fixture suites + Rust tests.
--
-- Run:
--   xmake run fetch_ms_suite
--   xmake run fetch_jsonpath_suite
--   xmake run test_all

local MS_JDT_COMMIT = "92f78c6106de9509d4dd27bd4943674e0c2617b1"
local MS_JDT_ZIP_SHA256 = "6c285c9e8e7e60e584abf101bf97165c594728835c04ecca6610591c3a38d893"

local JSONPATH_CTS_COMMIT = "b9d7153e58711ad38bb8e35ece69c13f4b2f7d63"
local JSONPATH_CTS_ZIP_SHA256 = "f8faffd551d1bb9102211fdae78880064948d751afabae697b38ce303f157a7a"

local function sha256(osmod, filepath)
    if osmod.host() == "windows" then
        -- certutil outputs: "SHA256 hash of file:\n<hex>\nCertUtil: ..."
        local out = osmod.iorunv("certutil", {"-hashfile", filepath, "SHA256"})
        return out:match("\n(%w+)\n")
    else
        local out = osmod.iorunv("shasum", {"-a", "256", filepath})
        return out:match("^(%w+)")
    end
end

local function unzip(osmod, zipfile, destdir)
    if osmod.host() == "windows" then
        osmod.vrunv("powershell", {"-NoProfile", "-Command",
            "Expand-Archive -Force -Path '" .. zipfile .. "' -DestinationPath '" .. destdir .. "'"})
    else
        osmod.vrunv("unzip", {"-q", "-o", zipfile, "-d", destdir})
    end
end

local function fetch_ms_suite(osmod, pathmod)
    local dir = pathmod.join(osmod.projectdir(), ".tmp", "json-document-transforms", MS_JDT_COMMIT)
    local zip = pathmod.join(dir, "suite.zip")
    local root = pathmod.join(dir, "json-document-transforms-" .. MS_JDT_COMMIT)
    local inputs = pathmod.join(root, "test", "Microsoft.VisualStudio.Jdt.Tests", "Inputs")

    osmod.mkdir(dir)
    if not osmod.exists(zip) then
        local url = "https://codeload.github.com/microsoft/json-document-transforms/zip/" .. MS_JDT_COMMIT
        osmod.vrunv("curl", {"-f", "-s", "-S", "-L", url, "-o", zip})
    end
    local zhash = sha256(osmod, zip)
    if zhash ~= MS_JDT_ZIP_SHA256 then
        raise("ms suite zip sha256 mismatch: expected " .. MS_JDT_ZIP_SHA256 .. ", got " .. tostring(zhash))
    end
    if not osmod.exists(inputs) then
        unzip(osmod, zip, dir)
    end
    if not osmod.exists(inputs) then
        raise("ms suite inputs not found at " .. inputs)
    end
    return inputs
end

local function fetch_jsonpath_suite(osmod, pathmod)
    local dir = pathmod.join(osmod.projectdir(), ".tmp", "jsonpath-compliance-test-suite", JSONPATH_CTS_COMMIT)
    local zip = pathmod.join(dir, "suite.zip")
    local root = pathmod.join(dir, "jsonpath-compliance-test-suite-" .. JSONPATH_CTS_COMMIT)
    local cts = pathmod.join(root, "cts.json")

    osmod.mkdir(dir)
    if not osmod.exists(zip) then
        local url = "https://codeload.github.com/jsonpath-standard/jsonpath-compliance-test-suite/zip/" .. JSONPATH_CTS_COMMIT
        osmod.vrunv("curl", {"-f", "-s", "-S", "-L", url, "-o", zip})
    end
    local zhash = sha256(osmod, zip)
    if zhash ~= JSONPATH_CTS_ZIP_SHA256 then
        raise("jsonpath cts zip sha256 mismatch: expected " .. JSONPATH_CTS_ZIP_SHA256 .. ", got " .. tostring(zhash))
    end
    if not osmod.exists(cts) then
        unzip(osmod, zip, dir)
    end
    if not osmod.exists(cts) then
        raise("jsonpath cts not found at " .. cts)
    end
    return cts
end

target("fetch_ms_suite")
    set_kind("phony")
    on_run(function ()
        local inputs = fetch_ms_suite(os, path)
        cprint("${green}OK:${clear} " .. inputs)
    end)
target_end()

target("fetch_jsonpath_suite")
    set_kind("phony")
    on_run(function ()
        local cts = fetch_jsonpath_suite(os, path)
        cprint("${green}OK:${clear} " .. cts)
    end)
target_end()

target("test_jdt")
    set_kind("phony")
    on_run(function ()
        local inputs = fetch_ms_suite(os, path)
        os.setenv("JDT_MS_INPUTS_DIR", inputs)
        os.vrunv("cargo", {"test", "-p", "jdt-codegen", "--test", "ms_jdt_suite", "--", "--nocapture"})
    end)
target_end()

target("test_jsonpath")
    set_kind("phony")
    on_run(function ()
        local cts = fetch_jsonpath_suite(os, path)
        os.setenv("JSONPATH_CTS_JSON", cts)
        os.vrunv("cargo", {"test", "-p", "jdt-codegen", "--test", "jsonpath_cts", "--", "--nocapture"})
    end)
target_end()

target("test_all")
    set_kind("phony")
    on_run(function ()
        -- Quality checks
        os.vrunv("cargo", {"fmt", "--all", "--", "--check"})
        os.vrunv("cargo", {"clippy", "--workspace", "--all-targets", "--", "-D", "warnings"})

        -- Run integration test suites
        local inputs = fetch_ms_suite(os, path)
        local cts = fetch_jsonpath_suite(os, path)
        
        os.setenv("JDT_MS_INPUTS_DIR", inputs)
        os.setenv("JSONPATH_CTS_JSON", cts)
        
        -- Run all tests (unit + integration)
        os.vrunv("cargo", {"test", "--workspace"})
        
        cprint("${green}OK:${clear} test_all")
    end)
target_end()
