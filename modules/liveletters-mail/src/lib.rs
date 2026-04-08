pub fn crate_name() -> &'static str {
    "liveletters-mail"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-mail");
    }
}

