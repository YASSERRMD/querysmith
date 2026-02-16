use async_trait::async_trait;
use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Column, Row, TypeInfo,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::traits::{Column as TableColumn, Error, QueryResult, TableSchema, Warehouse};

#[derive(Clone)]
pub struct PostgresWarehouseOptions {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for PostgresWarehouseOptions {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

pub struct PostgresWarehouse {
    pool: Arc<RwLock<Option<PgPool>>>,
    connection_string: String,
    options: PostgresWarehouseOptions,
}

impl PostgresWarehouse {
    pub fn new(connection_string: &str) -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            connection_string: connection_string.to_string(),
            options: PostgresWarehouseOptions::default(),
        }
    }

    pub fn with_options(mut self, options: PostgresWarehouseOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.options.max_connections = max;
        self
    }

    async fn get_pool(&self) -> Result<PgPool, Error> {
        let guard = self.pool.read().await;
        guard
            .clone()
            .ok_or_else(|| Error::Connection("Not connected".to_string()))
    }
}

#[async_trait]
impl Warehouse for PostgresWarehouse {
    async fn connect(&self) -> Result<(), Error> {
        let pool_options = PgPoolOptions::new()
            .max_connections(self.options.max_connections)
            .min_connections(self.options.min_connections)
            .acquire_timeout(self.options.acquire_timeout)
            .idle_timeout(self.options.idle_timeout)
            .max_lifetime(self.options.max_lifetime);

        let pool = pool_options
            .connect(&self.connection_string)
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        let mut guard = self.pool.write().await;
        *guard = Some(pool);
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), Error> {
        let mut guard = self.pool.write().await;
        if let Some(pool) = guard.take() {
            pool.close().await;
        }
        Ok(())
    }

    async fn execute(&self, sql: &str) -> Result<QueryResult, Error> {
        let pool = self.get_pool().await?;

        if sql.trim().to_uppercase().starts_with("SELECT") {
            let rows = sqlx::query(sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| Error::Query(e.to_string()))?;

            let columns: Vec<String> = if !rows.is_empty() {
                rows[0]
                    .columns()
                    .iter()
                    .map(|col| col.name().to_string())
                    .collect()
            } else {
                vec![]
            };

            let mut result_rows: Vec<Vec<serde_json::Value>> = Vec::new();
            for row in &rows {
                let mut row_values: Vec<serde_json::Value> = Vec::new();
                for (i, col) in row.columns().iter().enumerate() {
                    let value = Self::map_value(row, i, col.type_info().name());
                    row_values.push(value);
                }
                result_rows.push(row_values);
            }

            let row_count = result_rows.len();
            Ok(QueryResult {
                columns,
                rows: result_rows,
                row_count,
            })
        } else {
            sqlx::query(sql)
                .execute(&pool)
                .await
                .map_err(|e| Error::Query(e.to_string()))?;

            Ok(QueryResult {
                columns: vec!["affected_rows".to_string()],
                rows: vec![vec![serde_json::Value::Number(1.into())]],
                row_count: 1,
            })
        }
    }

    async fn get_schema(&self, table_name: &str) -> Result<TableSchema, Error> {
        let pool = self.get_pool().await?;

        let columns_sql = format!(
            r#"
            SELECT 
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_comment
            FROM information_schema.columns c
            WHERE c.table_name = '{}'
            AND c.table_schema = 'public'
            ORDER BY c.ordinal_position
            "#,
            table_name
        );

        let columns: Vec<TableColumn> = sqlx::query(&columns_sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| Error::Query(e.to_string()))?
            .iter()
            .map(|row| TableColumn {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                nullable: row.get::<&str, _>("is_nullable") == "YES",
                comment: row.get("column_comment"),
            })
            .collect();

        if columns.is_empty() {
            return Err(Error::Query(format!("Table '{}' not found", table_name)));
        }

        let pk_sql = format!(
            r#"
            SELECT kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu 
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            WHERE tc.table_name = '{}'
                AND tc.constraint_type = 'PRIMARY KEY'
            ORDER BY kcu.ordinal_position
            "#,
            table_name
        );

        let primary_key: Option<Vec<String>> = sqlx::query(&pk_sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| Error::Query(e.to_string()))?
            .iter()
            .map(|row| row.get::<String, _>("column_name"))
            .collect::<Vec<_>>()
            .into();

        Ok(TableSchema {
            name: table_name.to_string(),
            columns,
            primary_key,
        })
    }

    async fn list_tables(&self) -> Result<Vec<String>, Error> {
        let pool = self.get_pool().await?;

        let sql = r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_type = 'BASE TABLE'
            ORDER BY table_name
        "#;

        let rows = sqlx::query(sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| Error::Query(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect())
    }

    async fn preview_table(&self, table_name: &str, limit: usize) -> Result<QueryResult, Error> {
        let sql = format!("SELECT * FROM {} LIMIT {}", table_name, limit);
        self.execute(&sql).await
    }
}

impl PostgresWarehouse {
    fn map_value(row: &PgRow, idx: usize, _type_name: &str) -> serde_json::Value {
        if let Ok(v) = row.try_get::<i64, _>(idx) {
            return serde_json::Value::Number(v.into());
        }
        if let Ok(v) = row.try_get::<i32, _>(idx) {
            return serde_json::Value::Number(v.into());
        }
        if let Ok(v) = row.try_get::<f64, _>(idx) {
            return serde_json::Number::from_f64(v)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null);
        }
        if let Ok(v) = row.try_get::<bool, _>(idx) {
            return serde_json::Value::Bool(v);
        }
        if let Ok(v) = row.try_get::<String, _>(idx) {
            return serde_json::Value::String(v);
        }
        serde_json::Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_postgres_warehouse_creation() {
        let warehouse = PostgresWarehouse::new("postgres://user:pass@localhost/db");
        assert!(warehouse.pool.read().await.is_none());
    }

    #[tokio::test]
    async fn test_execute_without_connection() {
        let warehouse = PostgresWarehouse::new("postgres://user:pass@localhost/db");
        let result = warehouse.execute("SELECT 1").await;
        assert!(result.is_err());
    }
}
