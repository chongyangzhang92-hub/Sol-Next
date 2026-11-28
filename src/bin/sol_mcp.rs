//! Sol Next Public Edition — Zero-Dependency Rust 2024 Stdio MCP Server (`sol-mcp`).
//!
//! Speaks Model Context Protocol (`2024-11-05`, JSON-RPC 2.0 over stdin/stdout) directly
//! in-process for Claude Code, Claude Desktop, Cursor, Codex, and Antigravity.

use sol_next_tools::handle_mcp_line;
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line_res in stdin.lock().lines() {
        let line = line_res?;
        if let Some(resp) = handle_mcp_line(&line) {
            writeln!(stdout, "{resp}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}
