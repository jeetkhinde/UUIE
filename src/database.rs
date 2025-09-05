// Database module - handles Supabase connection and SQL operations
use sqlx::{Column, PgPool, Row};
use std::collections::HashMap;
use std::env;

// Database connection wrapper for Supabase
pub struct Database {
    pool: PgPool,
}

impl Database {
    // Create new database connection
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

        // Create connection pool
        let pool = PgPool::connect(&database_url).await?;

        Ok(Self { pool })
    }

    // Execute schema SQL files (CREATE TABLE, CREATE COMPONENT, etc.)
    pub async fn execute_schema(&self, sql: &str) -> Result<(), sqlx::Error> {
        // Split SQL by semicolons and execute each statement
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&self.pool).await?;
            }
        }
        Ok(())
    }

    // Load schema SQL file for a table
    pub async fn load_table_schema(
        &self,
        table_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match table_name {
            "users" => {
                let sql = include_str!("../schemas/users/users.sql");
                self.execute_schema(sql).await?;
            }
            // Add more tables here:
            // "products" => {
            //     let sql = include_str!("../schemas/products/products.sql");
            //     self.execute_schema(sql).await?;
            // }
            _ => {
                return Err(format!("Unknown table: {}", table_name).into());
            }
        }
        Ok(())
    }

    // Fetch single record by ID
    pub async fn get_record(
        &self,
        table: &str,
        id: &str,
    ) -> Result<HashMap<String, String>, sqlx::Error> {
        let query = format!("SELECT * FROM {} WHERE id = $1", table);
        let row = sqlx::query(&query).bind(id).fetch_one(&self.pool).await?;

        // Convert row to HashMap
        let mut record = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let value: Option<String> = row.try_get(i).ok();
            if let Some(val) = value {
                record.insert(column.name().to_string(), val);
            }
        }

        Ok(record)
    }

    // Fetch multiple records with optional limit
    pub async fn get_records(
        &self,
        table: &str,
        limit: Option<i32>,
    ) -> Result<Vec<HashMap<String, String>>, sqlx::Error> {
        let query = if let Some(limit) = limit {
            format!("SELECT * FROM {} LIMIT {}", table, limit)
        } else {
            format!("SELECT * FROM {}", table)
        };

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut records = Vec::new();
        for row in rows {
            let mut record = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: Option<String> = row.try_get(i).ok();
                if let Some(val) = value {
                    record.insert(column.name().to_string(), val);
                }
            }
            records.push(record);
        }

        Ok(records)
    }

    // Insert new record
    pub async fn insert_record(
        &self,
        table: &str,
        data: &HashMap<String, String>,
    ) -> Result<String, sqlx::Error> {
        let fields: Vec<&String> = data.keys().collect();
        let placeholders: Vec<String> = (1..=fields.len()).map(|i| format!("${}", i)).collect();

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
            table,
            fields
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            placeholders.join(", ")
        );

        let mut query_builder = sqlx::query(&query);
        for field in &fields {
            query_builder = query_builder.bind(data.get(*field).unwrap());
        }

        let row = query_builder.fetch_one(&self.pool).await?;
        let id: String = row.try_get("id")?;

        Ok(id)
    }

    // Close database connection
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_database_connection() {
        dotenv().ok();

        // Only run if DATABASE_URL is set
        if env::var("DATABASE_URL").is_ok() {
            let db = Database::new().await;
            assert!(db.is_ok());
        }
    }
}
