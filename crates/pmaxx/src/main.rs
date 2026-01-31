use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        list_prompts();
        return;
    }

    match args[0].as_str() {
        "list" | "l" => list_prompts(),
        "last" => show_last(),
        "count" | "c" => show_count(),
        _ => {
            let prompt = args.join(" ");
            suggest(&prompt);
        }
    }
}

fn list_prompts() {
    let prompts = promptmaxx::list().unwrap_or_default();
    for p in prompts {
        println!("{}", p.text);
        println!("---");
    }
}

fn show_last() {
    let prompts = promptmaxx::list().unwrap_or_default();
    if let Some(p) = prompts.first() {
        println!("{}", p.text);
    }
}

fn show_count() {
    let count = promptmaxx::count().unwrap_or(0);
    println!("{}", count);
}

fn suggest(prompt: &str) {
    let saved = promptmaxx::list().unwrap_or_default();
    let patterns: Vec<&str> = saved.iter().map(|p| p.text.as_str()).take(10).collect();

    let full_prompt = format!(
        r#"Rewrite this vague prompt into a specific, actionable one.

User's style (from their saved prompts):
{}

Vague prompt: {}

Output ONLY the better prompt, one line, nothing else:"#,
        patterns.join("\n"),
        prompt
    );

    match Command::new("claude").args(["-p", &full_prompt]).output() {
        Ok(o) if o.status.success() => {
            let result = String::from_utf8_lossy(&o.stdout);
            println!("→ {}", result.trim());
        }
        _ => println!("→ What specifically: {}?", prompt),
    }
}
