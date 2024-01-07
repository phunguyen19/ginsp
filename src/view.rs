use crate::commands::CommitInfo;

/// Print result as table like this
/// ```
/// Commit messages unique on branch:
/// ------------------------
///     eec4f1c - [ABC-10370] message
///     54912eb - [ABC-10365] message
/// ```
pub fn print_result(
    branch: &str,
    unique_commits: Vec<CommitInfo>,
) {
    println!("\nCommit messages unique on {}:", branch);
    println!("------------------------");
    for item in unique_commits {
        let CommitInfo { hash, message , status} = item;

        if status.is_none() {
            println!("    {} - {}", hash, message);
        } else {
            println!("    {} - {} - {}", hash, status.unwrap_or_default(), message);
        }
    }
}
