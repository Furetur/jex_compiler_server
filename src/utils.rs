use random_string::generate;

pub fn random_string(length: usize) -> String {
    let charset = "1234567890abcef";
    generate(length, charset)
}
