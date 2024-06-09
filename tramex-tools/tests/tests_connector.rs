// tests
#[cfg(test)]
mod tests {
    use connector::Connector;
    use tramex_tools::{
        connector,
        data::AdditionalInfos,
        interface::{
            layer::{Layer, Layers},
            types::Direction,
        },
    };

    #[test]
    #[allow(unreachable_patterns)]
    fn test_file() {
        let filename = "tests/enb.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        let _ = f.get_more_data(Layers::all());
        assert!(f.data.events.len() == 15);
        let one_trace = f.data.events.pop().unwrap();
        let infos = match one_trace.additional_infos {
            AdditionalInfos::RRCInfos(infos) => infos,
            _ => unreachable!(),
        };
        assert!(infos.direction == Direction::DL);
        assert!(infos.canal == "BCCH");
        assert!(infos.canal_msg == "SIB");
        assert!(one_trace.layer == Layer::RRC);
        assert!(one_trace.timestamp == 39668668);
        assert!(f.data.events.len() == 14);
        let one_trace = f.data.events[0].clone();
        let infos = match one_trace.additional_infos {
            AdditionalInfos::RRCInfos(infos) => infos,
            _ => unreachable!(),
        };
        assert!(one_trace.timestamp == 39668348);
        assert!(one_trace.layer == Layer::RRC);
        assert!(infos.canal == "BCCH");
        assert!(infos.canal_msg == "SIB");
        assert!(infos.direction == Direction::DL);
    }

    #[test]
    fn test_jsonlike() {
        let filename = "tests/enb_jsonlike_error.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        match f.get_more_data(Layers::all()) {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                eprintln!("{:?}", e);
                assert!(e.message.contains("Could not parse the JSON like part, missing closing }"));
            }
        }
    }
    #[test]
    fn test_malformed_fl() {
        let filename = "tests/enb_canal_or_canal_message_malformed.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        match f.get_more_data(Layers::all()) {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                eprintln!("{:?}", e);
                assert!(e.message.contains("The canal and/or canal message could not be parsed"));
            }
        }
    }
    #[test]
    fn test_error_date() {
        let filename = "tests/enb_date_err.log";
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        match f.get_more_data(Layers::all()) {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                eprintln!("{:?}", e);
                assert!(e.message.contains("Error while parsing date"));
            }
        }
    }
}
