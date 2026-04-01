use std::path::Path;

use cargo_arxml::codegen::CodeGenerator;
use cargo_arxml::parser::ArxmlParser;
use cargo_arxml::validator;

const FIXTURE_PATH: &str = "tests/fixtures/battery_service.arxml";

#[test]
fn test_parse_battery_service_fixture() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");

    assert_eq!(project.services.len(), 1);
    let svc = &project.services[0];
    assert_eq!(svc.short_name, "BatteryService");
    assert_eq!(svc.methods.len(), 2);
    assert_eq!(svc.events.len(), 1);

    // GetVoltage: 1 input (battery_id), 1 output (voltage)
    let get_voltage = &svc.methods[0];
    assert_eq!(get_voltage.name, "GetVoltage");
    assert!(!get_voltage.fire_and_forget);
    assert_eq!(get_voltage.input_params.len(), 1);
    assert_eq!(get_voltage.output_params.len(), 1);

    // SetChargeLimit: 1 input, fire-and-forget
    let set_limit = &svc.methods[1];
    assert_eq!(set_limit.name, "SetChargeLimit");
    assert!(set_limit.fire_and_forget);

    // VoltageChanged event
    assert_eq!(svc.events[0].name, "VoltageChanged");

    // Data types
    assert_eq!(project.data_types.len(), 1);
    let dt = &project.data_types[0];
    assert_eq!(dt.name, "BatteryStatus");
}

#[test]
fn test_validator_passes_for_fixture() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let errors = validator::validate(&project);
    assert!(
        errors.is_empty(),
        "unexpected validation errors: {:?}",
        errors
    );
}

#[test]
fn test_codegen_produces_all_expected_files() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let output = gen.generate().expect("code generation failed");

    // Expected files
    assert!(output.contains_key("types.rs"), "missing types.rs");
    assert!(output.contains_key("traits.rs"), "missing traits.rs");
    assert!(
        output.contains_key("proxy/battery_service.rs"),
        "missing proxy/battery_service.rs"
    );
    assert!(
        output.contains_key("skeleton/battery_service.rs"),
        "missing skeleton/battery_service.rs"
    );
    assert!(output.contains_key("tests.rs"), "missing tests.rs");
    assert!(output.contains_key("mod.rs"), "missing mod.rs");
}

#[test]
fn test_generated_types_contain_expected_identifiers() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let output = gen.generate().expect("code generation failed");

    let types = &output["types.rs"];
    assert!(
        types.contains("pub struct BatteryStatus"),
        "missing BatteryStatus struct"
    );
    assert!(types.contains("pub voltage: f64"), "missing voltage field");
    assert!(types.contains("pub current: f64"), "missing current field");
    assert!(
        types.contains("pub charging: bool"),
        "missing charging field"
    );
    assert!(
        types.contains("impl AraSerialize for BatteryStatus"),
        "missing serialize impl"
    );
    assert!(
        types.contains("impl AraDeserialize for BatteryStatus"),
        "missing deserialize impl"
    );
}

#[test]
fn test_generated_traits_contain_expected_methods() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let output = gen.generate().expect("code generation failed");

    let traits = &output["traits.rs"];
    assert!(traits.contains("pub trait BatteryService"), "missing trait");
    assert!(
        traits.contains("async fn get_voltage"),
        "missing get_voltage method"
    );
    assert!(
        traits.contains("async fn set_charge_limit"),
        "missing set_charge_limit method"
    );
}

#[test]
fn test_generated_proxy_contains_expected_structure() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let output = gen.generate().expect("code generation failed");

    let proxy = &output["proxy/battery_service.rs"];
    assert!(
        proxy.contains("pub struct BatteryServiceProxy"),
        "missing proxy struct"
    );
    assert!(
        proxy.contains("pub base: ProxyBase<T>"),
        "missing ProxyBase field"
    );
    assert!(
        proxy.contains("async fn get_voltage"),
        "missing get_voltage method"
    );
    assert!(
        proxy.contains("async fn set_charge_limit"),
        "missing set_charge_limit method"
    );
    assert!(
        proxy.contains("async fn subscribe_voltage_changed"),
        "missing event subscribe"
    );
    assert!(
        proxy.contains("call_fire_and_forget"),
        "set_charge_limit should use fire_and_forget"
    );
}

#[test]
fn test_generated_skeleton_contains_expected_structure() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let output = gen.generate().expect("code generation failed");

    let skeleton = &output["skeleton/battery_service.rs"];
    assert!(
        skeleton.contains("pub struct BatteryServiceSkeleton"),
        "missing skeleton struct"
    );
    assert!(
        skeleton.contains("pub base: SkeletonBase<T>"),
        "missing SkeletonBase field"
    );
    assert!(skeleton.contains("async fn offer"), "missing offer method");
    assert!(
        skeleton.contains("async fn stop_offer"),
        "missing stop_offer method"
    );
}

#[test]
fn test_parse_someip_deployment_ids() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");

    assert!(!project.services.is_empty(), "no services parsed");
    let svc = &project.services[0];

    // Service ID from deployment (0x4010 = 16400)
    assert_eq!(
        svc.service_id,
        Some(0x4010),
        "service_id should come from deployment"
    );

    // Method IDs from deployment
    assert_eq!(svc.methods[0].method_id, Some(1), "GetVoltage method_id");
    assert_eq!(
        svc.methods[1].method_id,
        Some(2),
        "SetChargeLimit method_id"
    );

    // Event ID from deployment (0x8001 = 32769)
    assert_eq!(
        svc.events[0].event_id,
        Some(0x8001),
        "VoltageChanged event_id"
    );

    // Event group ID from deployment
    assert_eq!(
        svc.events[0].event_group_id,
        Some(1),
        "VoltageChanged event_group_id"
    );
}

#[test]
fn test_deployment_ids_survive_assign_default_ids() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");

    // IDs should already be Some(...) from deployment so assign_default_ids won't overwrite them
    assert!(
        project.services[0].service_id.is_some(),
        "service_id should be set from deployment"
    );
    assert!(
        project.services[0].methods[0].method_id.is_some(),
        "method_id should be set from deployment"
    );
    assert!(
        project.services[0].events[0].event_id.is_some(),
        "event_id should be set from deployment"
    );
}

#[test]
fn test_generated_proxy_uses_deployment_ids() {
    let parser = ArxmlParser::load(Path::new(FIXTURE_PATH)).expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = CodeGenerator::new(&project);
    let output = gen.generate().expect("code generation failed");

    let proxy = &output["proxy/battery_service.rs"];
    // Should use ServiceId(16400) = 0x4010, not ServiceId(4096) = 0x1000 (the auto-assigned default)
    assert!(
        proxy.contains("ServiceId(16400)"),
        "proxy should use deployment service_id 0x4010 (16400), got snippet: {}",
        proxy.chars().take(400).collect::<String>()
    );
}
