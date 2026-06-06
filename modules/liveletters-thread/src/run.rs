use std::collections::HashMap;
use std::error::Error;

use liveletters_app_core::{CommentSummary, GetPostThreadQuery, PostThread, get_post_thread};
use liveletters_output::{CommandContext, print_kv};
use liveletters_store::Store;

use crate::Args;
use crate::error::ThreadError;

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), ThreadError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let thread = get_post_thread(
        &store,
        GetPostThreadQuery {
            post_id: &args.post_id,
        },
    )?;
    print_thread(&thread);
    Ok(())
}

pub fn print_thread(thread: &PostThread) {
    let post = thread.post();
    let visibility = if post.visibility.is_empty() {
        "—"
    } else {
        post.visibility.as_str()
    };
    let hidden_marker = if post.hidden { " (скрыт)" } else { "" };

    println!(
        "┌─ пост #{} от {}{hidden_marker}",
        post.post_id, post.author_id
    );
    println!("│  visibility: {visibility}");
    for line in post.body.lines() {
        println!("│  {line}");
    }
    println!("└─");
    println!();

    let comments = thread.comments();
    print_kv(&[("комментарии", &comments.len().to_string())]);

    if comments.is_empty() {
        println!();
        println!("(нет комментариев)");
        return;
    }

    println!();
    let nodes = build_tree(comments);
    for line in render_tree(&nodes, None) {
        println!("{line}");
    }
}

#[derive(Debug, Clone)]
struct Node {
    comment: CommentSummary,
    children: Vec<usize>,
}

fn build_tree(comments: &[CommentSummary]) -> Vec<Node> {
    let mut nodes: Vec<Node> = comments
        .iter()
        .map(|c| Node {
            comment: c.clone(),
            children: Vec::new(),
        })
        .collect();

    let mut by_id: HashMap<&str, usize> = HashMap::new();
    for (i, c) in comments.iter().enumerate() {
        by_id.insert(c.comment_id.as_str(), i);
    }

    let mut roots: Vec<usize> = (0..nodes.len()).collect();
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, c) in comments.iter().enumerate() {
        if let Some(parent_id) = c.parent_comment_id.as_deref()
            && let Some(&parent_idx) = by_id.get(parent_id)
        {
            nodes[parent_idx].children.push(i);
            to_remove.push(i);
        }
    }
    for &i in to_remove.iter().rev() {
        roots.retain(|&r| r != i);
    }
    roots.sort_by_key(|&i| comments[i].created_at);
    let _ = roots;
    let _ = &mut nodes;
    nodes
}

fn render_tree(nodes: &[Node], parent: Option<usize>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let children: Vec<usize> = match parent {
        None => (0..nodes.len())
            .filter(|&i| nodes[i].comment.parent_comment_id.is_none())
            .collect(),
        Some(p) => nodes[p].children.clone(),
    };
    for child in children {
        let c = &nodes[child];
        let prefix = match parent {
            None => "  •".to_owned(),
            Some(_) => "    ↳".to_owned(),
        };
        let hidden_marker = if c.comment.hidden {
            " (скрыт)"
        } else {
            ""
        };
        out.push(format!(
            "{prefix} {} ({}){}",
            c.comment.author_id, c.comment.comment_id, hidden_marker
        ));
        for line in c.comment.body.lines() {
            out.push(format!("        {line}"));
        }
        out.extend(render_tree(nodes, Some(child)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_comment(id: &str, parent: Option<&str>) -> CommentSummary {
        CommentSummary {
            comment_id: id.to_owned(),
            post_id: "post-1".to_owned(),
            parent_comment_id: parent.map(str::to_owned),
            author_id: "user".to_owned(),
            created_at: 0,
            body: "Тело".to_owned(),
            visibility: "public".to_owned(),
            hidden: false,
        }
    }

    #[test]
    fn render_tree_shows_root_and_reply_with_prefix() {
        let comments = vec![make_comment("c1", None), make_comment("c2", Some("c1"))];
        let nodes = build_tree(&comments);
        let lines = render_tree(&nodes, None);
        assert!(lines.iter().any(|l| l.contains("c1")));
        assert!(lines.iter().any(|l| l.contains("c2")));
        assert!(lines.iter().any(|l| l.contains("↳")));
    }

    #[test]
    fn render_tree_with_no_comments_is_empty() {
        let nodes = build_tree(&[]);
        let lines = render_tree(&nodes, None);
        assert!(lines.is_empty());
    }
}
