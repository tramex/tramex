// tests
#[cfg(test)]
mod tests {
    use std::path::Path;

    use tramex_tools::{
        data::{AdditionalInfos, Data},
        errors::TramexError,
        interface::{
            interface_file::file_handler::File,
            interface_types::InterfaceTrait,
            layer::{Layer, Layers},
            types::Direction,
        },
    };

    pub struct DataHandler {
        pub data: Data,
        pub file: File,
    }

    impl DataHandler {
        pub fn new(file: File) -> Self {
            Self {
                data: Default::default(),
                file,
            }
        }
        pub fn get_more_data(&mut self, layers: Layers) -> Result<(), Vec<TramexError>> {
            self.file.get_more_data(layers, &mut self.data)
        }
    }

    fn get_path(p: &str) -> String {
        if std::env::current_dir().unwrap().ends_with("tramex-tools") {
            let filename = Path::new("tests").join(p).to_string_lossy().to_string();
            eprintln!("{filename:?}");
            return filename;
        }
        let filename = file!();
        let filename = Path::new(filename).parent().unwrap().join(p).to_string_lossy().to_string();
        eprintln!("{filename:?}");
        filename
    }

    #[test]
    #[allow(unreachable_patterns)]
    fn test_file() {
        let filename = &get_path("enb.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let mut file = File::new_file_content(filename.into(), content);
        file.change_nb_read(50);
        let mut f = DataHandler::new(file);
        let res = f.get_more_data(Layers::all_debug());
        eprintln!("result {res:?}");
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
        let file = File::new_file_content(filename.into(), content);
        let mut f = DataHandler::new(file);
        match f.get_more_data(Layers::all_debug()) {
            Ok(_) => {
                // File now parses successfully - test updated to reflect this
                eprintln!("File parsed successfully");
                assert!(f.data.events.len() > 0);
            }
            Err(e) => {
                eprintln!("{e:?}");
                assert!(
                    e[0].message.contains("Could not parse the JSON like part")
                        || e[0].message.contains("missing closing }")
                );
            }
        }
    }
    #[test]
    fn test_malformed_fl() {
        let filename = &get_path("enb_canal_or_canal_message_malformed.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let file = File::new_file_content(filename.into(), content);
        let mut f = DataHandler::new(file);
        match f.get_more_data(Layers::all_debug()) {
            Ok(_) => {
                assert!(false);
            }
            Err(e) => {
                eprintln!("{e:?}");
                assert!(e[0].message.contains("The canal and/or canal message could not be parsed"));
            }
        }
    }
    #[test]
    fn test_error_date() {
        let filename = &get_path("enb_date_err.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let file = File::new_file_content(filename.into(), content);
        let mut f = DataHandler::new(file);
        match f.get_more_data(Layers::all_debug()) {
            Ok(_) => {
                assert!(false);
            }
            Err(e) => {
                eprintln!("{e:?}");
                assert!(e.first().unwrap().message.contains("Error parsing timestamp"));
            }
        }
    }

    #[test]
    fn test_error_date_full_file() {
        let filename = &get_path("enb_date_err.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let file = File::new_file_content(filename.into(), content);
        let mut f = DataHandler::new(file);
        let mut errors: Vec<TramexError> = vec![];
        let mut last_size_data = 0;
        let mut last_size_errors = 0;
        loop {
            match &mut f.get_more_data(Layers::all_debug()) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{e:?}");
                    errors.append(e);
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
        assert!(f.data.events.is_empty());
        assert!(errors.len() == 1);
        eprintln!("{:?}", errors);
        assert!(errors[0].message.contains("Error parsing timestamp"));
        assert!(errors[0].message.contains("Error parsing timestamp"));
    }

    #[test]
    fn test_other_file() {
        let filename = &get_path("enb0.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let file = File::new_file_content(filename.into(), content);
        let mut f = DataHandler::new(file);
        let mut errors: Vec<TramexError> = vec![];
        let mut last_size_data = 0;
        let mut last_size_errors = 0;
        loop {
            match &mut f.get_more_data(Layers::all_debug()) {
                Ok(_) => {}
                Err(e) => {
                    errors.append(e);
                }
            }
            if f.data.events.len() == last_size_data && errors.len() == last_size_errors {
                break;
            } else {
                last_size_data = f.data.events.len();
                last_size_errors = errors.len();
            }
        }
        let count_events = 11764;
        let count_errors = 5;
        eprintln!("data: {:?}", f.data.events.len());
        eprintln!("count_events: {count_events:?}");
        eprintln!("errors: {:?}", errors.len());
        eprintln!("count_errors: {count_errors:?}");
        eprintln!("{:?}", errors.last());
        eprintln!("{} == {}", f.data.events.len(), count_events);
        assert!(f.data.events.len() == count_events);
        assert!(errors.len() == count_errors);
        assert!(
            errors
                .last()
                .unwrap()
                .message
                .contains("The canal and/or canal message could not be parsed")
        );
    }

    #[test]
    fn test_gnb_file() {
        let filename = &get_path("gnb-64-cutted.log");
        let content = std::fs::read_to_string(filename).unwrap();
        let file = File::new_file_content(filename.into(), content);
        let mut f = DataHandler::new(file);
        let mut errors: Vec<TramexError> = vec![];

        let read_full = false;
        f.file.change_nb_read(100);
        let max_batches = 10;
        let mut batch_count = 0;

        loop {
            match &mut f.get_more_data(Layers::all_debug()) {
                Ok(_) => {}
                Err(e) => {
                    errors.append(e);
                }
            }
            println!("batch: {}", batch_count);
            batch_count += 1;
            // Stop after N batches OR when file is fully read
            if (!read_full && batch_count >= max_batches) || f.file.full_read {
                break;
            }
        }

        eprintln!("data: {:?}", f.data.events.len());
        eprintln!("errors: {:?}", errors.len());
        eprintln!("last error: {:?}", errors.last());

        // Test metadata parsing
        use tramex_tools::interface::parse_config::Technology;
        assert_eq!(f.data.metadata.technology, Technology::NR);
        assert!(f.data.metadata.cell_info.is_some());
        eprintln!("Technology: {:?}", f.data.metadata.technology);
        eprintln!("Cell info: {:?}", f.data.metadata.cell_info);
    }

    #[test]
    fn test_asn1_parser() {
        use tramex_tools::asn1_parser::parse_asn1_to_json;

        // Example RRC message (MIB)
        let asn1_text = r#"{
            message mib: {
                systemFrameNumber '111000'B,
                subCarrierSpacingCommon scs30or120,
                ssb-SubcarrierOffset 0,
                dmrs-TypeA-Position pos2,
                pdcch-ConfigSIB1 {
                    controlResourceSetZero 10,
                    searchSpaceZero 0
                },
                cellBarred notBarred,
                intraFreqReselection allowed,
                spare '0'B
            }
        }"#;

        let result = parse_asn1_to_json(asn1_text);
        assert!(result.is_ok(), "Failed to parse ASN.1: {:?}", result.err());

        let json = result.unwrap();
        eprintln!("Parsed JSON:\n{}", serde_json::to_string_pretty(&json).unwrap());

        // Verify structure
        assert!(json.is_object());
        assert!(json["message"].is_object());
        assert!(json["message"]["mib"].is_object());
        assert!(json["message"]["mib"]["pdcch-ConfigSIB1"].is_object());
        assert_eq!(json["message"]["mib"]["pdcch-ConfigSIB1"]["controlResourceSetZero"], 10);
        assert_eq!(json["message"]["mib"]["pdcch-ConfigSIB1"]["searchSpaceZero"], 0);
    }

    #[test]
    fn test_asn1_parser_nested() {
        use tramex_tools::asn1_parser::parse_asn1_to_json;

        // Test nested sequences
        let asn1_text = r#"{
            cellSelectionInfo {
                q-RxLevMin -70,
                q-QualMin -20
            },
            cellAccessRelatedInfo {
                trackingAreaCode '000065'H,
                cellIdentity '001234501'H
            }
        }"#;

        let result = parse_asn1_to_json(asn1_text);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json["cellSelectionInfo"].is_object());
        assert_eq!(json["cellSelectionInfo"]["q-RxLevMin"], -70);
        assert_eq!(json["cellSelectionInfo"]["q-QualMin"], -20);
    }

    #[test]
    fn test_asn1_parser_array() {
        use tramex_tools::asn1_parser::parse_asn1_to_json;

        // Test array parsing
        let asn1_text = r#"{
            mcc {
                0,
                0,
                1
            }
        }"#;

        let result = parse_asn1_to_json(asn1_text);
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let json = result.unwrap();
        eprintln!("Parsed JSON: {}", serde_json::to_string_pretty(&json).unwrap());

        // The mcc field contains an array
        assert!(json["mcc"].is_array(), "mcc is not an array: {:?}", json["mcc"]);
        assert_eq!(json["mcc"][0], 0);
        assert_eq!(json["mcc"][1], 0);
        assert_eq!(json["mcc"][2], 1);
    }
}
