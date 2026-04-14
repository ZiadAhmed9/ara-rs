//! Codegen integration tests for the diagnostics-service ARXML fixture.
//!
//! Exercises capabilities beyond battery-service: multiple methods, nested
//! structs, multiple input parameters, multiple events, and custom type
//! imports in generated proxy code.

use std::path::Path;

use cargo_arxml::codegen::CodeGenerator;
use cargo_arxml::parser::ArxmlParser;
use cargo_arxml::validator;

const FIXTURE_PATH: &str = "tests/fixtures/diagnostics_service.arxml";

#[test]
fn test_parse_diagnostics_fixture() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");

    assert_eq!(project.services.len(), 1);
    let svc = &project.services[0];
    assert_eq!(svc.short_name, "DiagnosticsService");
    assert_eq!(svc.methods.len(), 4, "expected 4 methods");
    assert_eq!(svc.events.len(), 2, "expected 2 events");

    // ReadDtc: 1 input, 1 output (struct type)
    assert_eq!(svc.methods[0].name, "ReadDtc");
    assert!(!svc.methods[0].fire_and_forget);
    assert_eq!(
        svc.methods[0].output_params[0].type_ref,
        "/types/DtcSnapshot"
    );

    // ClearDtc: 1 input, fire-and-forget
    assert_eq!(svc.methods[1].name, "ClearDtc");
    assert!(svc.methods[1].fire_and_forget);

    // ReadEcuIdentification: 0 inputs, 1 output (struct type)
    assert_eq!(svc.methods[2].name, "ReadEcuIdentification");
    assert!(svc.methods[2].input_params.is_empty());

    // ReadDataByIdentifier: 2 inputs, 1 output
    assert_eq!(svc.methods[3].name, "ReadDataByIdentifier");
    assert_eq!(svc.methods[3].input_params.len(), 2);

    // Data types: 3 structs
    assert_eq!(project.data_types.len(), 3);
    let type_names: Vec<&str> = project
        .data_types
        .iter()
        .map(|dt| dt.name.as_str())
        .collect();
    assert!(type_names.contains(&"DtcSnapshot"));
    assert!(type_names.contains(&"EcuInfo"));
    assert!(type_names.contains(&"DataRecord"));
}

#[test]
fn test_diagnostics_validates_cleanly() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let errors = validator::validate(&project);
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}

#[test]
fn test_diagnostics_deployment_ids() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");

    let svc = &project.services[0];
    assert_eq!(svc.service_id, Some(0x5000), "service ID from deployment");
    assert_eq!(svc.methods[0].method_id, Some(1), "ReadDtc method ID");
    assert_eq!(svc.methods[1].method_id, Some(2), "ClearDtc method ID");
    assert_eq!(
        svc.methods[2].method_id,
        Some(3),
        "ReadEcuIdentification method ID"
    );
    assert_eq!(
        svc.methods[3].method_id,
        Some(4),
        "ReadDataByIdentifier method ID"
    );

    assert_eq!(
        svc.events[0].event_id,
        Some(0x8001),
        "DtcStatusChanged event ID"
    );
    assert_eq!(
        svc.events[1].event_id,
        Some(0x8002),
        "SessionChanged event ID"
    );
}

#[test]
fn test_diagnostics_codegen_produces_all_files() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let (output, auto_assignments) = gen.generate().expect("code generation failed");

    assert!(
        auto_assignments.is_empty(),
        "all IDs should come from deployment"
    );

    assert!(output.contains_key("types.rs"));
    assert!(output.contains_key("traits.rs"));
    assert!(output.contains_key("proxy/diagnostics_service.rs"));
    assert!(output.contains_key("skeleton/diagnostics_service.rs"));
    assert!(output.contains_key("mod.rs"));
}

#[test]
fn test_diagnostics_generated_types_contain_structs() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let (output, _) = gen.generate().expect("code generation failed");

    let types = &output["types.rs"];
    assert!(
        types.contains("pub struct DtcSnapshot"),
        "missing DtcSnapshot"
    );
    assert!(types.contains("pub dtc_id: u32"), "missing dtc_id field");
    assert!(
        types.contains("pub status_byte: u8"),
        "missing status_byte field"
    );
    assert!(
        types.contains("pub timestamp: u64"),
        "missing timestamp field"
    );
    assert!(
        types.contains("pub occurrence_count: u16"),
        "missing occurrence_count"
    );

    assert!(types.contains("pub struct EcuInfo"), "missing EcuInfo");
    assert!(
        types.contains("pub serial_number: u32"),
        "missing serial_number"
    );
    assert!(
        types.contains("pub uptime_seconds: u64"),
        "missing uptime_seconds"
    );

    assert!(
        types.contains("pub struct DataRecord"),
        "missing DataRecord"
    );
}

#[test]
fn test_diagnostics_proxy_imports_custom_types() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let (output, _) = gen.generate().expect("code generation failed");

    let proxy = &output["proxy/diagnostics_service.rs"];
    // Must import types since proxy references DtcSnapshot, EcuInfo, DataRecord
    assert!(
        proxy.contains("use super::super::types::*"),
        "proxy must import custom types"
    );
    assert!(proxy.contains("pub struct DiagnosticsServiceProxy"));
    assert!(proxy.contains("async fn read_dtc"));
    assert!(proxy.contains("async fn clear_dtc"));
    assert!(proxy.contains("async fn read_ecu_identification"));
    assert!(proxy.contains("async fn read_data_by_identifier"));
    assert!(proxy.contains("subscribe_dtc_status_changed"));
    assert!(proxy.contains("subscribe_session_changed"));
}

#[test]
fn test_diagnostics_traits_import_custom_types() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let (output, _) = gen.generate().expect("code generation failed");

    let traits = &output["traits.rs"];
    assert!(
        traits.contains("use super::types::*"),
        "traits must import custom types"
    );
    assert!(traits.contains("pub trait DiagnosticsService"));
}
