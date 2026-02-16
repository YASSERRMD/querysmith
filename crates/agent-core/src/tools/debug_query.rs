use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::traits::{Tool, ToolParameters, ToolProperty, ToolResult};

pub struct DebugQueryTool;

impl DebugQueryTool {
    pub fn new() -> Self {
        Self
    }

    fn analyze_error(&self, sql: &str, error: &str) -> String {
        let mut suggestions = Vec::new();
        let error_lower = error.to_lowercase();

        if error_lower.contains("syntax") {
            suggestions.push("Check for typos in SQL keywords".to_string());
            suggestions.push("Verify proper use of quotes and parentheses".to_string());
            suggestions.push("Ensure proper table and column names".to_string());
        }

        if error_lower.contains("relation") || error_lower.contains("table") {
            suggestions.push("Verify the table exists".to_string());
            suggestions.push("Check if table name is spelled correctly".to_string());
            suggestions.push("Ensure proper schema prefix if required".to_string());
        }

        if error_lower.contains("column") {
            suggestions.push("Verify column names are correct".to_string());
            suggestions.push("Check for case sensitivity issues".to_string());
            suggestions.push("Ensure the column exists in the referenced table".to_string());
        }

        if error_lower.contains("permission") || error_lower.contains("denied") {
            suggestions.push("Check user permissions for this operation".to_string());
        }

        if error_lower.contains("timeout") || error_lower.contains("canceled") {
            suggestions.push("Query may be taking too long - consider adding limits".to_string());
            suggestions.push("Check for missing indexes on join columns".to_string());
        }

        format!(
            "Analysis of SQL Error:\n\nOriginal Error: {}\n\nSQL: {}\n\nSuggestions:\n{}",
            error,
            sql,
            suggestions
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl Default for DebugQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for DebugQueryTool {
    fn name(&self) -> &str {
        "debug_query"
    }

    fn description(&self) -> &str {
        "Analyze a failed SQL query and provide suggestions for fixing it."
    }

    fn parameters(&self) -> ToolParameters {
        let mut props = HashMap::new();
        props.insert(
            "sql".to_string(),
            ToolProperty {
                prop_type: "string".to_string(),
                description: "SQL query that failed".to_string(),
            },
        );
        props.insert(
            "error".to_string(),
            ToolProperty {
                prop_type: "string".to_string(),
                description: "Error message from the failed query".to_string(),
            },
        );
        ToolParameters {
            param_type: "object".to_string(),
            properties: props,
            required: vec!["sql".to_string(), "error".to_string()],
        }
    }

    fn execute(&self, params: HashMap<String, serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<ToolResult, String>> + Send>> {
        Box::pin(async move {
            let sql = params
                .get("sql")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: sql")?;

            let error = params
                .get("error")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: error")?;

            let analysis = Self.analyze_error(&sql, &error);

            Ok(ToolResult::success(serde_json::json!({
                "analysis": analysis,
                "suggestions": analysis
            })))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_query_tool() {
        let tool = DebugQueryTool::new();
        
        let sql = "SELECT * FORM users";
        let error = "syntax error at or near 'FORM'";
        
        let analysis = tool.analyze_error(sql, error);
        assert!(analysis.contains("syntax"));
    }
}
