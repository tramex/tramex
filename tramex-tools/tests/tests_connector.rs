// tests
#[cfg(test)]
mod tests {
    use tramex_tools::connector::Connector;

    #[test]
    fn test_file() {
        let filename = "tests/enb.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        f.try_recv();
        eprint!("{:?}", f.data.events)
        // TODO test value
    }
}
