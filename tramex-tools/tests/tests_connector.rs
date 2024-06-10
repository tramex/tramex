// tests
#[cfg(test)]
mod tests {
    use std::path::Path;

    use connector::Connector;
    use tramex_tools::{
        connector,
        data::AdditionalInfos,
        errors::TramexError,
        interface::{
            interface_types::Interface,
            layer::{Layer, Layers},
            types::Direction,
        },
    };

    fn get_path(p: &str) -> String {
        if std::env::current_dir().unwrap().ends_with("tramex-tools") {
            let filename = Path::new("tests").join(p).to_string_lossy().to_string();
            eprintln!("{:?}", filename);
            return filename;
        }
        let filename = file!();
        let filename = Path::new(filename).parent().unwrap().join(p).to_string_lossy().to_string();
        eprintln!("{:?}", filename);
        return filename;
    }

    #[test]
    #[allow(unreachable_patterns)]
    fn test_file() {
        let filename = &get_path("enb.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        match &mut f.interface {
            Some(Interface::File(file)) => {
                file.change_nb_read(50);
            }
            _ => {
                unreachable!();
            }
        }
        let res = f.get_more_data(Layers::all());
        eprintln!("result {:?}", res);
        eprintln!("count {:?}", f.data.events.len());
        assert!(f.data.events.len() == 15);
        let one_trace = f.data.events[0].clone();
        let infos = match one_trace.additional_infos {
            AdditionalInfos::RRCInfos(infos) => infos,
            _ => unreachable!(),
        };
        assert!(infos.direction == Direction::DL);
        assert!(infos.canal == "BCCH");
        assert!(infos.canal_msg == "SIB");
        assert!(one_trace.layer == Layer::RRC);
        eprintln!("{:?}", one_trace.timestamp);
        assert!(one_trace.timestamp == 39668348);
        let one_trace = f.data.events[1].clone();
        let infos = match one_trace.additional_infos {
            AdditionalInfos::RRCInfos(infos) => infos,
            _ => unreachable!(),
        };
        eprintln!("{:?}", one_trace.timestamp);
        assert!(one_trace.timestamp == 39668353);
        assert!(one_trace.layer == Layer::RRC);
        assert!(infos.canal == "BCCH");
        assert!(infos.canal_msg == "SIB1");
        assert!(infos.direction == Direction::DL);
    }

    #[test]
    fn test_jsonlike() {
        let filename = &get_path("enb_jsonlike_error.log");
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
        let filename = &get_path("enb_canal_or_canal_message_malformed.log");
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
        let filename = &get_path("enb_date_err.log");
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

    #[test]
    fn test_error_date_full_file() {
        let filename = &get_path("enb_date_err.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        let mut errors: Vec<TramexError> = vec![];
        let mut last_size_data = 0;
        let mut last_size_errors = 0;
        loop {
            match f.get_more_data(Layers::all()) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{:?}", e);
                    errors.push(e);
                }
            }
            if f.data.events.len() == last_size_data && errors.len() == last_size_errors {
                break;
            } else {
                last_size_data = f.data.events.len();
                last_size_errors = errors.len();
                eprintln!("data: {:?}", f.data.events.len());
                eprintln!("errors: {:?}", errors.len());
            }
        }
        eprintln!("data: {:?}", f.data.events.len());
        eprintln!("errors: {:?}", errors.len());
        assert!(f.data.events.len() == 0);
        assert!(errors.len() == 1);
        assert!(errors[0].message.contains("Error while parsing date"));
    }

    #[test]
    fn test_other_file() {
        let filename = &get_path("enb0.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let mut f = Connector::new_file_content(filename.into(), content);
        let mut errors: Vec<TramexError> = vec![];
        let mut last_size_data = 0;
        let mut last_size_errors = 0;
        loop {
            match f.get_more_data(Layers::all()) {
                Ok(_) => {}
                Err(e) => {
                    errors.push(e);
                }
            }
            if f.data.events.len() == last_size_data && errors.len() == last_size_errors {
                break;
            } else {
                last_size_data = f.data.events.len();
                last_size_errors = errors.len();
            }
        }
        let num_error_direction = 5;
        let number_rrc = 53;
        let total = 11769;
        let count_events = number_rrc - num_error_direction;
        let count_errors = total - number_rrc + num_error_direction;
        eprintln!("data: {:?}", f.data.events.len());
        eprintln!("count_events: {:?}", count_events);
        eprintln!("errors: {:?}", errors.len());
        eprintln!("count_errors: {:?}", count_errors);
        eprintln!("{:?}", errors.last());
        assert!(f.data.events.len() == count_events);
        assert!(errors.len() == count_errors);
        assert!(errors.last().unwrap().message.contains("Unknown message type"));
    }
}
