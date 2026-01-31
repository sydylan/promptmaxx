use promptmaxx_core::{db, init_db};
use serde::{Deserialize, Serialize};
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        list_prompts();
        return;
    }

    let first = args[0].as_str();
    let rest: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();

    match first {
        "list" | "l" => list_prompts(),
        "last" => show_last(),
        "count" | "c" => show_count(),
        "suggest" | "s" => suggest(&rest.join(" ")),
        _ => {
            // Treat entire input as prompt to suggest
            let full = args.join(" ");
            suggest(&full);
        }
    }
}

fn list_prompts() {
    let db = init_db().unwrap();
    let prompts = db::get_prompts(&db, None).unwrap_or_default();
    for p in prompts {
        println!("{}", p.text);
        println!("---");
    }
}

fn show_last() {
    let db = init_db().unwrap();
    let prompts = db::get_prompts(&db, None).unwrap_or_default();
    if let Some(p) = prompts.first() {
        println!("{}", p.text);
    }
}

fn show_count() {
    let db = init_db().unwrap();
    let count = db::get_prompt_count(&db).unwrap_or(0);
    println!("{}", count);
}

fn suggest(prompt: &str) {
    if prompt.trim().is_empty() {
        eprintln!("Usage: pmaxx <your vague prompt>");
        return;
    }

    // Get saved prompts for context
    let db = init_db().unwrap();
    let saved = db::get_prompts(&db, None).unwrap_or_default();

    let patterns: Vec<&str> = saved.iter().map(|p| p.text.as_str()).take(10).collect();

    // Build prompt for Ollama
    let system = format!(
        r#"You rewrite vague prompts into specific, actionable ones.

The user's best prompts (learn their style):
{}

Rules:
- Output ONLY the better prompt, nothing else
- Keep it concise
- Make it specific and actionable
- Match their style"#,
        patterns.join("\n---\n")
    );

    let full_prompt = format!("{}\n\nRewrite this vague prompt:\n{}\n\nBetter prompt:", system, prompt);

    match call_ollama(&full_prompt) {
        Ok(better) => println!("→ {}", better.trim()),
        Err(_) => {
            // Fallback: simple heuristic
            println!("→ What specific problem are you trying to solve with: {}?", prompt);
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

fn call_ollama(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let req = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };

    let resp = client
        .post("http://localhost:11434/api/generate")
        .json(&req)
        .timeout(std::time::Duration::from_secs(30))
        .send()?;

    let result: OllamaResponse = resp.json()?;
    Ok(result.response)
}
