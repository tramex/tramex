// tests
#[cfg(test)]
mod tests {
    use connector::Connector;
    use tramex_tools::{connector, types};

    #[test]
    fn test_file() {
        let filename = "tests/enb.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        f.try_recv();
        //eprint!("{:?}", f.data.events)
        assert!(f.data.events.len() == 15);
        let event = f.data.events.pop().unwrap();
        assert!(event.trace_type.direction == types::websocket_types::Direction::DL);
        assert!(event.trace_type.canal == "BCCH");
        assert!(event.trace_type.canal_msg == "SIB");
        assert!(event.trace_type.msgtype == "RRC");
        assert!(event.trace_type.timestamp == "11:01:08.668");
        assert!(f.data.events.len() == 14);
        let f_event = &f.data.events[0];
        assert!(f_event.trace_type.timestamp == "11:01:08.348");
        assert!(f_event.trace_type.msgtype == "RRC");
        assert!(f_event.trace_type.canal == "BCCH");
        assert!(f_event.trace_type.canal_msg == "SIB");
        assert!(f_event.trace_type.direction == types::websocket_types::Direction::DL);
    }
}
