#!/usr/bin/env lua
--- Drug Discovery Example using GraphLite High-Level Lua SDK
--
-- This example demonstrates how the GraphLite High-Level SDK can be used for
-- pharmaceutical research, modeling the relationships between compounds, targets
-- (proteins), and assays.
--
-- Requires Lua 5.4+ and dkjson (install via: ./setup.sh)
--
-- This version uses the high-level SDK from the luajit-sdk branch, which provides:
--   - Session-centric API (session objects vs session IDs)
--   - Typed error hierarchy
--   - Cleaner interface matching the Rust and Python SDKs
--
-- Domain Model:
--   - Targets: Proteins or enzymes that play a key role in a disease (e.g., TP53 in cancer)
--   - Compounds: Small molecules that can bind to or inhibit those proteins
--   - Assays: Experiments that measure how strongly a compound affects a target
--
-- Graph Structure:
--   Compound -> TESTED_IN -> Assay -> MEASURES_ACTIVITY_ON -> Target (Protein)
--   Compound -> INHIBITS -> Target (with IC50 measurements)
--
-- Run with: lua drug_discovery.lua

------------------------------------------------------------------------
-- SDK path bootstrapper
------------------------------------------------------------------------

local function resolve_sdk_path()
  -- 1. Environment variable override
  local env = os.getenv("GRAPHLITE_LUA_SDK")
  if env and env ~= "" then
    return env
  end

  -- 2. Default paths
  local home = os.getenv("HOME") or os.getenv("USERPROFILE") or ""
  local candidates = {
    home .. "/github/simbo1905/graphlite/lua-sdk",
  }

  for _, path in ipairs(candidates) do
    local f = io.open(path .. "/src/connection.lua", "r")
    if f then
      f:close()
      return path
    end
  end

  return nil
end

local sdk_path = resolve_sdk_path()
if not sdk_path then
  io.stderr:write([[
ERROR: GraphLite Lua SDK not found.

The high-level Lua SDK lives on the 'luajit-sdk' branch.
Please set it up using ONE of the following methods:

  Option A - set the environment variable:
    export GRAPHLITE_LUA_SDK=/path/to/graphlite/lua-sdk

  Option B - checkout the branch in the expected location:
    cd ~/github/simbo1905
    git clone https://github.com/simbo1905/graphlite.git graphlite  # if needed
    cd graphlite
    git checkout luajit-sdk
    cd lua-sdk && ./setup.sh   # installs dkjson via luarocks

  The examples expect the SDK at:
    ~/github/simbo1905/graphlite/lua-sdk/

]])
  os.exit(1)
end

-- Add the SDK root to package.path so require("src.connection") works
package.path = sdk_path .. "/?.lua;" .. sdk_path .. "/?/init.lua;" .. package.path

------------------------------------------------------------------------
-- Imports
------------------------------------------------------------------------
local GraphLite = require("src.connection").GraphLite
local errors    = require("src.errors")

------------------------------------------------------------------------
-- Helper: print a query result as a simple table
------------------------------------------------------------------------
local function print_result(result)
  for _, row in ipairs(result.rows) do
    local parts = {}
    for _, v in ipairs(result.variables) do
      parts[#parts + 1] = v .. "=" .. tostring(row[v] or "nil")
    end
    print("     " .. table.concat(parts, ", "))
  end
end

------------------------------------------------------------------------
-- Main
------------------------------------------------------------------------
local function main()
  print("=== GraphLite High-Level Lua SDK Drug Discovery Example ===\n")

  local db_path = "./drug_discovery_lua_sdk_db"

  -- Clean up old database
  os.execute("rm -rf " .. db_path .. " 2>/dev/null")

  local ok, err = xpcall(function()

    -- Step 1: Open database
    print("1. Opening database...")
    local db = GraphLite.open(db_path)
    print("   Database opened\n")

    -- Step 2: Create session
    print("2. Creating session...")
    local session = db:session("researcher")
    print("   Session created\n")

    -- Step 3: Setup schema and graph
    print("3. Setting up schema and graph...")
    session:execute("CREATE SCHEMA IF NOT EXISTS /drug_discovery")
    session:execute("SESSION SET SCHEMA /drug_discovery")
    session:execute("CREATE GRAPH IF NOT EXISTS pharma_research")
    session:execute("SESSION SET GRAPH pharma_research")
    print("   Schema and graph configured\n")

    -- Step 4: Insert data
    print("4. Inserting pharmaceutical data...")

    print("   -> Inserting target proteins...")
    session:execute([[INSERT
      (:Protein {
          id: 'TP53',
          name: 'Tumor Protein P53',
          disease: 'Cancer',
          function: 'Tumor suppressor',
          gene_location: '17p13.1'
      }),
      (:Protein {
          id: 'EGFR',
          name: 'Epidermal Growth Factor Receptor',
          disease: 'Cancer',
          function: 'Cell growth and division',
          gene_location: '7p11.2'
      }),
      (:Protein {
          id: 'ACE2',
          name: 'Angiotensin-Converting Enzyme 2',
          disease: 'Hypertension',
          function: 'Blood pressure regulation',
          gene_location: 'Xp22.2'
      }),
      (:Protein {
          id: 'BACE1',
          name: 'Beta-Secretase 1',
          disease: 'Alzheimers',
          function: 'Amyloid beta production',
          gene_location: '11q23.3'
      })]])

    print("   -> Inserting drug compounds...")
    session:execute([[INSERT
      (:Compound {
          id: 'CP-002',
          name: 'Gefitinib',
          molecular_formula: 'C22H24ClFN4O3',
          molecular_weight: 446.902,
          drug_type: 'EGFR inhibitor',
          development_stage: 'Approved'
      }),
      (:Compound {
          id: 'CP-003',
          name: 'Captopril',
          molecular_formula: 'C9H15NO3S',
          molecular_weight: 217.285,
          drug_type: 'ACE inhibitor',
          development_stage: 'Approved'
      }),
      (:Compound {
          id: 'CP-004',
          name: 'LY2811376',
          molecular_formula: 'C18H17F3N2O3',
          molecular_weight: 366.33,
          drug_type: 'BACE1 inhibitor',
          development_stage: 'Clinical Trial Phase 1'
      }),
      (:Compound {
          id: 'CP-005',
          name: 'APG-115',
          molecular_formula: 'C31H37N5O4',
          molecular_weight: 543.66,
          drug_type: 'MDM2-p53 inhibitor',
          development_stage: 'Clinical Trial Phase 2'
      })]])

    print("   -> Inserting experimental assays...")
    session:execute([[INSERT
      (:Assay {
          id: 'AS-001',
          name: 'EGFR Kinase Inhibition Assay',
          assay_type: 'Enzymatic',
          method: 'TR-FRET',
          date: '2024-01-15'
      }),
      (:Assay {
          id: 'AS-002',
          name: 'ACE2 Binding Assay',
          assay_type: 'Binding',
          method: 'SPR',
          date: '2024-02-20'
      }),
      (:Assay {
          id: 'AS-003',
          name: 'BACE1 Activity Assay',
          assay_type: 'Enzymatic',
          method: 'FRET',
          date: '2024-03-10'
      }),
      (:Assay {
          id: 'AS-004',
          name: 'p53-MDM2 Disruption Assay',
          assay_type: 'Protein-Protein Interaction',
          method: 'HTRF',
          date: '2024-03-25'
      })]])

    print("   Core data inserted\n")

    -- Step 5: Create relationships
    print("5. Creating relationships...")

    print("   -> Linking compounds to assays...")
    session:execute([[MATCH (c:Compound {id: 'CP-002'}), (a:Assay {id: 'AS-001'})
       INSERT (c)-[:TESTED_IN {
           test_date: '2024-01-15',
           concentration_range: '0.1-1000 nM',
           replicate_count: 3
       }]->(a)]])

    session:execute([[MATCH (c:Compound {id: 'CP-003'}), (a:Assay {id: 'AS-002'})
       INSERT (c)-[:TESTED_IN {
           test_date: '2024-02-20',
           concentration_range: '1-10000 nM',
           replicate_count: 4
       }]->(a)]])

    session:execute([[MATCH (c:Compound {id: 'CP-004'}), (a:Assay {id: 'AS-003'})
       INSERT (c)-[:TESTED_IN {
           test_date: '2024-03-10',
           concentration_range: '0.5-500 nM',
           replicate_count: 3
       }]->(a)]])

    session:execute([[MATCH (c:Compound {id: 'CP-005'}), (a:Assay {id: 'AS-004'})
       INSERT (c)-[:TESTED_IN {
           test_date: '2024-03-25',
           concentration_range: '1-1000 nM',
           replicate_count: 5
       }]->(a)]])

    print("   -> Linking assays to proteins...")
    session:execute([[MATCH (a:Assay {id: 'AS-001'}), (p:Protein {id: 'EGFR'})
       INSERT (a)-[:MEASURES_ACTIVITY_ON {
           readout: 'Kinase inhibition',
           units: 'percent inhibition'
       }]->(p)]])

    session:execute([[MATCH (a:Assay {id: 'AS-002'}), (p:Protein {id: 'ACE2'})
       INSERT (a)-[:MEASURES_ACTIVITY_ON {
           readout: 'Binding affinity',
           units: 'KD (nM)'
       }]->(p)]])

    session:execute([[MATCH (a:Assay {id: 'AS-003'}), (p:Protein {id: 'BACE1'})
       INSERT (a)-[:MEASURES_ACTIVITY_ON {
           readout: 'Enzymatic activity',
           units: 'percent inhibition'
       }]->(p)]])

    session:execute([[MATCH (a:Assay {id: 'AS-004'}), (p:Protein {id: 'TP53'})
       INSERT (a)-[:MEASURES_ACTIVITY_ON {
           readout: 'PPI disruption',
           units: 'IC50 (nM)'
       }]->(p)]])

    print("   -> Creating inhibition relationships with IC50 data...")
    session:execute([[MATCH (c:Compound {id: 'CP-002'}), (p:Protein {id: 'EGFR'})
       INSERT (c)-[:INHIBITS {
           IC50: 37.5,
           IC50_unit: 'nM',
           Ki: 12.3,
           selectivity_index: 25.6,
           measurement_date: '2024-01-15'
       }]->(p)]])

    session:execute([[MATCH (c:Compound {id: 'CP-003'}), (p:Protein {id: 'ACE2'})
       INSERT (c)-[:INHIBITS {
           IC50: 23.0,
           IC50_unit: 'nM',
           Ki: 7.8,
           selectivity_index: 15.2,
           measurement_date: '2024-02-20'
       }]->(p)]])

    session:execute([[MATCH (c:Compound {id: 'CP-004'}), (p:Protein {id: 'BACE1'})
       INSERT (c)-[:INHIBITS {
           IC50: 85.0,
           IC50_unit: 'nM',
           Ki: 28.5,
           selectivity_index: 45.1,
           measurement_date: '2024-03-10'
       }]->(p)]])

    session:execute([[MATCH (c:Compound {id: 'CP-005'}), (p:Protein {id: 'TP53'})
       INSERT (c)-[:INHIBITS {
           IC50: 12.5,
           IC50_unit: 'nM',
           Ki: 3.2,
           selectivity_index: 120.5,
           measurement_date: '2024-03-25'
       }]->(p)]])

    print("   Relationships created\n")

    -- Step 6: Analytical queries
    print("6. Running analytical queries...\n")

    -- Query 1: IC50 filtering
    print("   Query 1: Compounds targeting TP53 with IC50 < 100 nM")
    local result = session:query([[
      MATCH (c:Compound)-[i:INHIBITS]->(p:Protein {id: 'TP53'})
      WHERE i.IC50 < 100
      RETURN c.name, c.id, i.IC50, i.IC50_unit, i.Ki
      ORDER BY i.IC50]])
    print("   Results:")
    print_result(result)
    print()

    -- Query 2: Traversal — complete testing pathway
    print("   Query 2: Complete testing pathway for Gefitinib")
    result = session:query([[
      MATCH (c:Compound {id: 'CP-002'})-[t:TESTED_IN]->(a:Assay)-[m:MEASURES_ACTIVITY_ON]->(p:Protein)
      RETURN c.name, a.name, a.assay_type, p.name, p.disease]])
    print("   Results:")
    print_result(result)
    print()

    -- Query 3: All interactions sorted by potency
    print("   Query 3: All compound-target interactions sorted by potency")
    result = session:query([[
      MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
      RETURN c.name AS Compound,
             p.name AS Target,
             p.disease AS Disease,
             i.IC50 AS IC50_nM,
             c.development_stage AS Stage
      ORDER BY i.IC50]])
    print("   Columns: " .. table.concat(result.variables, ", "))
    print("   Results:")
    print_result(result)
    print()

    -- Query 4: Clinical trial compounds
    print("   Query 4: Clinical trial compounds and their targets")
    result = session:query([[
      MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
      WHERE c.development_stage LIKE '%Clinical Trial%'
      RETURN c.name AS Compound,
             c.development_stage AS Stage,
             p.name AS Target,
             i.IC50 AS Potency_nM,
             i.selectivity_index AS Selectivity]])
    print("   Results:")
    print_result(result)
    print()

    -- Query 5: Aggregation — proteins with compounds
    print("   Query 5: Proteins with multiple targeting compounds")
    result = session:query([[
      MATCH (p:Protein)<-[:INHIBITS]-(c:Compound)
      RETURN p.name AS Protein,
             p.disease AS Disease,
             COUNT(c) AS CompoundCount]])
    print("   Results:")
    print_result(result)
    print()

    -- Summary
    print("=== Drug Discovery Example Complete ===")
    print("\nKey Insights:")
    print("  - Modeled 3 node types: Protein, Compound, Assay")
    print("  - Created relationship types: TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS")
    print("  - Demonstrated graph traversals for drug discovery workflows")
    print("  - Showed IC50-based compound filtering and ranking")
    print("  - Used High-Level Lua SDK features:")
    print("    - Session-centric API (session:query() vs manual session ID)")
    print("    - Typed errors (ConnectionError, SessionError, QueryError)")
    print("    - JSON parsing via dkjson (luarocks)")
    print("    - Clean, idiomatic Lua interface")
    print("\nDatabase location: " .. db_path .. "/")
    print("To clean up: rm -rf " .. db_path .. "/")

    session:close()
    db:close()

  end, function(e)
    return tostring(e) .. "\n" .. debug.traceback()
  end)

  if not ok then
    io.stderr:write("\nError: " .. tostring(err) .. "\n")
    os.exit(1)
  end
end

main()
