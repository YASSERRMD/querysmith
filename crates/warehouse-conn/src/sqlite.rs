use async_trait::async_trait;
use sqlx::{
    sqlite::{SqlitePool, SqliteRow},
    Column, Row, TypeInfo,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::traits::{Column as TableColumn, Error, QueryResult, TableSchema, Warehouse};

pub struct SqliteWarehouse {
    pool: Arc<RwLock<Option<SqlitePool>>>,
    connection_string: String,
}

impl SqliteWarehouse {
    pub fn new(connection_string: &str) -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            connection_string: connection_string.to_string(),
        }
    }

    async fn get_pool(&self) -> Result<SqlitePool, Error> {
        let guard = self.pool.read().await;
        guard
            .clone()
            .ok_or_else(|| Error::Connection("Not connected".to_string()))
    }
}

#[async_trait]
impl Warehouse for SqliteWarehouse {
    async fn connect(&self) -> Result<(), Error> {
        let pool = SqlitePool::connect(&self.connection_string)
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

        let sql_upper = sql.trim().to_uppercase();
        if sql_upper.starts_with("SELECT") || sql_upper.starts_with("PRAGMA") {
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

        let columns_sql = format!("PRAGMA table_info('{}')", table_name);

        let columns: Vec<TableColumn> = sqlx::query(&columns_sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| Error::Query(e.to_string()))?
            .iter()
            .map(|row| TableColumn {
                name: row.get(1),
                data_type: row.get(2),
                nullable: row.get::<i32, _>(3) == 0,
                comment: None,
            })
            .collect();

        if columns.is_empty() {
            return Err(Error::Query(format!("Table '{}' not found", table_name)));
        }

        Ok(TableSchema {
            name: table_name.to_string(),
            columns,
            primary_key: None,
        })
    }

    async fn list_tables(&self) -> Result<Vec<String>, Error> {
        let pool = self.get_pool().await?;

        let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name";

        let rows = sqlx::query(sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| Error::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get::<String, _>(0)).collect())
    }

    async fn preview_table(&self, table_name: &str, limit: usize) -> Result<QueryResult, Error> {
        let sql = format!("SELECT * FROM {} LIMIT {}", table_name, limit);
        self.execute(&sql).await
    }
}

impl SqliteWarehouse {
    fn map_value(row: &SqliteRow, idx: usize, _type_name: &str) -> serde_json::Value {
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
    async fn test_sqlite_warehouse_creation() {
        let warehouse = SqliteWarehouse::new(":memory:");
        assert!(warehouse.pool.read().await.is_none());
    }
}
