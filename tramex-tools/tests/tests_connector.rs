// tests
#[cfg(test)]
mod tests {
    use connector::Connector;
    use tramex_tools::{connector, interface::layer::Layer, interface::types::Direction};

    #[test]
    fn test_file() {
        let filename = "tests/enb.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        let _ = f.try_recv();
        assert!(f.data.events.len() == 15);
        let event = f.data.events.pop().unwrap();
        assert!(event.trace_type.direction == Direction::DL);
        assert!(event.trace_type.canal == "BCCH");
        assert!(event.trace_type.canal_msg == "SIB");
        assert!(event.trace_type.layer == Layer::RRC);
        assert!(event.trace_type.timestamp == 39668668);
        assert!(f.data.events.len() == 14);
        let f_event = &f.data.events[0];
        assert!(f_event.trace_type.timestamp == 39668348);
        assert!(f_event.trace_type.layer == Layer::RRC);
        assert!(f_event.trace_type.canal == "BCCH");
        assert!(f_event.trace_type.canal_msg == "SIB");
        assert!(f_event.trace_type.direction == Direction::DL);
    }

    #[test]
    fn test_jsonlike() {
        let filename = "tests/enb_jsonlike_error.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        match f.try_recv() {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                assert!(e.message == "Could not parse the JSON like part, missing closing }");
            }
        }
    }
    #[test]
    fn test_malformed_fl() {
        let filename = "tests/enb_canal_or_canal_message_malformed.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        match f.try_recv() {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                assert!(e.message == "The canal and/or canal message could not be parsed");
            }
        }
    }
    #[test]
    fn test_error_date() {
        let filename = "tests/enb_date_err.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        match f.try_recv() {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                assert!(e.message == "Error while parsing date");
            }
        }
    }
}
