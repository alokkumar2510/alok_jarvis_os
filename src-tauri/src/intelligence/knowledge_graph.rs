use crate::database::db::{Database, Node, Edge};
use std::sync::{Arc, Mutex};

pub struct KnowledgeGraph {
    db: Arc<Mutex<Database>>,
}

impl KnowledgeGraph {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    /// Add a node to both the visual graph and semantic entity database.
    pub fn add_entity(&self, id: &str, label: &str, entity_type: &str, metadata: &str) -> Result<(), String> {
        let db_lock = self.db.lock().map_err(|_| "Failed to lock database")?;
        db_lock.add_node(&Node {
            id: id.to_string(),
            label: label.to_string(),
            node_type: entity_type.to_string(),
            metadata: metadata.to_string(),
        }).map_err(|e| e.to_string())?;

        db_lock.add_entity(id, label, entity_type, "").map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Add an edge to both the visual graph and semantic relationships database.
    pub fn add_relationship(&self, from: &str, to: &str, relation: &str) -> Result<(), String> {
        let db_lock = self.db.lock().map_err(|_| "Failed to lock database")?;
        db_lock.add_edge(&Edge {
            from_id: from.to_string(),
            to_id: to.to_string(),
            relation: relation.to_string(),
            metadata: "{}".to_string(),
        }).map_err(|e| e.to_string())?;

        db_lock.add_relationship(from, to, relation).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Fetch all entities (nodes) in the knowledge graph.
    pub fn get_entities(&self) -> Result<Vec<Node>, String> {
        let db_lock = self.db.lock().map_err(|_| "Failed to lock database")?;
        db_lock.get_nodes().map_err(|e| e.to_string())
    }

    /// Fetch all relationships (edges) in the knowledge graph.
    pub fn get_relationships(&self) -> Result<Vec<Edge>, String> {
        let db_lock = self.db.lock().map_err(|_| "Failed to lock database")?;
        db_lock.get_edges().map_err(|e| e.to_string())
    }

    /// Query relationships involving a specific keyword.
    pub fn query_graph(&self, entity_name: &str) -> Result<Vec<(String, String, String)>, String> {
        let db_lock = self.db.lock().map_err(|_| "Failed to lock database")?;
        db_lock.query_relationships(entity_name).map_err(|e| e.to_string())
    }
}
