#!/usr/bin/env luajit
--[[
  Drug Discovery Example using GraphLite High-Level LuaJIT SDK

  Domain Model:
  - Targets: Proteins (e.g., TP53 in cancer)
  - Compounds: Small molecules
  - Assays: Experiments measuring compound-target activity

  Graph Structure:
  Compound -> TESTED_IN -> Assay -> MEASURES_ACTIVITY_ON -> Target
  Compound -> INHIBITS -> Target (with IC50)

  Run: luajit drug_discovery.lua
]]

-- Add script directory to package.path for bootstrap
local src = debug.getinfo(1, "S").source
if src:sub(1, 1) == "@" then src = src:sub(2) end
local script_dir = src:match("(.+)/[^/]+$") or "."
package.path = script_dir .. "/?.lua;" .. package.path

require("bootstrap")

local connection = require("src.connection")
local GraphLite = connection.GraphLite
local errors = require("src.errors")

local ConnectionError = errors.ConnectionError
local SessionError = errors.SessionError
local QueryError = errors.QueryError
local GraphLiteError = errors.GraphLiteError

local function main()
  print("=== GraphLite High-Level SDK Drug Discovery Example ===\n")

  local db_path = "./drug_discovery_highlevel_sdk_db"

  -- Clean up old database
  os.execute("rm -rf " .. db_path)

  print("1. Opening database...")
  local db = GraphLite.open(db_path)
  print("   ✓ Database opened\n")

  print("2. Creating session...")
  local session = db:session("researcher")
  print("   ✓ Session created\n")

  print("3. Setting up schema and graph...")
  session:execute("CREATE SCHEMA IF NOT EXISTS /drug_discovery")
  session:execute("SESSION SET SCHEMA /drug_discovery")
  session:execute("CREATE GRAPH IF NOT EXISTS pharma_research")
  session:execute("SESSION SET GRAPH pharma_research")
  print("   ✓ Schema and graph configured\n")

  print("4. Inserting pharmaceutical data...")

  print("   → Inserting target proteins...")
  session:execute([[
    INSERT
      (:Protein {id: 'TP53', name: 'Tumor Protein P53', disease: 'Cancer', function: 'Tumor suppressor', gene_location: '17p13.1'}),
      (:Protein {id: 'EGFR', name: 'Epidermal Growth Factor Receptor', disease: 'Cancer', function: 'Cell growth and division', gene_location: '7p11.2'}),
      (:Protein {id: 'ACE2', name: 'Angiotensin-Converting Enzyme 2', disease: 'Hypertension', function: 'Blood pressure regulation', gene_location: 'Xp22.2'}),
      (:Protein {id: 'BACE1', name: 'Beta-Secretase 1', disease: 'Alzheimers', function: 'Amyloid beta production', gene_location: '11q23.3'})
  ]])

  print("   → Inserting drug compounds...")
  session:execute([[
    INSERT
      (:Compound {id: 'CP-002', name: 'Gefitinib', molecular_formula: 'C22H24ClFN4O3', molecular_weight: 446.902, drug_type: 'EGFR inhibitor', development_stage: 'Approved'}),
      (:Compound {id: 'CP-003', name: 'Captopril', molecular_formula: 'C9H15NO3S', molecular_weight: 217.285, drug_type: 'ACE inhibitor', development_stage: 'Approved'}),
      (:Compound {id: 'CP-004', name: 'LY2811376', molecular_formula: 'C18H17F3N2O3', molecular_weight: 366.33, drug_type: 'BACE1 inhibitor', development_stage: 'Clinical Trial Phase 1'}),
      (:Compound {id: 'CP-005', name: 'APG-115', molecular_formula: 'C31H37N5O4', molecular_weight: 543.66, drug_type: 'MDM2-p53 inhibitor', development_stage: 'Clinical Trial Phase 2'})
  ]])

  print("   → Inserting experimental assays...")
  session:execute([[
    INSERT
      (:Assay {id: 'AS-001', name: 'EGFR Kinase Inhibition Assay', assay_type: 'Enzymatic', method: 'TR-FRET', date: '2024-01-15'}),
      (:Assay {id: 'AS-002', name: 'ACE2 Binding Assay', assay_type: 'Binding', method: 'SPR', date: '2024-02-20'}),
      (:Assay {id: 'AS-003', name: 'BACE1 Activity Assay', assay_type: 'Enzymatic', method: 'FRET', date: '2024-03-10'}),
      (:Assay {id: 'AS-004', name: 'p53-MDM2 Disruption Assay', assay_type: 'Protein-Protein Interaction', method: 'HTRF', date: '2024-03-25'})
  ]])

  print("   ✓ Core data inserted\n")

  print("5. Creating relationships...")

  print("   → Linking compounds to assays...")
  session:execute([[
    MATCH (c:Compound {id: 'CP-002'}), (a:Assay {id: 'AS-001'})
    INSERT (c)-[:TESTED_IN {test_date: '2024-01-15', concentration_range: '0.1-1000 nM', replicate_count: 3}]->(a)
  ]])
  session:execute([[
    MATCH (c:Compound {id: 'CP-003'}), (a:Assay {id: 'AS-002'})
    INSERT (c)-[:TESTED_IN {test_date: '2024-02-20', concentration_range: '1-10000 nM', replicate_count: 4}]->(a)
  ]])
  session:execute([[
    MATCH (c:Compound {id: 'CP-004'}), (a:Assay {id: 'AS-003'})
    INSERT (c)-[:TESTED_IN {test_date: '2024-03-10', concentration_range: '0.5-500 nM', replicate_count: 3}]->(a)
  ]])
  session:execute([[
    MATCH (c:Compound {id: 'CP-005'}), (a:Assay {id: 'AS-004'})
    INSERT (c)-[:TESTED_IN {test_date: '2024-03-25', concentration_range: '1-1000 nM', replicate_count: 5}]->(a)
  ]])

  print("   → Linking assays to proteins...")
  session:execute([[
    MATCH (a:Assay {id: 'AS-001'}), (p:Protein {id: 'EGFR'})
    INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'Kinase inhibition', units: 'percent inhibition'}]->(p)
  ]])
  session:execute([[
    MATCH (a:Assay {id: 'AS-002'}), (p:Protein {id: 'ACE2'})
    INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'Binding affinity', units: 'KD (nM)'}]->(p)
  ]])
  session:execute([[
    MATCH (a:Assay {id: 'AS-003'}), (p:Protein {id: 'BACE1'})
    INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'Enzymatic activity', units: 'percent inhibition'}]->(p)
  ]])
  session:execute([[
    MATCH (a:Assay {id: 'AS-004'}), (p:Protein {id: 'TP53'})
    INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'PPI disruption', units: 'IC50 (nM)'}]->(p)
  ]])

  print("   → Creating inhibition relationships with IC50 data...")
  session:execute([[
    MATCH (c:Compound {id: 'CP-002'}), (p:Protein {id: 'EGFR'})
    INSERT (c)-[:INHIBITS {IC50: 37.5, IC50_unit: 'nM', Ki: 12.3, selectivity_index: 25.6, measurement_date: '2024-01-15'}]->(p)
  ]])
  session:execute([[
    MATCH (c:Compound {id: 'CP-003'}), (p:Protein {id: 'ACE2'})
    INSERT (c)-[:INHIBITS {IC50: 23.0, IC50_unit: 'nM', Ki: 7.8, selectivity_index: 15.2, measurement_date: '2024-02-20'}]->(p)
  ]])
  session:execute([[
    MATCH (c:Compound {id: 'CP-004'}), (p:Protein {id: 'BACE1'})
    INSERT (c)-[:INHIBITS {IC50: 85.0, IC50_unit: 'nM', Ki: 28.5, selectivity_index: 45.1, measurement_date: '2024-03-10'}]->(p)
  ]])
  session:execute([[
    MATCH (c:Compound {id: 'CP-005'}), (p:Protein {id: 'TP53'})
    INSERT (c)-[:INHIBITS {IC50: 12.5, IC50_unit: 'nM', Ki: 3.2, selectivity_index: 120.5, measurement_date: '2024-03-25'}]->(p)
  ]])

  print("   ✓ Relationships created\n")

  print("6. Running analytical queries...\n")

  print("   Query 1: Compounds targeting TP53 with IC50 < 100 nM")
  local result = session:query([[
    MATCH (c:Compound)-[i:INHIBITS]->(p:Protein {id: 'TP53'})
    WHERE i.IC50 < 100
    RETURN c.name, c.id, i.IC50, i.IC50_unit, i.Ki
    ORDER BY i.IC50
  ]])
  print("   Results:")
  for _, row in ipairs(result.rows) do
    print("     -", row["c.name"], row["c.id"], row["i.IC50"], row["i.IC50_unit"], row["i.Ki"])
  end
  print()

  print("   Query 2: Complete testing pathway for Gefitinib")
  result = session:query([[
    MATCH (c:Compound {id: 'CP-002'})-[t:TESTED_IN]->(a:Assay)-[m:MEASURES_ACTIVITY_ON]->(p:Protein)
    RETURN c.name, a.name, a.assay_type, p.name, p.disease
  ]])
  print("   Results:")
  for _, row in ipairs(result.rows) do
    print("    ", row["c.name"], row["a.name"], row["a.assay_type"], row["p.name"], row["p.disease"])
  end
  print()

  print("   Query 3: All compound-target interactions sorted by potency")
  result = session:query([[
    MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
    RETURN c.name AS Compound, p.name AS Target, p.disease AS Disease, i.IC50 AS IC50_nM, c.development_stage AS Stage
    ORDER BY i.IC50
  ]])
  print("   Columns:", table.concat(result.variables, ", "))
  print("   Results:")
  for _, row in ipairs(result.rows) do
    print("    ", row.Compound, row.Target, row.Disease, row.IC50_nM, row.Stage)
  end
  print()

  print("   Query 4: Clinical trial compounds and their targets")
  result = session:query([[
    MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
    WHERE c.development_stage LIKE '%Clinical Trial%'
    RETURN c.name AS Compound, c.development_stage AS Stage, p.name AS Target, i.IC50 AS Potency_nM, i.selectivity_index AS Selectivity
  ]])
  print("   Results:")
  for _, row in ipairs(result.rows) do
    print("    ", row.Compound, row.Stage, row.Target, row.Potency_nM, row.Selectivity)
  end
  print()

  print("   Query 5: Proteins with multiple targeting compounds")
  result = session:query([[
    MATCH (p:Protein)<-[:INHIBITS]-(c:Compound)
    RETURN p.name AS Protein, p.disease AS Disease, COUNT(c) AS CompoundCount
  ]])
  print("   Results:")
  for _, row in ipairs(result.rows) do
    print("    ", row.Protein, row.Disease, row.CompoundCount)
  end
  print()

  print("=== Drug Discovery Example Complete ===")
  print("\nKey Insights:")
  print("  • Modeled 4 node types: Protein, Compound, Assay")
  print("  • Created relationship types: TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS")
  print("  • Demonstrated graph traversals for drug discovery workflows")
  print("  • Showed IC50-based compound filtering and ranking")
  print("  • Used High-Level SDK: session-centric API, typed errors")
  print("\nDatabase location: " .. db_path .. "/")
  print("To clean up: rm -rf " .. db_path .. "/")

  session:close()
  db:close()

  return 0
end

local ok, ret = pcall(main)
if not ok then
  local err = ret
  print("\n❌ Error:", type(err) == "table" and err.message or err)
  os.exit(1)
end
os.exit(ret or 0)
