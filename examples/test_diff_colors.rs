use colored::*;

fn main() {
    let original = r#"fn main() {
    let x = 1;
    println!("x = {}", x);
    println!("x = {}", x);
}"#;

    let modified = r#"fn main() {
    let x = 42;
    println!("x = {}", x);
    println!("x = {}", x);
}"#;

    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(original, modified);

    println!("{}", "📋 即将应用以下修改:".bright_cyan().bold());
    println!();

    for ops in diff.grouped_ops(3) {
        for op in ops {
            for change in diff.iter_changes(&op) {
                match change.tag() {
                    ChangeTag::Equal => {
                        print!(" {}", change.value().dimmed());
                    }
                    ChangeTag::Delete => {
                        print!("{}{}", "-".red(), change.value().red());
                    }
                    ChangeTag::Insert => {
                        print!("{}{}", "+".green(), change.value().green());
                    }
                }
            }
        }
    }
    println!();
}
