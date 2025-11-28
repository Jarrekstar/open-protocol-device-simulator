use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Parameter Set (PSET) configuration for tightening operations
/// Each PSET defines the target ranges for torque and angle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pset {
    pub id: u32,
    pub name: String,
    pub torque_min: f64,
    pub torque_max: f64,
    pub angle_min: f64,
    pub angle_max: f64,
    pub description: Option<String>,
}

impl Pset {
    pub fn new(
        id: u32,
        name: String,
        torque_min: f64,
        torque_max: f64,
        angle_min: f64,
        angle_max: f64,
        description: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            torque_min,
            torque_max,
            angle_min,
            angle_max,
            description,
        }
    }

    /// Check if a tightening result is within this PSET's parameters
    pub fn is_within_range(&self, torque: f64, angle: f64) -> bool {
        torque >= self.torque_min
            && torque <= self.torque_max
            && angle >= self.angle_min
            && angle <= self.angle_max
    }
}

/// Repository trait for PSET persistence
/// This abstraction allows for easy switching between in-memory and database storage
pub trait PsetRepository: Send + Sync {
    fn get_all(&self) -> Vec<Pset>;
    fn get_by_id(&self, id: u32) -> Option<Pset>;
    fn create(&mut self, pset: Pset) -> Result<Pset, String>;
    fn update(&mut self, id: u32, pset: Pset) -> Result<Pset, String>;
    fn delete(&mut self, id: u32) -> Result<(), String>;
}

/// In-memory implementation of PsetRepository
/// Future: Replace with SQLite-backed implementation
pub struct InMemoryPsetRepository {
    psets: Vec<Pset>,
}

impl InMemoryPsetRepository {
    pub fn new() -> Self {
        Self {
            psets: Self::default_psets(),
        }
    }

    /// Returns 5 hardcoded default PSETs
    fn default_psets() -> Vec<Pset> {
        vec![
            Pset::new(
                1,
                "Light Duty".to_string(),
                5.0,
                10.0,
                30.0,
                45.0,
                Some("Low torque applications (e.g., electronics, small assemblies)".to_string()),
            ),
            Pset::new(
                2,
                "Standard".to_string(),
                10.0,
                15.0,
                35.0,
                50.0,
                Some("General purpose tightening operations".to_string()),
            ),
            Pset::new(
                3,
                "Heavy Duty".to_string(),
                15.0,
                25.0,
                40.0,
                60.0,
                Some("High torque applications (e.g., automotive, machinery)".to_string()),
            ),
            Pset::new(
                4,
                "Precision".to_string(),
                8.0,
                12.0,
                20.0,
                30.0,
                Some("Tight tolerance requirements".to_string()),
            ),
            Pset::new(
                5,
                "Extra Heavy".to_string(),
                25.0,
                40.0,
                50.0,
                90.0,
                Some("Maximum torque applications (e.g., industrial equipment)".to_string()),
            ),
        ]
    }
}

impl Default for InMemoryPsetRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PsetRepository for InMemoryPsetRepository {
    fn get_all(&self) -> Vec<Pset> {
        self.psets.clone()
    }

    fn get_by_id(&self, id: u32) -> Option<Pset> {
        self.psets.iter().find(|p| p.id == id).cloned()
    }

    fn create(&mut self, mut pset: Pset) -> Result<Pset, String> {
        // Generate new ID
        let max_id = self.psets.iter().map(|p| p.id).max().unwrap_or(0);
        pset.id = max_id + 1;

        self.psets.push(pset.clone());
        Ok(pset)
    }

    fn update(&mut self, id: u32, pset: Pset) -> Result<Pset, String> {
        if let Some(existing) = self.psets.iter_mut().find(|p| p.id == id) {
            *existing = pset.clone();
            Ok(pset)
        } else {
            Err(format!("PSET with id {} not found", id))
        }
    }

    fn delete(&mut self, id: u32) -> Result<(), String> {
        let initial_len = self.psets.len();
        self.psets.retain(|p| p.id != id);

        if self.psets.len() < initial_len {
            Ok(())
        } else {
            Err(format!("PSET with id {} not found", id))
        }
    }
}

/// SQLite-backed implementation of PsetRepository
pub struct SqlitePsetRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqlitePsetRepository {
    /// Create a new SQLite repository with the given database file path
    pub fn new(db_path: &str) -> Result<Self, String> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::new(manager).map_err(|e| format!("Failed to create pool: {}", e))?;

        let repo = Self { pool };
        repo.init_schema()?;
        repo.seed_if_empty()?;

        Ok(repo)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<(), String> {
        let conn = self
            .pool
            .get()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS psets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                torque_min REAL NOT NULL,
                torque_max REAL NOT NULL,
                angle_min REAL NOT NULL,
                angle_max REAL NOT NULL,
                description TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| format!("Failed to create table: {}", e))?;

        Ok(())
    }

    /// Seed database with default PSETs if empty
    fn seed_if_empty(&self) -> Result<(), String> {
        let conn = self
            .pool
            .get()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM psets", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count PSETs: {}", e))?;

        if count == 0 {
            println!("Seeding database with default PSETs...");
            for pset in InMemoryPsetRepository::default_psets() {
                conn.execute(
                    "INSERT INTO psets (name, torque_min, torque_max, angle_min, angle_max, description, is_default)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                    params![
                        pset.name,
                        pset.torque_min,
                        pset.torque_max,
                        pset.angle_min,
                        pset.angle_max,
                        pset.description
                    ],
                )
                .map_err(|e| format!("Failed to seed PSET: {}", e))?;
            }
            println!("Default PSETs seeded successfully");
        }

        Ok(())
    }

    /// Helper to convert a rusqlite Row to a Pset
    fn row_to_pset(row: &rusqlite::Row) -> SqliteResult<Pset> {
        Ok(Pset {
            id: row.get::<_, i64>(0)? as u32,
            name: row.get(1)?,
            torque_min: row.get(2)?,
            torque_max: row.get(3)?,
            angle_min: row.get(4)?,
            angle_max: row.get(5)?,
            description: row.get(6)?,
        })
    }
}

impl PsetRepository for SqlitePsetRepository {
    fn get_all(&self) -> Vec<Pset> {
        let conn = match self.pool.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to get connection: {}", e);
                return vec![];
            }
        };

        let mut stmt = match conn.prepare("SELECT id, name, torque_min, torque_max, angle_min, angle_max, description FROM psets ORDER BY id") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to prepare statement: {}", e);
                return vec![];
            }
        };

        match stmt.query_map([], Self::row_to_pset) {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(e) => {
                eprintln!("Query failed: {}", e);
                vec![]
            }
        }
    }

    fn get_by_id(&self, id: u32) -> Option<Pset> {
        let conn = self.pool.get().ok()?;

        conn.query_row(
            "SELECT id, name, torque_min, torque_max, angle_min, angle_max, description FROM psets WHERE id = ?1",
            params![id as i64],
            Self::row_to_pset,
        )
        .ok()
    }

    fn create(&mut self, pset: Pset) -> Result<Pset, String> {
        let conn = self
            .pool
            .get()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Validate ranges
        if pset.torque_min >= pset.torque_max {
            return Err("torque_min must be less than torque_max".to_string());
        }
        if pset.angle_min >= pset.angle_max {
            return Err("angle_min must be less than angle_max".to_string());
        }
        if pset.torque_min < 0.0 || pset.angle_min < 0.0 {
            return Err("Values must be non-negative".to_string());
        }
        if pset.angle_max > 360.0 {
            return Err("angle_max cannot exceed 360 degrees".to_string());
        }

        conn.execute(
            "INSERT INTO psets (name, torque_min, torque_max, angle_min, angle_max, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                pset.name,
                pset.torque_min,
                pset.torque_max,
                pset.angle_min,
                pset.angle_max,
                pset.description
            ],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                format!("A PSET with name '{}' already exists", pset.name)
            } else {
                format!("Failed to create PSET: {}", e)
            }
        })?;

        let id = conn.last_insert_rowid() as u32;
        self.get_by_id(id)
            .ok_or_else(|| "Failed to retrieve created PSET".to_string())
    }

    fn update(&mut self, id: u32, pset: Pset) -> Result<Pset, String> {
        let conn = self
            .pool
            .get()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Validate ranges
        if pset.torque_min >= pset.torque_max {
            return Err("torque_min must be less than torque_max".to_string());
        }
        if pset.angle_min >= pset.angle_max {
            return Err("angle_min must be less than angle_max".to_string());
        }
        if pset.torque_min < 0.0 || pset.angle_min < 0.0 {
            return Err("Values must be non-negative".to_string());
        }
        if pset.angle_max > 360.0 {
            return Err("angle_max cannot exceed 360 degrees".to_string());
        }

        let rows_affected = conn
            .execute(
                "UPDATE psets SET name = ?1, torque_min = ?2, torque_max = ?3,
                 angle_min = ?4, angle_max = ?5, description = ?6, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?7",
                params![
                    pset.name,
                    pset.torque_min,
                    pset.torque_max,
                    pset.angle_min,
                    pset.angle_max,
                    pset.description,
                    id as i64
                ],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    format!("A PSET with name '{}' already exists", pset.name)
                } else {
                    format!("Failed to update PSET: {}", e)
                }
            })?;

        if rows_affected == 0 {
            return Err(format!("PSET with id {} not found", id));
        }

        self.get_by_id(id)
            .ok_or_else(|| "Failed to retrieve updated PSET".to_string())
    }

    fn delete(&mut self, id: u32) -> Result<(), String> {
        let conn = self
            .pool
            .get()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Check if it's a default PSET
        let is_default: bool = conn
            .query_row(
                "SELECT is_default FROM psets WHERE id = ?1",
                params![id as i64],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if is_default {
            return Err("Cannot delete default PSET".to_string());
        }

        let rows_affected = conn
            .execute("DELETE FROM psets WHERE id = ?1", params![id as i64])
            .map_err(|e| format!("Failed to delete PSET: {}", e))?;

        if rows_affected == 0 {
            Err(format!("PSET with id {} not found", id))
        } else {
            Ok(())
        }
    }
}

/// Thread-safe wrapper for PsetRepository
pub type SharedPsetRepository = Arc<RwLock<Box<dyn PsetRepository>>>;

pub fn create_default_repository() -> SharedPsetRepository {
    Arc::new(RwLock::new(Box::new(InMemoryPsetRepository::new())))
}

pub fn create_sqlite_repository(db_path: &str) -> Result<SharedPsetRepository, String> {
    let repo = SqlitePsetRepository::new(db_path)?;
    Ok(Arc::new(RwLock::new(Box::new(repo))))
}
