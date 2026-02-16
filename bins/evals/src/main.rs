use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalDataset {
    pub name: String,
    pub description: String,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub question: String,
    pub golden_sql: String,
    pub expected_result_sample: Option<Vec<serde_json::Value>>,
    pub difficulty: Difficulty,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub test_case_id: String,
    pub passed: bool,
    pub generated_sql: String,
    pub execution_result: ExecResult,
    pub similarity_score: f32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalSummary {
    pub dataset_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f32,
    pub avg_similarity: f32,
    pub results: Vec<EvalResult>,
}

pub fn load_dataset(path: &str) -> Result<EvalDataset, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read dataset: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse dataset: {}", e))
}

pub fn compare_results(golden: &ExecResult, actual: &ExecResult) -> f32 {
    if golden.row_count == 0 && actual.row_count == 0 {
        return 1.0;
    }

    if golden.columns != actual.columns {
        return 0.0;
    }

    let golden_set: HashSet<String> = golden.rows.iter()
        .map(|r| format!("{:?}", r))
        .collect();
    
    let actual_set: HashSet<String> = actual.rows.iter()
        .map(|r| format!("{:?}", r))
        .collect();

    let intersection = golden_set.intersection(&actual_set).count();
    let union = golden_set.union(&actual_set).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f32 / union as f32
}

pub fn sql_similarity(sql1: &str, sql2: &str) -> f32 {
    let normalize = |s: &str| {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
    };
    
    let s1 = normalize(sql1);
    let s2 = normalize(sql2);
    
    if s1 == s2 {
        return 1.0;
    }

    let words1: HashSet<&str> = s1.split_whitespace().collect();
    let words2: HashSet<&str> = s2.split_whitespace().collect();
    
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    
    if union == 0 {
        return 0.0;
    }
    
    intersection as f32 / union as f32
}

pub fn generate_report(summary: &EvalSummary) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("=== Evaluation Report: {} ===\n\n", summary.dataset_name));
    report.push_str(&format!("Total Tests: {}\n", summary.total_tests));
    report.push_str(&format!("Passed: {}\n", summary.passed));
    report.push_str(&format!("Failed: {}\n", summary.failed));
    report.push_str(&format!("Pass Rate: {:.1}%\n\n", summary.pass_rate * 100.0));
    report.push_str(&format!("Average Similarity: {:.2}\n\n", summary.avg_similarity));
    
    report.push_str("=== Failed Tests ===\n");
    for result in &summary.results {
        if !result.passed {
            report.push_str(&format!("\n- {}: {}\n", result.test_case_id, result.error.as_deref().unwrap_or("Failed")));
        }
    }
    
    report
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("QuerySmith Evaluation Harness");
    println!("Usage: evals --dataset <path> --sql <sql-to-test>");
    println!("Or: evals --run-all --dataset <path>");

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("\nExample dataset format:");
        let example = EvalDataset {
            name: "example".to_string(),
            description: "Example dataset".to_string(),
            test_cases: vec![
                TestCase {
                    id: "test_1".to_string(),
                    question: "How many users are there?".to_string(),
                    golden_sql: "SELECT COUNT(*) FROM users".to_string(),
                    expected_result_sample: None,
                    difficulty: Difficulty::Easy,
                    tags: vec!["count".to_string()],
                }
            ],
        };
        println!("{}", serde_json::to_string_pretty(&example).unwrap());
    }

    Ok(())
}
