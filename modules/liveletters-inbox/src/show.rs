use std::path::Path;

use liveletters_store::Store;

use crate::args::ShowArgs;
use crate::error::InboxError;

pub fn run(home: &Path, args: &ShowArgs) -> Result<(), InboxError> {
    let store = Store::open_for_home_dir(home)?;
    let rec = store
        .get_raw_message_record(&args.id)?
        .ok_or_else(|| InboxError::MessageNotFound(args.id.clone()))?;
    print_message(&rec.message_id, &rec.status, &rec.raw_message);
    Ok(())
}

fn print_message(message_id: &str, status: &str, raw: &str) {
    println!("message_id: {message_id}");
    println!("status: {status}");
    println!();
    println!("--- body ---");
    println!("{raw}");
}
