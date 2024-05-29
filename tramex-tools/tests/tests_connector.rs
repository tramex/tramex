// tests
#[cfg(test)]
mod tests {
    use connector::Connector;
    use tramex_tools::{
        connector,
        websocket::{self, layer::Layer},
    };

    #[test]
    fn test_file() {
        let filename = "tests/enb.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        if let Err(err) = f.try_recv() {
            eprint!("{:?}", err);
            assert!(false);
        }
        if let Err(err) = f.try_recv() {
            eprint!("{:?}", err);
            assert!(false);
        }
        if let Err(err) = f.try_recv() {
            eprint!("{:?}", err);
            assert!(false);
        }
        eprintln!("{:?}", f.data.events.len());
        assert!(f.data.events.len() == 15);
        let event = f.data.events.pop().unwrap();
        assert!(event.trace_type.direction == websocket::types::Direction::DL);
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
        assert!(f_event.trace_type.direction == websocket::types::Direction::DL);
    }
}
