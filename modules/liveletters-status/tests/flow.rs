use liveletters_output::CommandContext;
use liveletters_status::{Args, StatusCounts, print_status};

mod common;

#[test]
fn print_status_prints_zero_counts_on_empty() {
    let counts = StatusCounts {
        posts: 0,
        comments: 0,
        deferred: 0,
        outbox: 0,
        last_activity: None,
    };
    print_status(&counts);
}

#[test]
fn print_status_includes_last_activity() {
    let counts = StatusCounts {
        posts: 1,
        comments: 0,
        deferred: 0,
        outbox: 0,
        last_activity: Some(1_700_000_000),
    };
    print_status(&counts);
}

#[test]
fn run_uses_real_store() {
    let tmp = tempfile::tempdir().unwrap();
    let ctx = CommandContext {
        home: tmp.path().to_path_buf(),
        state_home: tmp.path().to_path_buf(),
        identity_name: "default".to_owned(),
    };
    let _ = liveletters_store::Store::open_for_home_dir(&ctx.state_home).unwrap();
    liveletters_status::run(&ctx, &Args {}).unwrap();
}

#[test]
fn run_reports_inserted_posts() {
    let (store, tmp) = common::open_temp_store();
    store
        .save_post_record(&liveletters_store::PostRecord {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            author_id: "alice".into(),
            created_at: 1_700_000_000,
            body: "тело".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    let ctx = CommandContext {
        home: tmp.path().to_path_buf(),
        state_home: tmp.path().to_path_buf(),
        identity_name: "default".to_owned(),
    };
    liveletters_status::run(&ctx, &Args {}).unwrap();
}
