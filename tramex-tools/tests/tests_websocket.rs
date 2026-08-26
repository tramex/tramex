//! Tests for WebSocket message parsing through the unified LayerParser pipeline.
//! Uses realistic API response fixtures containing PHY, MAC, PDCP, RRC, and NGAP logs.

#[cfg(test)]
mod tests {
    use tramex_tools::data::AdditionalInfos;
    use tramex_tools::interface::parse_config::{FileMetadata, Technology};
    use tramex_tools::interface::parser::parser_phy::{PHYChannelData, PHYChannelType};
    use tramex_tools::interface::types::{Direction, WebSocketLog};

    /// Load and deserialize the fixture file
    fn load_fixture() -> WebSocketLog {
        let json = include_str!("ws_response.json");
        serde_json::from_str(json).expect("Failed to deserialize ws_response.json")
    }

    #[test]
    fn test_deserialize_ws_response() {
        let ws_log = load_fixture();
        assert_eq!(ws_log.message, "log_get");
        assert_eq!(ws_log.message_id, Some(42));
        assert_eq!(ws_log.logs.len(), 5);
    }

    #[test]
    fn test_all_logs_parse_successfully() {
        let ws_log = load_fixture();
        for (i, one_log) in ws_log.logs.iter().enumerate() {
            let result = one_log.extract_data();
            assert!(
                result.is_ok(),
                "Log #{i} ({:?}) failed to parse: {:?}",
                one_log.layer,
                result.err()
            );
        }
    }

    // ── PHY PDCCH ──────────────────────────────────────────────────────

    #[test]
    fn test_phy_pdcch_additional_infos() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[0].extract_data().unwrap();

        assert_eq!(trace.timestamp, 1785487689385);
        assert_eq!(trace.layer, tramex_tools::interface::layer::Layer::PHY);

        match &trace.additional_infos {
            AdditionalInfos::PHYInfos(phy) => {
                assert_eq!(phy.direction, Direction::DL);
                assert_eq!(phy.channel_type, PHYChannelType::PDCCH);
                assert_eq!(phy.frame, 46);
                assert_eq!(phy.slot, 15);
                assert_eq!(phy.prb_start, 0); // PDCCH has no prb
                assert_eq!(phy.prb_length, 0);
            }
            other => panic!("Expected PHYInfos, got {:?}", other),
        }
    }

    #[test]
    fn test_phy_pdcch_channel_data() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[0].extract_data().unwrap();

        match &trace.additional_infos {
            AdditionalInfos::PHYInfos(phy) => match &phy.channel_data {
                PHYChannelData::Pdcch {
                    dci,
                    harq_process,
                    ndi,
                    rv_idx,
                    harq_feedback_timing,
                } => {
                    assert_eq!(dci, "0_1");
                    assert_eq!(*harq_process, Some(0));
                    assert_eq!(*ndi, Some(0));
                    assert_eq!(*rv_idx, Some(0));
                    assert_eq!(*harq_feedback_timing, None); // DCI 0_1 has no harq_feedback_timing
                }
                other => panic!("Expected Pdcch channel data, got {:?}", other),
            },
            other => panic!("Expected PHYInfos, got {:?}", other),
        }
    }

    #[test]
    fn test_phy_pdcch_text_preserved() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[0].extract_data().unwrap();

        let text = trace.text.as_ref().expect("text should be present");
        assert_eq!(text.len(), 13);
        assert!(text[0].contains("ss_id=2"));
        assert!(text[6].contains("harq_process=0"));
    }

    // ── MAC ────────────────────────────────────────────────────────────

    #[test]
    fn test_mac_basic_parsing() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[1].extract_data().unwrap();

        assert_eq!(trace.timestamp, 1785489496003);
        assert_eq!(trace.layer, tramex_tools::interface::layer::Layer::MAC);
        assert!(matches!(trace.additional_infos, AdditionalInfos::None));

        let text = trace.text.as_ref().expect("text should be present");
        assert_eq!(text.len(), 1);
        assert!(text[0].contains("TAG:0 ta=31"));
    }

    // ── PDCP ───────────────────────────────────────────────────────────

    #[test]
    fn test_pdcp_basic_parsing() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[2].extract_data().unwrap();

        assert_eq!(trace.timestamp, 1785489499513);
        assert_eq!(trace.layer, tramex_tools::interface::layer::Layer::PDCP);
        assert!(matches!(trace.additional_infos, AdditionalInfos::None));

        let text = trace.text.as_ref().expect("text should be present");
        assert_eq!(text.len(), 1);
        assert_eq!(text[0], "SRB1 SN=3");
    }

    // ── RRC ────────────────────────────────────────────────────────────

    #[test]
    fn test_rrc_additional_infos() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[3].extract_data().unwrap();

        assert_eq!(trace.timestamp, 1785489499513);
        assert_eq!(trace.layer, tramex_tools::interface::layer::Layer::RRC);

        match &trace.additional_infos {
            AdditionalInfos::RRCInfos(rrc) => {
                assert_eq!(rrc.direction, Direction::DL);
                assert_eq!(rrc.canal, "DCCH-NR");
                assert_eq!(rrc.canal_msg, "RRC release");
            }
            other => panic!("Expected RRCInfos, got {:?}", other),
        }
    }

    #[test]
    fn test_rrc_binary_extracted() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[3].extract_data().unwrap();

        let binary = trace.binary.as_ref().expect("binary should be extracted from hex dump");
        assert_eq!(binary, &[0x10, 0x00]);
    }

    // ── NGAP ───────────────────────────────────────────────────────────

    #[test]
    fn test_ngap_additional_infos() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[4].extract_data().unwrap();

        assert_eq!(trace.timestamp, 1785489499513);
        assert_eq!(trace.layer, tramex_tools::interface::layer::Layer::NGAP);

        match &trace.additional_infos {
            AdditionalInfos::NGAPInfos(ngap) => {
                assert_eq!(ngap.direction, Direction::TO);
                assert_eq!(ngap.connection_info, Some("127.0.1.100:38412".to_string()));
                assert_eq!(ngap.message_type, "UE context release request");
            }
            other => panic!("Expected NGAPInfos, got {:?}", other),
        }
    }

    #[test]
    fn test_ngap_binary_extracted() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[4].extract_data().unwrap();

        let binary = trace.binary.as_ref().expect("binary should be extracted from hex dump");
        // First bytes: 00 2a 40 1e ...
        assert!(binary.len() >= 4);
        assert_eq!(binary[0], 0x00);
        assert_eq!(binary[1], 0x2a);
        assert_eq!(binary[2], 0x40);
        assert_eq!(binary[3], 0x1e);
    }

    #[test]
    fn test_ngap_text_contains_asn1() {
        let ws_log = load_fixture();
        let trace = ws_log.logs[4].extract_data().unwrap();

        let text = trace.text.as_ref().expect("text should be present");
        assert!(text.iter().any(|line| line.contains("initiatingMessage")));
        assert!(text.iter().any(|line| line.contains("UEContextReleaseRequest")));
    }

    // ── Direction and message_name helpers ──────────────────────────────

    #[test]
    fn test_direction_from_all_layers() {
        let ws_log = load_fixture();
        let directions: Vec<Option<Direction>> = ws_log
            .logs
            .iter()
            .map(|log| log.extract_data().unwrap().additional_infos.get_direction())
            .collect();

        assert_eq!(directions[0], Some(Direction::DL)); // PHY
        assert_eq!(directions[1], None); // MAC (BasicParser, no direction)
        assert_eq!(directions[2], None); // PDCP (BasicParser, no direction)
        assert_eq!(directions[3], Some(Direction::DL)); // RRC
        assert_eq!(directions[4], Some(Direction::TO)); // NGAP
    }

    #[test]
    fn test_message_name_from_all_layers() {
        let ws_log = load_fixture();
        let names: Vec<Option<String>> = ws_log
            .logs
            .iter()
            .map(|log| log.extract_data().unwrap().additional_infos.get_message_name())
            .collect();

        assert_eq!(names[0], Some("PDCCH".to_string())); // PHY
        assert_eq!(names[1], None); // MAC
        assert_eq!(names[2], None); // PDCP
        assert_eq!(names[3], Some("RRC release".to_string())); // RRC
        assert_eq!(names[4], Some("UE context release request".to_string())); // NGAP
    }

    // ── Headers / FileMetadata ─────────────────────────────────────────

    #[test]
    fn test_headers_present() {
        let ws_log = load_fixture();

        assert!(ws_log.headers.is_some());
        let headers = ws_log.headers.as_ref().unwrap();
        assert_eq!(headers.len(), 4);
        assert!(headers[0].contains("lteenb version"));
        assert!(headers[1].contains("nr_arfcn=640000"));
    }

    #[test]
    fn test_parse_metadata_from_headers() {
        let ws_log = load_fixture();
        let headers = ws_log.headers.as_ref().unwrap();

        // Headers already come with "# " prefix from the API
        let metadata = FileMetadata::parse_from_lines(headers);

        assert_eq!(metadata.technology, Technology::NR);
        assert_eq!(metadata.pci, Some(1));
        assert_eq!(metadata.mode, Some("TDD".to_string()));
        assert_eq!(metadata.arfcn, Some(640000));
        assert_eq!(metadata.n_rb, Some(51));
        assert_eq!(metadata.io_mode, Some("SISO".to_string()));
        assert!(metadata.version.is_some());
        assert!(metadata.version.unwrap().contains("lteenb version"));
        assert_eq!(metadata.ssb_info.len(), 1);
        assert!(metadata.ssb_info[0].contains("ssb_nr_arfcn=640000"));
    }
}
