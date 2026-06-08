use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub node_type: String, // "person", "app", "project", "file"
    pub metadata: String,  // JSON string
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from_id: String,
    pub to_id: String,
    pub relation: String, // "launches", "edits", "knows", "contains"
    pub metadata: String, // JSON string
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationEntry {
    pub id: i64,
    pub timestamp: String,
    pub sender: String, // "User" or "Alok"
    pub message: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        // Create Entity Graph Tables
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                node_type TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS edges (
                from_id TEXT NOT NULL,
                to_id TEXT NOT NULL,
                relation TEXT NOT NULL,
                metadata TEXT,
                PRIMARY KEY (from_id, to_id, relation),
                FOREIGN KEY(from_id) REFERENCES nodes(id) ON DELETE CASCADE,
                FOREIGN KEY(to_id) REFERENCES nodes(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create Dialog Logs Table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                sender TEXT NOT NULL,
                message TEXT NOT NULL
            )",
            [],
        )?;

        // Create Persistent Settings Table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Create Memory Tables
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                description TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS relationships (
                from_entity_id TEXT NOT NULL,
                to_entity_id TEXT NOT NULL,
                relation TEXT NOT NULL,
                PRIMARY KEY (from_entity_id, to_entity_id, relation),
                FOREIGN KEY(from_entity_id) REFERENCES entities(id) ON DELETE CASCADE,
                FOREIGN KEY(to_entity_id) REFERENCES entities(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                importance_score REAL DEFAULT 0.5,
                decay_score REAL DEFAULT 0.0,
                recall_frequency INTEGER DEFAULT 0,
                last_recalled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Migrate existing memories table if columns don't exist
        let table_info: Vec<String> = {
            let mut stmt = self.conn.prepare("PRAGMA table_info(memories)")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
            let mut names = Vec::new();
            for r in rows {
                names.push(r?);
            }
            names
        };
        if !table_info.contains(&"importance_score".to_string()) {
            let _ = self.conn.execute("ALTER TABLE memories ADD COLUMN importance_score REAL DEFAULT 0.5", []);
        }
        if !table_info.contains(&"decay_score".to_string()) {
            let _ = self.conn.execute("ALTER TABLE memories ADD COLUMN decay_score REAL DEFAULT 0.0", []);
        }
        if !table_info.contains(&"recall_frequency".to_string()) {
            let _ = self.conn.execute("ALTER TABLE memories ADD COLUMN recall_frequency INTEGER DEFAULT 0", []);
        }
        if !table_info.contains(&"last_recalled_at".to_string()) {
            let _ = self.conn.execute("ALTER TABLE memories ADD COLUMN last_recalled_at DATETIME DEFAULT CURRENT_TIMESTAMP", []);
        }

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS memory_tags (
                memory_id INTEGER,
                tag TEXT NOT NULL,
                PRIMARY KEY (memory_id, tag),
                FOREIGN KEY(memory_id) REFERENCES memories(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS conversation_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                sender TEXT NOT NULL,
                message TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS user_preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Create Habit tracker and Proactive tables
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS behavior_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                process_name TEXT NOT NULL,
                window_title TEXT NOT NULL,
                browser_url TEXT,
                active_folder TEXT,
                active_file TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS user_habits (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                habit_type TEXT NOT NULL,
                target TEXT NOT NULL,
                hour INTEGER NOT NULL,
                confidence REAL DEFAULT 0.0,
                last_detected_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS calendar_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                start_time DATETIME NOT NULL,
                description TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS learned_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_type TEXT NOT NULL,
                key_name TEXT NOT NULL,
                val_name TEXT NOT NULL,
                frequency INTEGER DEFAULT 1,
                confidence REAL DEFAULT 0.1,
                last_used DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(pattern_type, key_name)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS workspace_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_name TEXT NOT NULL,
                project_name TEXT NOT NULL,
                project_path TEXT NOT NULL,
                open_apps TEXT NOT NULL,
                open_files TEXT NOT NULL,
                browser_tabs TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS task_executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_name TEXT NOT NULL,
                command TEXT NOT NULL,
                output TEXT NOT NULL,
                success INTEGER NOT NULL,
                error_msg TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                duration_ms REAL DEFAULT 0.0,
                retries INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Migrate existing task_executions table if columns don't exist
        let exec_table_info: Vec<String> = {
            let mut stmt = self.conn.prepare("PRAGMA table_info(task_executions)")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
            let mut names = Vec::new();
            for r in rows {
                names.push(r?);
            }
            names
        };
        if !exec_table_info.contains(&"duration_ms".to_string()) {
            let _ = self.conn.execute("ALTER TABLE task_executions ADD COLUMN duration_ms REAL DEFAULT 0.0", []);
        }
        if !exec_table_info.contains(&"retries".to_string()) {
            let _ = self.conn.execute("ALTER TABLE task_executions ADD COLUMN retries INTEGER DEFAULT 0", []);
        }

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS solutions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                error_pattern TEXT NOT NULL,
                solution_details TEXT NOT NULL,
                build_command TEXT NOT NULL,
                resolved_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS performance_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                metric_name TEXT NOT NULL,
                value_ms REAL NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS corrections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                incorrect_input TEXT NOT NULL,
                corrected_input TEXT NOT NULL,
                associated_intent TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS self_improvement_stats (
                command_name TEXT PRIMARY KEY,
                success_count INTEGER DEFAULT 0,
                fail_count INTEGER DEFAULT 0,
                success_rate REAL DEFAULT 1.0,
                confidence REAL DEFAULT 0.5
            )",
            [],
        )?;

        // Seed initial graph nodes if empty
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM nodes",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            self.seed_initial_data()?;
        }

        // Seed default settings if empty
        let settings_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM settings WHERE key = 'groq_api_key'",
            [],
            |row| row.get(0),
        )?;

        if settings_count == 0 {
            let env_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
            self.set_setting("groq_api_key", &env_key)?;
            self.set_setting("groq_model", "llama-3.1-8b-instant")?;
            self.set_setting("wakeword_sensitivity", "0.75")?;
            self.set_setting("tts_speed", "1.0")?;
        }

        Ok(())
    }

    fn seed_initial_data(&self) -> Result<()> {
        let initial_nodes = vec![
            ("user", "User (Alok)", "person", "{}"),
            ("alok_os", "ALOK OS", "system", "{}"),
            ("chrome", "Chrome", "app", "{\"path\":\"chrome.exe\"}"),
            ("vscode", "VS Code", "app", "{\"path\":\"code.cmd\"}"),
        ];

        for (id, label, node_type, meta) in initial_nodes {
            self.conn.execute(
                "INSERT INTO nodes (id, label, node_type, metadata) VALUES (?1, ?2, ?3, ?4)",
                params![id, label, node_type, meta],
            )?;
        }

        let initial_edges = vec![
            ("user", "alok_os", "Controls", "{}"),
            ("alok_os", "chrome", "Launches", "{}"),
            ("alok_os", "vscode", "Launches", "{}"),
        ];

        for (from, to, rel, meta) in initial_edges {
            self.conn.execute(
                "INSERT INTO edges (from_id, to_id, relation, metadata) VALUES (?1, ?2, ?3, ?4)",
                params![from, to, rel, meta],
            )?;
        }

        Ok(())
    }

    // Graph Database Operations
    pub fn get_nodes(&self) -> Result<Vec<Node>> {
        let mut stmt = self.conn.prepare("SELECT id, label, node_type, metadata FROM nodes")?;
        let node_iter = stmt.query_map([], |row| {
            Ok(Node {
                id: row.get(0)?,
                label: row.get(1)?,
                node_type: row.get(2)?,
                metadata: row.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for node in node_iter {
            list.push(node?);
        }
        Ok(list)
    }

    pub fn get_edges(&self) -> Result<Vec<Edge>> {
        let mut stmt = self.conn.prepare("SELECT from_id, to_id, relation, metadata FROM edges")?;
        let edge_iter = stmt.query_map([], |row| {
            Ok(Edge {
                from_id: row.get(0)?,
                to_id: row.get(1)?,
                relation: row.get(2)?,
                metadata: row.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for edge in edge_iter {
            list.push(edge?);
        }
        Ok(list)
    }

    pub fn add_node(&self, node: &Node) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO nodes (id, label, node_type, metadata) VALUES (?1, ?2, ?3, ?4)",
            params![node.id, node.label, node.node_type, node.metadata],
        )?;
        Ok(())
    }

    pub fn add_edge(&self, edge: &Edge) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO edges (from_id, to_id, relation, metadata) VALUES (?1, ?2, ?3, ?4)",
            params![edge.from_id, edge.to_id, edge.relation, edge.metadata],
        )?;
        Ok(())
    }

    // Conversation logs Operations
    pub fn add_conversation_entry(&self, sender: &str, message: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO conversations (sender, message) VALUES (?1, ?2)",
            params![sender, message],
        )?;
        // Also add to conversation_history for consistency
        let _ = self.add_conversation_history(sender, message);
        Ok(())
    }

    pub fn add_conversation_history(&self, sender: &str, message: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO conversation_history (sender, message) VALUES (?1, ?2)",
            params![sender, message],
        )?;
        Ok(())
    }

    pub fn get_conversation_history(&self, limit: u32) -> Result<Vec<ConversationEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, sender, message FROM conversation_history ORDER BY id DESC LIMIT ?1",
        )?;
        let entry_iter = stmt.query_map(params![limit], |row| {
            Ok(ConversationEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                sender: row.get(2)?,
                message: row.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for entry in entry_iter {
            list.push(entry?);
        }
        list.reverse();
        Ok(list)
    }

    // Settings Operations
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            if val.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(val))
            }
        } else {
            Ok(None)
        }
    }

    // Memories Operations
    pub fn auto_extract_relationships(&self, content: &str) {
        let content_clean = content.trim().trim_end_matches('.');
        let words: Vec<&str> = content_clean.split_whitespace().collect();
        let relations = ["likes", "loves", "prefers", "works", "lives", "is"];
        
        for (i, word) in words.iter().enumerate() {
            let word_lower = word.to_lowercase();
            if relations.contains(&word_lower.as_str()) {
                if i > 0 && i < words.len() - 1 {
                    let subject = words[..i].join(" ");
                    let mut rel = word.to_string();
                    let mut object_start = i + 1;
                    
                    if (word_lower == "works" || word_lower == "lives") && i + 1 < words.len() {
                        let next_word = words[i + 1].to_lowercase();
                        if next_word == "at" || next_word == "in" {
                            rel = format!("{} {}", word, words[i + 1]);
                            object_start = i + 2;
                        }
                    }
                    
                    if object_start < words.len() {
                        let object = words[object_start..].join(" ");
                        let subject_id = subject.to_lowercase().replace(' ', "_");
                        let object_id = object.to_lowercase().replace(' ', "_");
                        
                        let subject_type = if subject.chars().next().map_or(false, |c| c.is_uppercase()) {
                            "person"
                        } else {
                            "entity"
                        };
                        let object_type = if object.chars().next().map_or(false, |c| c.is_uppercase()) {
                            "person"
                        } else {
                            "concept"
                        };
                        
                        let _ = self.add_entity(&subject_id, &subject, subject_type, "");
                        let _ = self.add_entity(&object_id, &object, object_type, "");
                        let _ = self.add_relationship(&subject_id, &object_id, &rel);

                        // Also add to visual graph database nodes and edges tables
                        let _ = self.add_node(&Node {
                            id: subject_id.clone(),
                            label: subject.clone(),
                            node_type: subject_type.to_string(),
                            metadata: "{}".to_string(),
                        });
                        let _ = self.add_node(&Node {
                            id: object_id.clone(),
                            label: object.clone(),
                            node_type: object_type.to_string(),
                            metadata: "{}".to_string(),
                        });
                        let _ = self.add_edge(&Edge {
                            from_id: subject_id,
                            to_id: object_id,
                            relation: rel,
                            metadata: "{}".to_string(),
                        });
                    }
                }
                break;
            }
        }
    }

    pub fn store_memory_autonomous(&self, content: &str, tags: Vec<String>, importance: f64) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO memories (content, importance_score, decay_score, recall_frequency, last_recalled_at) VALUES (?1, ?2, 0.0, 0, CURRENT_TIMESTAMP)",
            params![content, importance],
        )?;
        let id = self.conn.last_insert_rowid();
        for tag in tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO memory_tags (memory_id, tag) VALUES (?1, ?2)",
                params![id, tag],
            )?;
        }
        self.auto_extract_relationships(content);
        Ok(id)
    }

    pub fn store_memory(&self, content: &str, tags: Vec<String>) -> Result<i64> {
        self.store_memory_autonomous(content, tags, 0.5)
    }

    pub fn search_memories(&self, query: &str) -> Result<Vec<String>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT m.id, m.content, m.importance_score, m.decay_score, m.recall_frequency FROM memories m 
             LEFT JOIN memory_tags t ON m.id = t.memory_id 
             WHERE m.content LIKE ?1 OR t.tag LIKE ?1",
        )?;
        
        struct MemoryItem {
            id: i64,
            content: String,
            score: f64,
        }

        let rows = stmt.query_map(params![pattern], |row| {
            let id: i64 = row.get(0)?;
            let content: String = row.get(1)?;
            let importance: f64 = row.get(2).unwrap_or(0.5);
            let decay: f64 = row.get(3).unwrap_or(0.0);
            let frequency: i64 = row.get(4).unwrap_or(0);
            
            let score = importance * (1.0 - decay) * (1.0 + 0.1 * (frequency as f64));
            Ok(MemoryItem { id, content, score })
        })?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }

        for item in &items {
            let _ = self.conn.execute(
                "UPDATE memories SET recall_frequency = recall_frequency + 1, last_recalled_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![item.id],
            );
        }

        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let results = items.into_iter().map(|item| item.content).collect();
        Ok(results)
    }

    pub fn update_memory(&self, id: i64, new_content: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE memories SET content = ?1 WHERE id = ?2",
            params![new_content, id],
        )?;
        Ok(())
    }

    pub fn forget_memory(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Entities and Relationships Operations
    pub fn add_entity(&self, id: &str, name: &str, entity_type: &str, description: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO entities (id, name, entity_type, description) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, entity_type, description],
        )?;
        Ok(())
    }

    pub fn add_relationship(&self, from_id: &str, to_id: &str, relation: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO relationships (from_entity_id, to_entity_id, relation) VALUES (?1, ?2, ?3)",
            params![from_id, to_id, relation],
        )?;
        Ok(())
    }

    pub fn query_relationships(&self, entity_name: &str) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT e1.name, r.relation, e2.name FROM relationships r 
             JOIN entities e1 ON r.from_entity_id = e1.id 
             JOIN entities e2 ON r.to_entity_id = e2.id 
             WHERE e1.name LIKE ?1 OR e2.name LIKE ?1",
        )?;
        let pattern = format!("%{}%", entity_name);
        let rows = stmt.query_map(params![pattern], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // User Preferences Operations
    pub fn set_user_preference(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_preferences (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_user_preference(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM user_preferences WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn decay_memories(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT id, importance_score, (julianday('now') - julianday(last_recalled_at)) * 24.0 FROM memories"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let importance: f64 = row.get(1).unwrap_or(0.5);
            let hours: f64 = row.get(2).unwrap_or(0.0);
            Ok((id, importance, hours))
        })?;

        let mut to_delete = Vec::new();
        let mut to_update = Vec::new();

        for r in rows {
            let (id, importance, hours) = r?;
            let decay_rate = 0.005 * (1.0 - importance);
            let decay_score = 1.0 - (-decay_rate * hours).exp();
            let decay_score = decay_score.clamp(0.0, 1.0);

            if decay_score >= 0.8 {
                to_delete.push(id);
            } else {
                to_update.push((id, decay_score));
            }
        }

        for (id, decay) in to_update {
            self.conn.execute(
                "UPDATE memories SET decay_score = ?1 WHERE id = ?2",
                params![decay, id],
            )?;
        }

        for id in to_delete {
            let _ = self.forget_memory(id);
        }

        Ok(())
    }

    pub fn get_all_memories_raw(&self) -> Result<Vec<(i64, String, f64)>> {
        let mut stmt = self.conn.prepare("SELECT id, content, importance_score FROM memories")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2).unwrap_or(0.5)))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_all_memories_summary_raw(&self) -> Result<Vec<(String, f64, f64, i64)>> {
        let mut stmt = self.conn.prepare("SELECT content, importance_score, decay_score, recall_frequency FROM memories")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1).unwrap_or(0.5),
                row.get(2).unwrap_or(0.0),
                row.get(3).unwrap_or(0),
            ))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn update_last_recalled_at_for_test(&self, id: i64, hours_ago: f64) -> Result<()> {
        let hours_str = format!("-{} hours", hours_ago);
        self.conn.execute(
            "UPDATE memories SET last_recalled_at = datetime('now', ?1) WHERE id = ?2",
            params![hours_str, id],
        )?;
        Ok(())
    }

    pub fn get_current_local_hour(&self) -> Result<i32> {
        let mut stmt = self.conn.prepare("SELECT strftime('%H', 'now', 'localtime')")?;
        let hour_str: String = stmt.query_row([], |row| row.get(0))?;
        let hour = hour_str.parse::<i32>().unwrap_or(-1);
        Ok(hour)
    }

    // --- Habit Engine & Proactive Assistant database operations ---
    
    pub fn log_behavior(&self, process: &str, title: &str, url: Option<&str>, folder: Option<&str>, file: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO behavior_logs (process_name, window_title, browser_url, active_folder, active_file) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![process, title, url, folder, file],
        )?;
        Ok(())
    }

    pub fn log_behavior_with_timestamp(&self, process: &str, title: &str, url: Option<&str>, folder: Option<&str>, file: Option<&str>, timestamp: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO behavior_logs (process_name, window_title, browser_url, active_folder, active_file, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![process, title, url, folder, file, timestamp],
        )?;
        Ok(())
    }

    pub fn get_behavior_logs(&self) -> Result<Vec<(String, String, Option<String>, Option<String>, Option<String>, String)>> {
        let mut stmt = self.conn.prepare("SELECT process_name, window_title, browser_url, active_folder, active_file, timestamp FROM behavior_logs ORDER BY id DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn add_user_habit(&self, habit_type: &str, target: &str, hour: i32, confidence: f64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_habits (habit_type, target, hour, confidence, last_detected_at) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
            params![habit_type, target, hour, confidence],
        )?;
        Ok(())
    }

    pub fn clear_user_habits(&self) -> Result<()> {
        self.conn.execute("DELETE FROM user_habits", [])?;
        Ok(())
    }

    pub fn get_habits_for_hour(&self, hour: i32) -> Result<Vec<(String, String, i32, f64)>> {
        let mut stmt = self.conn.prepare("SELECT habit_type, target, hour, confidence FROM user_habits WHERE hour = ?1 OR hour = ?2 OR hour = ?3 ORDER BY confidence DESC")?;
        let prev_hour = if hour == 0 { 23 } else { hour - 1 };
        let next_hour = if hour == 23 { 0 } else { hour + 1 };
        
        let rows = stmt.query_map(params![hour, prev_hour, next_hour], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_all_habits(&self) -> Result<Vec<(String, String, i32, f64)>> {
        let mut stmt = self.conn.prepare("SELECT habit_type, target, hour, confidence FROM user_habits ORDER BY confidence DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn add_calendar_event(&self, title: &str, start_time_offset_mins: i64, description: &str) -> Result<()> {
        let start_time_str = format!("+{} minutes", start_time_offset_mins);
        self.conn.execute(
            "INSERT INTO calendar_events (title, start_time, description) VALUES (?1, datetime('now', ?2), ?3)",
            params![title, start_time_str, description],
        )?;
        Ok(())
    }

    pub fn get_upcoming_meetings(&self, within_minutes: i64) -> Result<Vec<(i64, String, String, Option<String>)>> {
        let threshold_str = format!("+{} minutes", within_minutes);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, start_time, description FROM calendar_events 
             WHERE start_time >= datetime('now') AND start_time <= datetime('now', ?1)"
        )?;
        let rows = stmt.query_map(params![threshold_str], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn record_learned_pattern(&self, pattern_type: &str, key_name: &str, val_name: &str) -> Result<()> {
        let key_lower = key_name.to_lowercase().trim().to_string();
        let val_lower = val_name.to_lowercase().trim().to_string();

        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM learned_patterns WHERE pattern_type = ?1 AND key_name = ?2",
            params![pattern_type, key_lower],
            |row| row.get(0),
        )?;

        if count > 0 {
            let freq: i64 = self.conn.query_row(
                "SELECT frequency FROM learned_patterns WHERE pattern_type = ?1 AND key_name = ?2",
                params![pattern_type, key_lower],
                |row| row.get(0),
            )?;
            let new_freq = freq + 1;
            let new_conf = (new_freq as f64 * 0.15).min(1.0);
            self.conn.execute(
                "UPDATE learned_patterns SET val_name = ?1, frequency = ?2, confidence = ?3, last_used = CURRENT_TIMESTAMP WHERE pattern_type = ?4 AND key_name = ?5",
                params![val_lower, new_freq, new_conf, pattern_type, key_lower],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO learned_patterns (pattern_type, key_name, val_name, frequency, confidence) VALUES (?1, ?2, ?3, 1, 0.15)",
                params![pattern_type, key_lower, val_lower],
            )?;
        }
        Ok(())
    }

    pub fn get_learned_patterns(&self, pattern_type: &str) -> Result<Vec<(String, String, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT key_name, val_name, confidence FROM learned_patterns WHERE pattern_type = ?1"
        )?;
        let rows = stmt.query_map(params![pattern_type], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_preferred_app(&self, generic_name: &str) -> Option<String> {
        let key_lower = generic_name.to_lowercase().trim().to_string();
        self.conn.query_row(
            "SELECT val_name FROM learned_patterns WHERE pattern_type = 'preferred_app' AND key_name = ?1 ORDER BY frequency DESC LIMIT 1",
            params![key_lower],
            |row| row.get(0)
        ).ok()
    }

    // Workspace session methods
    pub fn save_workspace_session(&self, name: &str, project_name: &str, project_path: &str, open_apps_json: &str, open_files_json: &str, browser_tabs_json: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO workspace_sessions (session_name, project_name, project_path, open_apps, open_files, browser_tabs) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, project_name, project_path, open_apps_json, open_files_json, browser_tabs_json],
        )?;
        Ok(())
    }

    pub fn get_last_workspace_session(&self) -> Result<Option<WorkspaceSession>> {
        let mut stmt = self.conn.prepare("SELECT id, session_name, project_name, project_path, open_apps, open_files, browser_tabs, created_at FROM workspace_sessions ORDER BY id DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(WorkspaceSession {
                id: row.get(0)?,
                session_name: row.get(1)?,
                project_name: row.get(2)?,
                project_path: row.get(3)?,
                open_apps: row.get(4)?,
                open_files: row.get(5)?,
                browser_tabs: row.get(6)?,
                created_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    // Task execution methods
    pub fn log_task_execution(&self, task_name: &str, command: &str, output: &str, success: bool, error_msg: Option<&str>, duration_ms: f64, retries: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO task_executions (task_name, command, output, success, error_msg, duration_ms, retries) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![task_name, command, output, if success { 1 } else { 0 }, error_msg, duration_ms, retries],
        )?;
        Ok(())
    }

    pub fn get_recent_task_executions(&self, limit: u32) -> Result<Vec<TaskExecution>> {
        let mut stmt = self.conn.prepare("SELECT id, task_name, command, output, success, error_msg, timestamp, duration_ms, retries FROM task_executions ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], |row| {
            let success_val: i32 = row.get(4)?;
            Ok(TaskExecution {
                id: row.get(0)?,
                task_name: row.get(1)?,
                command: row.get(2)?,
                output: row.get(3)?,
                success: success_val != 0,
                error_msg: row.get(5)?,
                timestamp: row.get(6)?,
                duration_ms: row.get(7).unwrap_or(0.0),
                retries: row.get(8).unwrap_or(0),
            })
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    // Solution database methods
    pub fn add_solution(&self, error_pattern: &str, solution_details: &str, build_command: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO solutions (error_pattern, solution_details, build_command) VALUES (?1, ?2, ?3)",
            params![error_pattern, solution_details, build_command],
        )?;
        Ok(())
    }

    pub fn get_solutions(&self) -> Result<Vec<SolutionEntry>> {
        let mut stmt = self.conn.prepare("SELECT id, error_pattern, solution_details, build_command, resolved_at FROM solutions ORDER BY id DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(SolutionEntry {
                id: row.get(0)?,
                error_pattern: row.get(1)?,
                solution_details: row.get(2)?,
                build_command: row.get(3)?,
                resolved_at: row.get(4)?,
            })
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    // Performance metrics methods
    pub fn log_performance_metric(&self, metric_name: &str, value_ms: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO performance_metrics (metric_name, value_ms) VALUES (?1, ?2)",
            params![metric_name, value_ms],
        )?;
        Ok(())
    }

    pub fn get_average_performance_metrics(&self) -> Result<std::collections::HashMap<String, f64>> {
        let mut stmt = self.conn.prepare("SELECT metric_name, AVG(value_ms) FROM performance_metrics GROUP BY metric_name")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;
        let mut results = std::collections::HashMap::new();
        for r in rows {
            let (name, val) = r?;
            results.insert(name, val);
        }
        Ok(results)
    }

    // Self-Improvement & Corrections Database Operations
    pub fn add_correction(&self, incorrect: &str, corrected: &str, intent: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO corrections (incorrect_input, corrected_input, associated_intent) VALUES (?1, ?2, ?3)",
            params![incorrect, corrected, intent],
        )?;
        Ok(())
    }

    pub fn get_corrections_count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM corrections",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn get_recent_corrections(&self, limit: usize) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT incorrect_input, corrected_input, timestamp FROM corrections ORDER BY timestamp DESC LIMIT ?"
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn log_self_improvement_outcome(&self, command: &str, success: bool) -> Result<()> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM self_improvement_stats WHERE command_name = ?1",
            params![command],
            |row| row.get(0),
        )?;

        if count == 0 {
            let success_count = if success { 1 } else { 0 };
            let fail_count = if success { 0 } else { 1 };
            let success_rate = if success { 1.0 } else { 0.0 };
            let confidence = if success { 0.6 } else { 0.4 };
            self.conn.execute(
                "INSERT INTO self_improvement_stats (command_name, success_count, fail_count, success_rate, confidence) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![command, success_count, fail_count, success_rate, confidence],
            )?;
        } else {
            let (mut sc, mut fc): (i32, i32) = self.conn.query_row(
                "SELECT success_count, fail_count FROM self_improvement_stats WHERE command_name = ?1",
                params![command],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;

            if success { sc += 1; } else { fc += 1; }
            let total = sc + fc;
            let success_rate = sc as f64 / total as f64;
            let confidence = if success {
                (sc as f64 / total as f64).min(0.99)
            } else {
                (sc as f64 / total as f64).max(0.01)
            };

            self.conn.execute(
                "UPDATE self_improvement_stats SET success_count = ?1, fail_count = ?2, success_rate = ?3, confidence = ?4 WHERE command_name = ?5",
                params![sc, fc, success_rate, confidence, command],
            )?;
        }
        Ok(())
    }

    pub fn get_self_improvement_metrics(&self) -> Result<Vec<(String, i32, i32, f64, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT command_name, success_count, fail_count, success_rate, confidence FROM self_improvement_stats ORDER BY success_rate DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_memories_count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memories",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceSession {
    pub id: i64,
    pub session_name: String,
    pub project_name: String,
    pub project_path: String,
    pub open_apps: String,
    pub open_files: String,
    pub browser_tabs: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskExecution {
    pub id: i64,
    pub task_name: String,
    pub command: String,
    pub output: String,
    pub success: bool,
    pub error_msg: Option<String>,
    pub timestamp: String,
    pub duration_ms: f64,
    pub retries: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolutionEntry {
    pub id: i64,
    pub error_pattern: String,
    pub solution_details: String,
    pub build_command: String,
    pub resolved_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceMetric {
    pub id: i64,
    pub metric_name: String,
    pub value_ms: f64,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_ranking_and_decay() {
        let db = Database::new(":memory:").unwrap();
        
        let _id_high = db.store_memory_autonomous("User prefers Rust programming.", vec!["rust".to_string()], 0.9).unwrap();
        let id_low = db.store_memory_autonomous("User had pizza for lunch.", vec!["pizza".to_string()], 0.2).unwrap();
        let _id_med = db.store_memory_autonomous("User is working on ALOK OS.", vec!["alok".to_string()], 0.5).unwrap();

        // 1. Test ranking
        let results = db.search_memories("").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], "User prefers Rust programming.");
        assert_eq!(results[1], "User is working on ALOK OS.");
        assert_eq!(results[2], "User had pizza for lunch.");

        // 2. Test recall frequency tracking
        let _ = db.search_memories("Rust").unwrap();
        let list = db.get_all_memories_summary_raw().unwrap();
        let rust_mem = list.iter().find(|(content, _, _, _)| content.contains("Rust")).unwrap();
        assert_eq!(rust_mem.3, 2);

        // 3. Test decay calculations
        db.conn.execute(
            "UPDATE memories SET last_recalled_at = datetime('now', '-200 hours') WHERE id = ?1",
            params![id_low],
        ).unwrap();

        db.decay_memories().unwrap();

        let list2 = db.get_all_memories_summary_raw().unwrap();
        let pizza_mem = list2.iter().find(|(content, _, _, _)| content.contains("pizza")).unwrap();
        assert!(pizza_mem.2 > 0.0);
        assert!(pizza_mem.2 < 0.8);

        db.conn.execute(
            "UPDATE memories SET last_recalled_at = datetime('now', '-2000 hours') WHERE id = ?1",
            params![id_low],
        ).unwrap();

        db.decay_memories().unwrap();

        let list3 = db.get_all_memories_summary_raw().unwrap();
        let pizza_exists = list3.iter().any(|(content, _, _, _)| content.contains("pizza"));
        assert!(!pizza_exists);
    }
}


