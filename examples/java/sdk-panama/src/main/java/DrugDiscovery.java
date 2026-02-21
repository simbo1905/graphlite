import io.graphlite.sdk.Errors;
import io.graphlite.sdk.GraphLite;
import io.graphlite.sdk.QueryResult;
import io.graphlite.sdk.Session;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.Map;

/**
 * Drug discovery workflow demo using the high-level Java Panama SDK.
 */
public final class DrugDiscovery {
    private DrugDiscovery() {
    }

    public static void main(String[] args) {
        System.out.println("=== GraphLite Java 25 SDK (Panama) Drug Discovery Demo ===");
        Path dbPath = Path.of("./drug_discovery_panama_sdk_db").toAbsolutePath().normalize();

        try {
            deleteRecursively(dbPath);
            runDemo(dbPath);
            System.out.println();
            System.out.println("Demo completed. Database location: " + dbPath);
            System.out.println("Cleanup with: rm -rf " + dbPath);
        } catch (Errors.ConnectionException e) {
            System.err.println("Connection error: " + e.getMessage() + ", code=" + e.errorCode());
            System.exit(1);
        } catch (Errors.SessionException e) {
            System.err.println("Session error: " + e.getMessage() + ", code=" + e.errorCode());
            System.exit(1);
        } catch (Errors.QueryException e) {
            System.err.println("Query error: " + e.getMessage() + ", code=" + e.errorCode());
            System.exit(1);
        } catch (Errors.GraphLiteException e) {
            System.err.println("GraphLite error: " + e.getMessage() + ", code=" + e.errorCode());
            System.exit(1);
        } catch (Exception e) {
            System.err.println("Unexpected failure: " + e.getMessage());
            e.printStackTrace(System.err);
            System.exit(1);
        }
    }

    private static void runDemo(Path dbPath) {
        System.out.println("GraphLite core version: " + GraphLite.version());

        try (GraphLite db = GraphLite.open(dbPath.toString());
             Session session = db.session("researcher")) {
            System.out.println("\n1) Configuring schema and graph...");
            setupSchemaAndGraph(session);

            System.out.println("2) Inserting target, compound, and assay data...");
            insertCoreData(session);

            System.out.println("3) Creating experimental relationships...");
            createRelationships(session);

            System.out.println("4) Running analytical queries...\n");
            runAnalyticalQueries(session);
        }
    }

    private static void setupSchemaAndGraph(Session session) {
        session.execute("CREATE SCHEMA IF NOT EXISTS /drug_discovery");
        session.execute("SESSION SET SCHEMA /drug_discovery");
        session.execute("CREATE GRAPH IF NOT EXISTS pharma_research");
        session.execute("SESSION SET GRAPH pharma_research");
    }

    private static void insertCoreData(Session session) {
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
                })
            """);

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
                })
            """);

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
                })
            """);
    }

    private static void createRelationships(Session session) {
        session.execute("""
            MATCH (c:Compound {id: 'CP-002'}), (a:Assay {id: 'AS-001'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-01-15',
                concentration_range: '0.1-1000 nM',
                replicate_count: 3
            }]->(a)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-003'}), (a:Assay {id: 'AS-002'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-02-20',
                concentration_range: '1-10000 nM',
                replicate_count: 4
            }]->(a)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-004'}), (a:Assay {id: 'AS-003'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-03-10',
                concentration_range: '0.5-500 nM',
                replicate_count: 3
            }]->(a)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-005'}), (a:Assay {id: 'AS-004'})
            INSERT (c)-[:TESTED_IN {
                test_date: '2024-03-25',
                concentration_range: '1-1000 nM',
                replicate_count: 5
            }]->(a)
            """);

        session.execute("""
            MATCH (a:Assay {id: 'AS-001'}), (p:Protein {id: 'EGFR'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'Kinase inhibition',
                units: 'percent inhibition'
            }]->(p)
            """);
        session.execute("""
            MATCH (a:Assay {id: 'AS-002'}), (p:Protein {id: 'ACE2'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'Binding affinity',
                units: 'KD (nM)'
            }]->(p)
            """);
        session.execute("""
            MATCH (a:Assay {id: 'AS-003'}), (p:Protein {id: 'BACE1'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'Enzymatic activity',
                units: 'percent inhibition'
            }]->(p)
            """);
        session.execute("""
            MATCH (a:Assay {id: 'AS-004'}), (p:Protein {id: 'TP53'})
            INSERT (a)-[:MEASURES_ACTIVITY_ON {
                readout: 'PPI disruption',
                units: 'IC50 (nM)'
            }]->(p)
            """);

        session.execute("""
            MATCH (c:Compound {id: 'CP-002'}), (p:Protein {id: 'EGFR'})
            INSERT (c)-[:INHIBITS {
                IC50: 37.5,
                IC50_unit: 'nM',
                Ki: 12.3,
                selectivity_index: 25.6,
                measurement_date: '2024-01-15'
            }]->(p)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-003'}), (p:Protein {id: 'ACE2'})
            INSERT (c)-[:INHIBITS {
                IC50: 23.0,
                IC50_unit: 'nM',
                Ki: 7.8,
                selectivity_index: 15.2,
                measurement_date: '2024-02-20'
            }]->(p)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-004'}), (p:Protein {id: 'BACE1'})
            INSERT (c)-[:INHIBITS {
                IC50: 85.0,
                IC50_unit: 'nM',
                Ki: 28.5,
                selectivity_index: 45.1,
                measurement_date: '2024-03-10'
            }]->(p)
            """);
        session.execute("""
            MATCH (c:Compound {id: 'CP-005'}), (p:Protein {id: 'TP53'})
            INSERT (c)-[:INHIBITS {
                IC50: 12.5,
                IC50_unit: 'nM',
                Ki: 3.2,
                selectivity_index: 120.5,
                measurement_date: '2024-03-25'
            }]->(p)
            """);
    }

    private static void runAnalyticalQueries(Session session) {
        QueryResult potentTp53 = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein {id: 'TP53'})
            WHERE i.IC50 < 100
            RETURN c.name AS compound, c.id AS id, i.IC50 AS ic50, i.IC50_unit AS unit, i.Ki AS ki
            ORDER BY i.IC50
            """);
        printSection("Query 1: Compounds targeting TP53 with IC50 < 100 nM", potentTp53);

        QueryResult pathway = session.query("""
            MATCH (c:Compound {id: 'CP-002'})-[t:TESTED_IN]->(a:Assay)-[m:MEASURES_ACTIVITY_ON]->(p:Protein)
            RETURN c.name AS compound, a.name AS assay, a.assay_type AS type, p.name AS target, p.disease AS disease
            """);
        printSection("Query 2: Complete testing pathway for Gefitinib", pathway);

        QueryResult byPotency = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
            RETURN c.name AS compound,
                   p.name AS target,
                   p.disease AS disease,
                   i.IC50 AS ic50_nM,
                   c.development_stage AS stage
            ORDER BY i.IC50
            """);
        printSection("Query 3: Compound-target interactions ordered by potency", byPotency);

        QueryResult clinical = session.query("""
            MATCH (c:Compound)-[i:INHIBITS]->(p:Protein)
            WHERE c.development_stage LIKE '%Clinical Trial%'
            RETURN c.name AS compound,
                   c.development_stage AS stage,
                   p.name AS target,
                   i.IC50 AS potency_nM,
                   i.selectivity_index AS selectivity
            """);
        printSection("Query 4: Clinical-trial compounds and targets", clinical);

        QueryResult aggregation = session.query("""
            MATCH (p:Protein)<-[:INHIBITS]-(c:Compound)
            RETURN p.name AS protein, p.disease AS disease, COUNT(c) AS compound_count
            """);
        printSection("Query 5: Proteins with multiple targeting compounds", aggregation);
    }

    private static void printSection(String title, QueryResult result) {
        System.out.println(title);
        if (result.isEmpty()) {
            System.out.println("  (no rows)");
            System.out.println();
            return;
        }

        System.out.println("  columns: " + result.variables());
        for (Map<String, Object> row : result) {
            System.out.println("  - " + row);
        }
        System.out.println();
    }

    private static void deleteRecursively(Path root) throws IOException {
        if (!Files.exists(root)) {
            return;
        }

        try (var walk = Files.walk(root)) {
            walk.sorted(Comparator.reverseOrder())
                .forEach(path -> {
                    try {
                        Files.deleteIfExists(path);
                    } catch (IOException ignored) {
                        // Best-effort cleanup for demo directories.
                    }
                });
        }
    }
}
