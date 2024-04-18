// tests
#[cfg(test)]
mod tests {
    use tramex_tools::websocket::types::WebSocketLog;

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "message": "log_get",
            "message_id": 1,
            "logs": [
                {
                    "data": [
                        "cce_index=0/12 L=4 dci=1a",
                        "\tdistributed_vrb=0",
                        "\triv=0x4b",
                        "\tharq=0",
                        "\tmcs1=2",
                        "\tnew_data_indicator1=0",
                        "\trv_idx1=1",
                        "\ttpc_command=1"
                    ],
                    "src": "ENB",
                    "idx": 31909948,
                    "level": 4,
                    "timestamp": 1711704467494,
                    "layer": "PHY",
                    "dir": "DL",
                    "cell": 1,
                    "rnti": 65535,
                    "frame": 470,
                    "slot": 5,
                    "channel": "PDCCH"
                }
            ],
            "time": 164772.65,
            "utc": 1711704509.801
        }"#;
        let ws_log: Result<WebSocketLog, serde_json::Error> = serde_json::from_str(json);
        let oked = ws_log.is_ok();
        if let Err(ref e) = ws_log {
            println!("-------------------");
            println!("{:?}", e);
            println!("-------------------");
            assert!(oked);
        }
        if let Ok(ws_log) = ws_log {
            println!("{:?}", ws_log.logs[0]);
        }
    }

    use tramex_tools::functions::extract_hexe;
    #[test]
    fn test_extract_hexe() {
        let data = vec![
            "0000:  00 80 4c 61 bc 8c 8c c1  16 08 a8 02 40 04 08 01  ..La........@...",
            "0010:  73 39 4f 52 d5 42 48 00  18 01 2e 38 03 84 28 c5  s9OR.BH....8..(.",
            "0020:  b0 9d 4b 48",
        ];
        let hexe = extract_hexe(&data);
        println!("{:?}", hexe);
        let res: Vec<u8> = vec![
            0, 128, 76, 97, 188, 140, 140, 193, 22, 8, 168, 2, 64, 4, 8, 1, 115, 57, 79, 82, 213,
            66, 72, 0, 24, 1, 46, 56, 3, 132, 40, 197,
        ]
        .into();

        assert!(hexe == res)
    }
}
