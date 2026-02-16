use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::traits::{Tool, ToolParameters, ToolResult};
use warehouse_conn::{PostgresWarehouse, SqliteWarehouse, Warehouse};

pub struct RunSqlTool {
    warehouse: Arc<dyn Warehouse>,
}

impl RunSqlTool {
    pub fn new_postgres(connection_string: &str) -> Self {
        let warehouse = PostgresWarehouse::new(connection_string);
        Self {
            warehouse: Arc::new(warehouse),
        }
    }

    pub fn new_sqlite(connection_string: &str) -> Self {
        let warehouse = SqliteWarehouse::new(connection_string);
        Self {
            warehouse: Arc::new(warehouse),
        }
    }

    pub async fn execute_query(&self, sql: &str) -> Result<ToolResult, String> {
        match self.warehouse.execute(sql).await {
            Ok(result) => Ok(ToolResult::success(serde_json::json!({
                "columns": result.columns,
                "rows": result.rows,
                "row_count": result.row_count
            }))),
            Err(e) => Ok(ToolResult::error(e.to_string())),
        }
    }
}

impl Tool for RunSqlTool {
    fn name(&self) -> &str {
        "run_sql"
    }

    fn description(&self) -> &str {
        "Execute a SQL query against the database and return results."
    }

    fn parameters(&self) -> ToolParameters {
        let mut props = HashMap::new();
        props.insert(
            "sql".to_string(),
            crate::traits::ToolProperty {
                prop_type: "string".to_string(),
                description: "SQL query to execute".to_string(),
            },
        );
        ToolParameters {
            param_type: "object".to_string(),
            properties: props,
            required: vec!["sql".to_string()],
        }
    }

    fn execute(&self, params: HashMap<String, serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<ToolResult, String>> + Send>> {
        let warehouse = self.warehouse.clone();
        let sql = params
            .get("sql")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Box::pin(async move {
            let sql = sql.ok_or("Missing required parameter: sql")?;
            match warehouse.execute(&sql).await {
                Ok(result) => Ok(ToolResult::success(serde_json::json!({
                    "columns": result.columns,
                    "rows": result.rows,
                    "row_count": result.row_count
                }))),
                Err(e) => Ok(ToolResult::error(e.to_string())),
            }
        })
    }
}
