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

local function fmt_value(v)
    if type(v) == "table" then
        return "<table>"
    end
    return tostring(v)
end

local function print_result(title, result)
    print(title)
    if not result.rows or #result.rows == 0 then
        print("  (no rows)\n")
        return
    end

    for i = 1, #result.rows do
        local row = result.rows[i]
        local parts = {}

        if result.variables and #result.variables > 0 then
            for j = 1, #result.variables do
                local key = result.variables[j]
                parts[#parts + 1] = key .. "=" .. fmt_value(row[key])
            end
        else
            for key, value in pairs(row) do
                parts[#parts + 1] = key .. "=" .. fmt_value(value)
            end
            table.sort(parts)
        end

        print("  - " .. table.concat(parts, ", "))
    end
    print("")
end

local function run()
    print("=== GraphLite LuaJIT High-Level SDK Drug Discovery Example ===")
    print("Using SDK from: " .. sdk_root .. "\n")

    local db_path = "./drug_discovery_lua_sdk_db"
    remove_tree(db_path)

    local db = nil
    local session = nil

    local ok, err = pcall(function()
        print("1) Opening database...")
        db = GraphLite.open(db_path)
        print("   GraphLite version: " .. db:version() .. "\n")

        print("2) Creating session...")
        session = db:session("researcher")
        print("   Session ready\n")

        print("3) Configuring schema + graph...")
        session:execute("CREATE SCHEMA IF NOT EXISTS /drug_discovery")
        session:execute("SESSION SET SCHEMA /drug_discovery")
        session:execute("CREATE GRAPH IF NOT EXISTS pharma_research")
        session:execute("SESSION SET GRAPH pharma_research")
        print("   Schema and graph configured\n")

        print("4) Inserting representative data...")
        session:execute([[
            INSERT
                (:Protein {id: 'TP53', name: 'Tumor Protein P53', disease: 'Cancer'}),
                (:Protein {id: 'EGFR', name: 'Epidermal Growth Factor Receptor', disease: 'Cancer'}),
                (:Protein {id: 'ACE2', name: 'Angiotensin-Converting Enzyme 2', disease: 'Hypertension'})
        ]])

        session:execute([[
            INSERT
                (:Compound {id: 'CP-002', name: 'Gefitinib', stage: 'Approved'}),
                (:Compound {id: 'CP-003', name: 'Captopril', stage: 'Approved'}),
                (:Compound {id: 'CP-005', name: 'APG-115', stage: 'Clinical Trial'})
        ]])

        session:execute([[
            INSERT
                (:Assay {id: 'AS-001', name: 'EGFR Kinase Assay', assay_type: 'Enzymatic'}),
                (:Assay {id: 'AS-002', name: 'ACE2 Binding Assay', assay_type: 'Binding'}),
                (:Assay {id: 'AS-003', name: 'TP53 Interaction Assay', assay_type: 'PPI'})
        ]])

        session:execute([[
            MATCH (c:Compound {id: 'CP-002'}), (a:Assay {id: 'AS-001'})
            INSERT (c)-[:TESTED_IN {test_date: '2024-01-15'}]->(a)
        ]])
        session:execute([[
            MATCH (c:Compound {id: 'CP-003'}), (a:Assay {id: 'AS-002'})
            INSERT (c)-[:TESTED_IN {test_date: '2024-02-20'}]->(a)
        ]])
        session:execute([[
            MATCH (c:Compound {id: 'CP-005'}), (a:Assay {id: 'AS-003'})
            INSERT (c)-[:TESTED_IN {test_date: '2024-03-25'}]->(a)
        ]])

        session:execute([[
            MATCH (a:Assay {id: 'AS-001'}), (p:Protein {id: 'EGFR'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON]->(p)
        ]])
        session:execute([[
            MATCH (a:Assay {id: 'AS-002'}), (p:Protein {id: 'ACE2'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON]->(p)
        ]])
        session:execute([[
            MATCH (a:Assay {id: 'AS-003'}), (p:Protein {id: 'TP53'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON]->(p)
        ]])

        session:execute([[
            MATCH (c:Compound {id: 'CP-002'}), (p:Protein {id: 'EGFR'})
            INSERT (c)-[:INHIBITS {IC50: 37.5, unit: 'nM'}]->(p)
        ]])
        session:execute([[
            MATCH (c:Compound {id: 'CP-003'}), (p:Protein {id: 'ACE2'})
            INSERT (c)-[:INHIBITS {IC50: 23.0, unit: 'nM'}]->(p)
        ]])
        session:execute([[
            MATCH (c:Compound {id: 'CP-005'}), (p:Protein {id: 'TP53'})
            INSERT (c)-[:INHIBITS {IC50: 12.5, unit: 'nM'}]->(p)
        ]])
        print("   Data insertion complete\n")

        print("5) Running analytical queries...\n")

        local potent = session:query([[
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
            WHERE i.IC50 < 30
            RETURN c.name AS compound, p.name AS target, i.IC50 AS ic50_nM
            ORDER BY i.IC50
        ]])
        print_result("Query A: Potent compounds (IC50 < 30 nM)", potent)

        local traversal = session:query([[
            MATCH (c:Compound {id: 'CP-002'})-[:TESTED_IN]->(a:Assay)-[:MEASURES_ACTIVITY_ON]->(p:Protein)
            RETURN c.name AS compound, a.name AS assay, p.name AS target
        ]])
        print_result("Query B: Traversal from compound -> assay -> target", traversal)

        local aggregation = session:query([[
            MATCH (p:Protein)<-[:INHIBITS]-(c:Compound)
            RETURN p.name AS protein, COUNT(c) AS inhibitor_count
        ]])
        print_result("Query C: Aggregation by target protein", aggregation)

        print("=== Example complete ===")
        print("Database path: " .. db_path)
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

    if not ok then
        error(err, 0)
    end
end

local success, err = pcall(run)
if not success then
    io.stderr:write("\nExample failed: " .. tostring(err) .. "\n")
    os.exit(1)
end
