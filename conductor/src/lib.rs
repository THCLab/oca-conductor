#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result =
            zip_resolver::resolve_from_zip("/tmp/Downloads/82cc6d6fb50fb4dd1bbf1b67f48c7cd7.zip");
        assert_eq!(result.len(), 4);
    }
}
