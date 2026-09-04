//! Formats html for display in the console

pub fn show(body: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for c in body.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            ' ' if output.ends_with(" ") => (),
            _ if !in_tag => output.push(c),
            _ => (),
        }
    }
    output
}
