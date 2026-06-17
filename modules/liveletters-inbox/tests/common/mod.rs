use std::path::{Path, PathBuf};

use liveletters_store::Store;

#[allow(dead_code)]
pub fn open_temp_store() -> (Store, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    (store, tmp)
}

/// Пишет в `home/<file>.eml` валидное протокольное сообщение с заданным
/// телом поста; возвращает путь к файлу.
#[allow(dead_code)]
pub fn write_valid_post_eml(home: &Path, body: &str) -> PathBuf {
    let path = home.join("post.eml");
    let eml = build_post_eml(body);
    std::fs::write(&path, eml).unwrap();
    path
}

#[allow(dead_code)]
fn build_post_eml(body: &str) -> String {
    format!(
        "From: alice <alice@example.test>\n\
         To: bob-feed@example.test\n\
         Subject: post-1\n\
         MIME-Version: 1.0\n\
         Content-Type: multipart/mixed; boundary=\"liveletters-boundary\"\n\
         \n\
         --liveletters-boundary\n\
         Content-Type: text/plain; charset=\"utf-8\"\n\
         \n\
         {body}\n\
         --liveletters-boundary\n\
         Content-Type: application/json\n\
         \n\
         {{\n\
         \x20\x20\"envelope\": {{\n\
         \x20\x20\x20\x20\"schema_version\": \"1\",\n\
         \x20\x20\x20\x20\"event_type\": \"post_created\",\n\
         \x20\x20\x20\x20\"resource_id\": \"blog-1\",\n\
         \x20\x20\x20\x20\"event_id\": \"post-1\"\n\
         \x20\x20}},\n\
         \x20\x20\"human_readable_body\": \"{body}\",\n\
         \x20\x20\"origin\": \"Alice <alice@example.test>\",\n\
         \x20\x20\"payload\": {{\n\
         \x20\x20\x20\x20\"kind\": \"post_created\",\n\
         \x20\x20\x20\x20\"post_id\": \"post-1\",\n\
         \x20\x20\x20\x20\"resource_id\": \"blog-1\",\n\
         \x20\x20\x20\x20\"actor_id\": \"alice\",\n\
         \x20\x20\x20\x20\"created_at\": 1710000000,\n\
         \x20\x20\x20\x20\"visibility\": \"public\",\n\
         \x20\x20\x20\x20\"body_format\": \"plain\",\n\
         \x20\x20\x20\x20\"body\": \"{body}\"\n\
         \x20\x20}}\n\
         }}\n\
         --liveletters-boundary--\n"
    )
}
