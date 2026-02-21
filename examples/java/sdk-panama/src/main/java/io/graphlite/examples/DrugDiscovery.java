package io.graphlite.examples;

import io.graphlite.sdk.*;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;

/**
 * Drug Discovery Example using GraphLite High-Level Java SDK (Panama/FFM).
 *
 * Mirrors the Python SDK drug discovery demo:
 * - Schema + graph setup
 * - Insert targets, compounds, assays; relationships TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS
 * - Analytical queries: filtering, traversal, aggregation
 */
public class DrugDiscovery {

    public static void main(String[] args) {
        System.out.println("=== GraphLite High-Level SDK Drug Discovery Example ===\n");

        Path dbPath = Path.of("./drug_discovery_highlevel_sdk_db");
        try {
            if (Files.exists(dbPath)) {
                deleteRecursively(dbPath);
            }

            runDemo(dbPath.toString());
            System.out.println("\n=== Drug Discovery Example Complete ===");

        } catch (ConnectionException e) {
            System.err.println("\nConnection Error: " + e.getMessage());
            System.exit(1);
        } catch (SessionException e) {
            System.err.println("\nSession Error: " + e.getMessage());
            System.exit(1);
        } catch (QueryException e) {
            System.err.println("\nQuery Error: " + e.getMessage());
            System.exit(1);
        } catch (GraphLiteException e) {
            System.err.println("\nGraphLite Error: " + e.getMessage());
            System.exit(1);
        } catch (Exception e) {
            System.err.println("\nUnexpected error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    private static void runDemo(String dbPath) throws Exception {
        try (GraphLite db = GraphLite.open(dbPath)) {
            System.out.println("1. Database opened\n");

            try (Session session = db.session("researcher")) {
                System.out.println("2. Session created\n");

                System.out.println("3. Setting up schema and graph...");
                session.execute("CREATE SCHEMA IF NOT EXISTS /drug_discovery");
                session.execute("SESSION SET SCHEMA /drug_discovery");
                session.execute("CREATE GRAPH IF NOT EXISTS pharma_research");
                session.execute("SESSION SET GRAPH pharma_research");
                System.out.println("   Schema and graph configured\n");

                System.out.println("4. Inserting pharmaceutical data...");
                insertData(session);
                System.out.println("   Data inserted\n");

                System.out.println("5. Creating relationships...");
                createRelationships(session);
                System.out.println("   Relationships created\n");

                System.out.println("6. Running analytical queries...\n");
                runQueries(session);
            }
        }
    }

    private static void insertData(Session session) {
        session.execute("""
            INSERT
            (:Protein {id: 'TP53', name: 'Tumor Protein P53', disease: 'Cancer', function: 'Tumor suppressor', gene_location: '17p13.1'}),
            (:Protein {id: 'EGFR', name: 'Epidermal Growth Factor Receptor', disease: 'Cancer', function: 'Cell growth and division', gene_location: '7p11.2'}),
            (:Protein {id: 'ACE2', name: 'Angiotensin-Converting Enzyme 2', disease: 'Hypertension', function: 'Blood pressure regulation', gene_location: 'Xp22.2'}),
            (:Protein {id: 'BACE1', name: 'Beta-Secretase 1', disease: 'Alzheimers', function: 'Amyloid beta production', gene_location: '11q23.3'})
            """);

        session.execute("""
            INSERT
            (:Compound {id: 'CP-002', name: 'Gefitinib', molecular_formula: 'C22H24ClFN4O3', molecular_weight: 446.902, drug_type: 'EGFR inhibitor', development_stage: 'Approved'}),
            (:Compound {id: 'CP-003', name: 'Captopril', molecular_formula: 'C9H15NO3S', molecular_weight: 217.285, drug_type: 'ACE inhibitor', development_stage: 'Approved'}),
            (:Compound {id: 'CP-004', name: 'LY2811376', molecular_formula: 'C18H17F3N2O3', molecular_weight: 366.33, drug_type: 'BACE1 inhibitor', development_stage: 'Clinical Trial Phase 1'}),
            (:Compound {id: 'CP-005', name: 'APG-115', molecular_formula: 'C31H37N5O4', molecular_weight: 543.66, drug_type: 'MDM2-p53 inhibitor', development_stage: 'Clinical Trial Phase 2'})
            """);

        session.execute("""
            INSERT
            (:Assay {id: 'AS-001', name: 'EGFR Kinase Inhibition Assay', assay_type: 'Enzymatic', method: 'TR-FRET', date: '2024-01-15'}),
            (:Assay {id: 'AS-002', name: 'ACE2 Binding Assay', assay_type: 'Binding', method: 'SPR', date: '2024-02-20'}),
            (:Assay {id: 'AS-003', name: 'BACE1 Activity Assay', assay_type: 'Enzymatic', method: 'FRET', date: '2024-03-10'}),
            (:Assay {id: 'AS-004', name: 'p53-MDM2 Disruption Assay', assay_type: 'Protein-Protein Interaction', method: 'HTRF', date: '2024-03-25'})
            """);
    }

    private static void createRelationships(Session session) {
        session.execute("""
            MATCH (c:Compound {id: 'CP-002'}), (a:Assay {id: 'AS-001'})
            INSERT (c)-[:TESTED_IN {test_date: '2024-01-15', concentration_range: '0.1-1000 nM', replicate_count: 3}]->(a)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-003'}), (a:Assay {id: 'AS-002'})
            INSERT (c)-[:TESTED_IN {test_date: '2024-02-20', concentration_range: '1-10000 nM', replicate_count: 4}]->(a)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-004'}), (a:Assay {id: 'AS-003'})
            INSERT (c)-[:TESTED_IN {test_date: '2024-03-10', concentration_range: '0.5-500 nM', replicate_count: 3}]->(a)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-005'}), (a:Assay {id: 'AS-004'})
            INSERT (c)-[:TESTED_IN {test_date: '2024-03-25', concentration_range: '1-1000 nM', replicate_count: 5}]->(a)
            """);

        session.execute("""
            MATCH (a:Assay {id: 'AS-001'}), (p:Protein {id: 'EGFR'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'Kinase inhibition', units: 'percent inhibition'}]->(p)
            """);
        session.execute("""
            MATCH (a:Assay {id: 'AS-002'}), (p:Protein {id: 'ACE2'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'Binding affinity', units: 'KD (nM)'}]->(p)
            """);
        session.execute("""
            MATCH (a:Assay {id: 'AS-003'}), (p:Protein {id: 'BACE1'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'Enzymatic activity', units: 'percent inhibition'}]->(p)
            """);
        session.execute("""
            MATCH (a:Assay {id: 'AS-004'}), (p:Protein {id: 'TP53'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {readout: 'PPI disruption', units: 'IC50 (nM)'}]->(p)
            """);

        session.execute("""
            MATCH (c:Compound {id: 'CP-002'}), (p:Protein {id: 'EGFR'})
            INSERT (c)-[:INHIBITS {IC50: 37.5, IC50_unit: 'nM', Ki: 12.3, selectivity_index: 25.6, measurement_date: '2024-01-15'}]->(p)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-003'}), (p:Protein {id: 'ACE2'})
            INSERT (c)-[:INHIBITS {IC50: 23.0, IC50_unit: 'nM', Ki: 7.8, selectivity_index: 15.2, measurement_date: '2024-02-20'}]->(p)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-004'}), (p:Protein {id: 'BACE1'})
            INSERT (c)-[:INHIBITS {IC50: 85.0, IC50_unit: 'nM', Ki: 28.5, selectivity_index: 45.1, measurement_date: '2024-03-10'}]->(p)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-005'}), (p:Protein {id: 'TP53'})
            INSERT (c)-[:INHIBITS {IC50: 12.5, IC50_unit: 'nM', Ki: 3.2, selectivity_index: 120.5, measurement_date: '2024-03-25'}]->(p)
            """);
    }

    private static void runQueries(Session session) {
        System.out.println("   Query 1: Compounds targeting TP53 with IC50 < 100 nM");
        QueryResult r1 = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein {id: 'TP53'})
            WHERE i.IC50 < 100
            RETURN c.name, c.id, i.IC50, i.IC50_unit, i.Ki
            ORDER BY i.IC50
            """);
        for (Map<String, Object> row : r1.getRows()) {
            System.out.println("     - " + row);
        }
        System.out.println();

        System.out.println("   Query 2: Complete testing pathway for Gefitinib");
        QueryResult r2 = session.query("""
            MATCH (c:Compound {id: 'CP-002'})-[t:TESTED_IN]->(a:Assay)-[m:MEASURES_ACTIVITY_ON]->(p:Protein)
            RETURN c.name, a.name, a.assay_type, p.name, p.disease
            """);
        for (Map<String, Object> row : r2.getRows()) {
            System.out.println("     " + row);
        }
        System.out.println();

        System.out.println("   Query 3: All compound-target interactions sorted by potency");
        QueryResult r3 = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
            RETURN c.name AS Compound, p.name AS Target, p.disease AS Disease, i.IC50 AS IC50_nM, c.development_stage AS Stage
            ORDER BY i.IC50
            """);
        System.out.println("   Columns: " + r3.getVariables());
        for (Map<String, Object> row : r3.getRows()) {
            System.out.println("     " + row);
        }
        System.out.println();

        System.out.println("   Query 4: Clinical trial compounds and their targets");
        QueryResult r4 = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
            WHERE c.development_stage LIKE '%Clinical Trial%'
            RETURN c.name AS Compound, c.development_stage AS Stage, p.name AS Target, i.IC50 AS Potency_nM, i.selectivity_index AS Selectivity
            """);
        for (Map<String, Object> row : r4.getRows()) {
            System.out.println("     " + row);
        }
        System.out.println();

        System.out.println("   Query 5: Proteins with multiple targeting compounds");
        QueryResult r5 = session.query("""
            MATCH (p:Protein)<-[:INHIBITS]-(c:Compound)
            RETURN p.name AS Protein, p.disease AS Disease, COUNT(c) AS CompoundCount
            """);
        for (Map<String, Object> row : r5.getRows()) {
            System.out.println("     " + row);
        }
    }

    private static void deleteRecursively(Path path) throws java.io.IOException {
        if (Files.isDirectory(path)) {
            try (var stream = Files.list(path)) {
                stream.forEach(p -> {
                    try {
                        deleteRecursively(p);
                    } catch (java.io.IOException e) {
                        throw new RuntimeException(e);
                    }
                });
            }
        }
        Files.delete(path);
    }
}
