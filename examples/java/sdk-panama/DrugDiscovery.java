import io.graphlite.sdk.GraphLite;
import io.graphlite.sdk.Session;
import io.graphlite.sdk.QueryResult;
import io.graphlite.sdk.Errors.*;

import java.io.IOException;
import java.nio.file.*;
import java.util.Comparator;
import java.util.Map;

/**
 * Drug Discovery Example using GraphLite High-Level Java SDK (Panama FFM).
 * <p>
 * Mirrors the Python SDK drug discovery demo, demonstrating:
 * <ul>
 *   <li>Session-centric API ({@code session.query()} instead of {@code db.query(sessionId, ...)})</li>
 *   <li>Typed exceptions ({@link ConnectionException}, {@link SessionException}, {@link QueryException})</li>
 *   <li>AutoCloseable resource management (try-with-resources)</li>
 * </ul>
 *
 * Domain Model:
 * <pre>
 * Compound -> TESTED_IN -> Assay -> MEASURES_ACTIVITY_ON -> Target (Protein)
 * Compound -> INHIBITS -> Target (with IC50 measurements)
 * </pre>
 *
 * Run with:
 * <pre>
 * mvn compile exec:java -Dexec.mainClass="DrugDiscovery"
 * </pre>
 */
public class DrugDiscovery {

    public static void main(String[] args) {
        System.out.println("=== GraphLite Java SDK (Panama FFM) Drug Discovery Example ===\n");

        Path dbPath = Paths.get("drug_discovery_panama_db");

        try {
            deleteIfExists(dbPath);

            // 1. Open database
            System.out.println("1. Opening database...");
            try (GraphLite db = GraphLite.open(dbPath.toString())) {
                System.out.println("   [OK] Database opened (version: " + GraphLite.version() + ")\n");

                // 2. Create session
                System.out.println("2. Creating session...");
                try (Session session = db.session("researcher")) {
                    System.out.println("   [OK] Session created for '" + session.username() + "'\n");

                    setupSchemaAndGraph(session);
                    insertData(session);
                    createRelationships(session);
                    runAnalyticalQueries(session);

                    System.out.println("=== Drug Discovery Example Complete ===");
                    System.out.println("\nKey Insights:");
                    System.out.println("  * Modeled 3 node types: Protein, Compound, Assay");
                    System.out.println("  * Created relationship types: TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS");
                    System.out.println("  * Demonstrated graph traversals for drug discovery workflows");
                    System.out.println("  * Showed IC50-based compound filtering and ranking");
                    System.out.println("  * Used High-Level SDK features:");
                    System.out.println("    - Panama FFM API (no JNI)");
                    System.out.println("    - Session-centric API (session.query() vs db.query(sessionId, ...))");
                    System.out.println("    - Typed exceptions (ConnectionException, SessionException, QueryException)");
                    System.out.println("    - AutoCloseable resources (try-with-resources)");
                }
            }
        } catch (ConnectionException e) {
            System.err.println("\n[ERROR] Connection Error: " + e.getMessage());
            System.exit(1);
        } catch (SessionException e) {
            System.err.println("\n[ERROR] Session Error: " + e.getMessage());
            System.exit(1);
        } catch (QueryException e) {
            System.err.println("\n[ERROR] Query Error: " + e.getMessage());
            System.exit(1);
        } catch (GraphLiteException e) {
            System.err.println("\n[ERROR] GraphLite Error (" + e.errorCode() + "): " + e.getMessage());
            System.exit(1);
        } catch (Exception e) {
            System.err.println("\n[ERROR] Unexpected error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        } finally {
            try { deleteIfExists(dbPath); } catch (IOException ignored) {}
        }
    }

    // -----------------------------------------------------------------------

    private static void setupSchemaAndGraph(Session session) {
        System.out.println("3. Setting up schema and graph...");
        session.execute("CREATE SCHEMA IF NOT EXISTS /drug_discovery");
        session.execute("SESSION SET SCHEMA /drug_discovery");
        session.execute("CREATE GRAPH IF NOT EXISTS pharma_research");
        session.execute("SESSION SET GRAPH pharma_research");
        System.out.println("   [OK] Schema and graph configured\n");
    }

    private static void insertData(Session session) {
        System.out.println("4. Inserting pharmaceutical data...");

        System.out.println("   -> Inserting target proteins...");
        session.execute("""
            INSERT
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
                })""");

        System.out.println("   -> Inserting drug compounds...");
        session.execute("""
            INSERT
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
                })""");

        System.out.println("   -> Inserting experimental assays...");
        session.execute("""
            INSERT
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
                })""");

        System.out.println("   [OK] Core data inserted\n");
    }

    private static void createRelationships(Session session) {
        System.out.println("5. Creating relationships...");

        System.out.println("   -> Linking compounds to assays...");
        session.execute("""
            MATCH (c:Compound {id: 'CP-002'}), (a:Assay {id: 'AS-001'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-01-15',
                concentration_range: '0.1-1000 nM',
                replicate_count: 3
            }]->(a)""");
        session.execute("""
            MATCH (c:Compound {id: 'CP-003'}), (a:Assay {id: 'AS-002'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-02-20',
                concentration_range: '1-10000 nM',
                replicate_count: 4
            }]->(a)""");
        session.execute("""
            MATCH (c:Compound {id: 'CP-004'}), (a:Assay {id: 'AS-003'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-03-10',
                concentration_range: '0.5-500 nM',
                replicate_count: 3
            }]->(a)""");
        session.execute("""
            MATCH (c:Compound {id: 'CP-005'}), (a:Assay {id: 'AS-004'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-03-25',
                concentration_range: '1-1000 nM',
                replicate_count: 5
            }]->(a)""");

        System.out.println("   -> Linking assays to proteins...");
        session.execute("""
            MATCH (a:Assay {id: 'AS-001'}), (p:Protein {id: 'EGFR'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'Kinase inhibition', units: 'percent inhibition'
            }]->(p)""");
        session.execute("""
            MATCH (a:Assay {id: 'AS-002'}), (p:Protein {id: 'ACE2'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'Binding affinity', units: 'KD (nM)'
            }]->(p)""");
        session.execute("""
            MATCH (a:Assay {id: 'AS-003'}), (p:Protein {id: 'BACE1'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'Enzymatic activity', units: 'percent inhibition'
            }]->(p)""");
        session.execute("""
            MATCH (a:Assay {id: 'AS-004'}), (p:Protein {id: 'TP53'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'PPI disruption', units: 'IC50 (nM)'
            }]->(p)""");

        System.out.println("   -> Creating inhibition relationships with IC50 data...");
        session.execute("""
            MATCH (c:Compound {id: 'CP-002'}), (p:Protein {id: 'EGFR'})
            INSERT (c)-[:INHIBITS {
                IC50: 37.5, IC50_unit: 'nM', Ki: 12.3,
                selectivity_index: 25.6, measurement_date: '2024-01-15'
            }]->(p)""");
        session.execute("""
            MATCH (c:Compound {id: 'CP-003'}), (p:Protein {id: 'ACE2'})
            INSERT (c)-[:INHIBITS {
                IC50: 23.0, IC50_unit: 'nM', Ki: 7.8,
                selectivity_index: 15.2, measurement_date: '2024-02-20'
            }]->(p)""");
        session.execute("""
            MATCH (c:Compound {id: 'CP-004'}), (p:Protein {id: 'BACE1'})
            INSERT (c)-[:INHIBITS {
                IC50: 85.0, IC50_unit: 'nM', Ki: 28.5,
                selectivity_index: 45.1, measurement_date: '2024-03-10'
            }]->(p)""");
        session.execute("""
            MATCH (c:Compound {id: 'CP-005'}), (p:Protein {id: 'TP53'})
            INSERT (c)-[:INHIBITS {
                IC50: 12.5, IC50_unit: 'nM', Ki: 3.2,
                selectivity_index: 120.5, measurement_date: '2024-03-25'
            }]->(p)""");

        System.out.println("   [OK] Relationships created\n");
    }

    private static void runAnalyticalQueries(Session session) {
        System.out.println("6. Running analytical queries...\n");

        // Query 1: Compounds targeting TP53
        System.out.println("   Query 1: Compounds targeting TP53 with IC50 < 100 nM");
        QueryResult result = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein {id: 'TP53'})
            WHERE i.IC50 < 100
            RETURN c.name, c.id, i.IC50, i.IC50_unit, i.Ki
            ORDER BY i.IC50""");
        printResult(result);

        // Query 2: Complete testing pathway for Gefitinib
        System.out.println("   Query 2: Complete testing pathway for Gefitinib");
        result = session.query("""
            MATCH (c:Compound {id: 'CP-002'})-[t:TESTED_IN]->(a:Assay)-[m:MEASURES_ACTIVITY_ON]->(p:Protein)
            RETURN c.name, a.name, a.assay_type, p.name, p.disease""");
        printResult(result);

        // Query 3: All interactions sorted by potency
        System.out.println("   Query 3: All compound-target interactions sorted by potency");
        result = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
            RETURN c.name AS Compound,
                   p.name AS Target,
                   p.disease AS Disease,
                   i.IC50 AS IC50_nM,
                   c.development_stage AS Stage
            ORDER BY i.IC50""");
        System.out.println("   Columns: " + result.variables());
        printResult(result);

        // Query 4: Clinical trial compounds
        System.out.println("   Query 4: Clinical trial compounds and their targets");
        result = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
            WHERE c.development_stage LIKE '%Clinical Trial%'
            RETURN c.name AS Compound,
                   c.development_stage AS Stage,
                   p.name AS Target,
                   i.IC50 AS Potency_nM,
                   i.selectivity_index AS Selectivity""");
        printResult(result);

        // Query 5: Aggregation
        System.out.println("   Query 5: Proteins with multiple targeting compounds");
        result = session.query("""
            MATCH (p:Protein)<-[:INHIBITS]-(c:Compound)
            RETURN p.name AS Protein,
                   p.disease AS Disease,
                   COUNT(c) AS CompoundCount""");
        printResult(result);
    }

    private static void printResult(QueryResult result) {
        System.out.println("   Results:");
        for (Map<String, Object> row : result.rows()) {
            System.out.println("     " + row);
        }
        System.out.println();
    }

    private static void deleteIfExists(Path dir) throws IOException {
        if (!Files.exists(dir)) return;
        try (var walk = Files.walk(dir)) {
            walk.sorted(Comparator.reverseOrder()).forEach(p -> {
                try { Files.deleteIfExists(p); } catch (IOException ignored) {}
            });
        }
    }
}
