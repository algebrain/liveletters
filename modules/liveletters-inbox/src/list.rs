use std::path::Path;

use liveletters_output::{print_kv, print_table};
use liveletters_store::Store;

use crate::args::ListArgs;
use crate::error::InboxError;

const ALLOWED_STATUSES: &[&str] = &[
    "applied",
    "duplicate",
    "replay",
    "unauthorized",
    "invalid",
    "malformed",
];

pub fn run(home: &Path, args: &ListArgs) -> Result<(), InboxError> {
    if let Some(s) = &args.status
        && !ALLOWED_STATUSES.contains(&s.as_str())
    {
        return Err(InboxError::InvalidStatus(s.clone()));
    }
    let store = Store::open_for_home_dir(home)?;
    let all_count = store.list_raw_message_records()?.len();
    let shown = store.list_raw_message_records_paged(args.status.as_deref(), args.limit)?;

    print_kv(&[("входящих всего", &all_count.to_string())]);
    if let Some(s) = &args.status {
        print_kv(&[("фильтр", s)]);
    }
    print_kv(&[("показано", &shown.len().to_string())]);
    println!();

    if shown.is_empty() {
        println!("(пусто)");
        return Ok(());
    }

    let headers = &["message_id", "status", "preview"];
    let rows: Vec<Vec<String>> = shown
        .iter()
        .map(|r| {
            vec![
                r.message_id.clone(),
                r.status.clone(),
                preview_of(&r.raw_message),
            ]
        })
        .collect();
    print_table(headers, &rows);
    Ok(())
}

fn preview_of(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| {
            if l.chars().count() > 80 {
                let mut out: String = l.chars().take(80).collect();
                out.push('…');
                out
            } else {
                l.to_owned()
            }
        })
        .unwrap_or_default()
}
