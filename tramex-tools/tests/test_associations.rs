use std::path::Path;
use tramex_tools::data::Data;
use tramex_tools::interface::association::AssociationRules;
use tramex_tools::interface::interface_file::file_handler::File;
use tramex_tools::interface::interface_types::InterfaceTrait;
use tramex_tools::interface::layer::Layers;

fn get_path(p: &str) -> String {
    if std::env::current_dir().unwrap().ends_with("tramex-tools") {
        return Path::new("tests").join(p).to_string_lossy().to_string();
    }
    let filename = file!();
    Path::new(filename).parent().unwrap().join(p).to_string_lossy().to_string()
}

const NB_EVENTS: usize = 20;
#[test]
fn test_nas_rrc_associations() {
    // Read and parse the log file like other tests
    let filename = &get_path("gnb_associations.log");
    let content = std::fs::read_to_string(filename).unwrap();
    let mut file = File::new_file_content(filename.into(), content);
    let mut data: Data = Default::default();

    // Load all data from file, capturing any errors
    let mut parse_errors = Vec::new();
    while !file.full_read {
        if let Err(e) = file.get_more_data(Layers::all_debug(), &mut data) {
            parse_errors.extend(e);
        }
    }
    if !parse_errors.is_empty() {
        println!("Parse errors: {:?}", parse_errors.len());
        for e in &parse_errors {
            println!("  - {}", e.message);
        }
    }
    assert!(parse_errors.is_empty(), "Parse errors");

    // Print parsed traces
    println!("\n=== Parsed Traces ===");
    println!("Total traces: {}", data.events.len());
    assert!(
        data.events.len() == NB_EVENTS,
        "Total traces should be {}, found {}",
        NB_EVENTS,
        data.events.len()
    );

    // for (idx, trace) in data.events.iter().enumerate() {
    //     let msg_name = trace.additional_infos.get_message_name().unwrap_or_default();
    //     println!("Trace {}: {:?} - {}", idx, trace.layer, msg_name);
    // }

    // Compute ALL associations automatically
    let rules = AssociationRules::new();
    data.compute_all_associations(&rules);

    // Show relationship status for all traces
    println!("\n=== Relationship Status ===");
    for (idx, trace) in data.events.iter().enumerate() {
        let parent_status = &trace.relation.parent;
        let child_status = &trace.relation.child;
        println!(
            "Trace {} ({:?}): parent={:?}, child={:?}",
            idx, trace.layer, parent_status, child_status
        );
    }

    assert!(
        data.events[0].relation.get_child_indices() == Some(&vec![1]),
        "Child index of trace 0 should be 1"
    );
    assert!(
        data.events[1].relation.get_parent_indices() == Some(&vec![0]),
        "Parent index of trace 1 should be 0"
    );

    assert!(data.events[2].relation.has_child() == false, "Trace 2 should have no child");
    assert!(
        data.events[13].relation.has_parent() == false,
        "Trace 13 should have no parent"
    );

    assert!(
        data.events[14].relation.get_child_indices() == Some(&vec![15]),
        "Child index of trace 14 should be 15"
    );
    assert!(
        data.events[15].relation.get_parent_indices() == Some(&vec![14, 16]),
        "Parent index of trace 15 should be 14 and 16"
    );
    assert!(
        data.events[16].relation.get_child_indices() == Some(&vec![15]),
        "Child index of trace 16 should be 15"
    );

    assert!(
        data.events[17].relation.get_child_indices() == Some(&vec![18]),
        "Child index of trace 17 should be 18"
    );
    assert!(
        data.events[18].relation.get_parent_indices() == Some(&vec![17, 19]),
        "Parent index of trace 18 should be 17 and 19"
    );
    assert!(
        data.events[19].relation.get_child_indices() == Some(&vec![18]),
        "Child index of trace 19 should be 18"
    );
}
