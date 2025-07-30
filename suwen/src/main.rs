use std::fs::read_to_string;

fn main() {
    let content =
        read_to_string("./记一次对 Rust Embed 压缩的探索.md").expect("Failed to read file");
    let output = suwen_markdown::parse_markdown(&content).expect("Failed to parse markdown");
    std::fs::write("./output.html", output).expect("Failed to write output file");
    println!("Markdown parsed and written to output.html");
}
