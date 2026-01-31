use promptmaxx_core::{db, init_db};
use std::env;

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
        _ => list_prompts(),
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
