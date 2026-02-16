use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "query-smith")]
#[command(version = "0.1.0")]
#[command(about = "QuerySmith - SQL Data Agent CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "postgres://localhost/querysmith")]
    database: String,

    #[arg(short, long, default_value = "minimax-m2.5")]
    model: String,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start interactive REPL mode")]
    Repl,

    #[command(about = "Run a single query")]
    Query {
        #[arg(short, long)]
        question: String,
    },

    #[command(about = "Execute a SQL file")]
    Script {
        #[arg(short, long)]
        file: String,
    },

    #[command(about = "List available tables")]
    Tables,

    #[command(about = "Show table schema")]
    Schema { table: String },

    #[command(about = "Show memory context")]
    Memory {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let agent = Arc::new(agent_core::AgentRuntime::new(
        cli.model,
        agent_core::ToolRegistry::new(),
    ));

    let memory = Arc::new(memory_svc::MemoryService::new());

    match cli.command {
        Commands::Repl => {
            run_repl(agent, memory).await?;
        }
        Commands::Query { question } => {
            println!("Question: {}", question);
            println!("Response: (Connect to LLM for actual response)");
            println!("This is a placeholder - integrate with LLM client for real responses.");
        }
        Commands::Script { file } => {
            println!("Running script: {}", file);
            let content = std::fs::read_to_string(&file)?;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with("--") {
                    println!("Executing: {}", line);
                }
            }
        }
        Commands::Tables => {
            println!("Available tables: (Connect to database to list)");
        }
        Commands::Schema { table } => {
            println!("Schema for {}: (Connect to database to fetch)", table);
        }
        Commands::Memory { limit } => {
            let all_memory = memory.get_all().await?;
            println!("Memory entries (showing up to {}):", limit);
            for (i, mem) in all_memory.iter().take(limit).enumerate() {
                println!(
                    "{}. [{:?}] {}",
                    i + 1,
                    mem.memory_type,
                    &mem.content[..mem.content.len().min(80)]
                );
            }
        }
    }

    Ok(())
}

async fn run_repl(
    _agent: Arc<agent_core::AgentRuntime>,
    memory: Arc<memory_svc::MemoryService>,
) -> anyhow::Result<()> {
    println!("QuerySmith REPL (v0.1.0)");
    println!("Type 'help' for commands, 'exit' to quit\n");

    loop {
        print!("query-smith> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input {
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            "help" => {
                println!("Commands:");
                println!("  help     - Show this help");
                println!("  tables   - List available tables");
                println!("  memory   - Show memory context");
                println!("  clear    - Clear memory");
                println!("  exit     - Exit REPL");
                println!("  <question> - Ask a question");
            }
            "tables" => {
                println!("(Connect to database to list tables)");
            }
            "memory" => {
                let all = memory.get_all().await?;
                println!("Memory entries: {}", all.len());
                for mem in all.iter().take(5) {
                    println!(
                        "  - [{:?}] {}",
                        mem.memory_type,
                        &mem.content[..mem.content.len().min(60)]
                    );
                }
            }
            "clear" => {
                memory.clear(&memory_svc::MemoryScope::global()).await?;
                println!("Memory cleared");
            }
            _ => {
                let user_memory_scope = memory_svc::MemoryScope::user("cli");
                let context = memory
                    .inject_into_prompt(input, Some(user_memory_scope.clone()))
                    .await?;

                println!("Processing: {}", input);
                if !context.is_empty() {
                    println!("Context: {}", context);
                }
                println!("(Connect to LLM for response)");

                let _ = memory
                    .save(memory_svc::Memory::new(
                        user_memory_scope,
                        format!("Q: {}", input),
                        memory_svc::MemoryType::Conversation,
                    ))
                    .await;
            }
        }

        println!();
    }

    Ok(())
}
