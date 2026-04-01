/// Probe: verifies the fixture loads and checks which deployment elements were parsed.
/// Run with: cargo test -p cargo-arxml probe_ -- --nocapture
use autosar_data::{AutosarModel, AutosarVersion, ElementName};

/// Build a SOMEIP-SERVICE-INTERFACE-DEPLOYMENT element in memory and probe its
/// valid children by trying to create each candidate.
#[test]
fn probe_valid_children_of_someip_deployment() {
    let model = AutosarModel::new();
    model
        .create_file("probe.arxml", AutosarVersion::Autosar_00051)
        .expect("create file");

    let autosar = model.root_element();
    let ar_packages = autosar
        .create_sub_element(ElementName::ArPackages)
        .expect("ArPackages");
    let pkg = ar_packages
        .create_named_sub_element(ElementName::ArPackage, "deployment")
        .expect("ArPackage");
    let elements = pkg
        .create_sub_element(ElementName::Elements)
        .expect("Elements");
    let dep = elements
        .create_named_sub_element(ElementName::SomeipServiceInterfaceDeployment, "TestDep")
        .expect("SomeipServiceInterfaceDeployment");

    let candidates = [
        ElementName::ServiceInterfaceRef,
        ElementName::ServiceInterfaceId,
        ElementName::MethodDeployments,
        ElementName::EventDeployments,
        ElementName::EventGroups,
    ];

    for name in &candidates {
        match dep.create_sub_element(*name) {
            Ok(_) => println!("{name:?}: VALID child of SomeipServiceInterfaceDeployment"),
            Err(e) => println!("{name:?}: INVALID — {e}"),
        }
    }
}

/// Probe whether SOMEIP-METHOD-DEPLOYMENT is valid inside METHOD-DEPLOYMENTS.
#[test]
fn probe_method_deployment_inside_method_deployments() {
    let model = AutosarModel::new();
    model
        .create_file("probe2.arxml", AutosarVersion::Autosar_00051)
        .expect("create file");

    let autosar = model.root_element();
    let ar_packages = autosar
        .create_sub_element(ElementName::ArPackages)
        .expect("ArPackages");
    let pkg = ar_packages
        .create_named_sub_element(ElementName::ArPackage, "deployment")
        .expect("ArPackage");
    let elements = pkg
        .create_sub_element(ElementName::Elements)
        .expect("Elements");
    let dep = elements
        .create_named_sub_element(ElementName::SomeipServiceInterfaceDeployment, "TestDep")
        .expect("SomeipServiceInterfaceDeployment");

    match dep.create_sub_element(ElementName::MethodDeployments) {
        Ok(container) => {
            match container.create_named_sub_element(ElementName::SomeipMethodDeployment, "M1") {
                Ok(_) => println!("METHOD-DEPLOYMENTS > SOMEIP-METHOD-DEPLOYMENT: VALID"),
                Err(e) => println!("METHOD-DEPLOYMENTS > SOMEIP-METHOD-DEPLOYMENT: INVALID — {e}"),
            }
        }
        Err(e) => println!("METHOD-DEPLOYMENTS container: INVALID — {e}"),
    }
}

/// Load the actual battery_service fixture and print what deployment elements are found.
#[test]
fn probe_fixture_loads_deployment() {
    use cargo_arxml::parser::ArxmlParser;
    use std::path::Path;

    let parser = ArxmlParser::load(Path::new("tests/fixtures/battery_service.arxml"))
        .expect("fixture must load");
    let project = parser.extract_ir().expect("IR extraction must succeed");

    println!("Deployments found: {}", project.deployments.len());
    for dep in &project.deployments {
        println!(
            "  deployment: service_ref={} service_id={}",
            dep.service_interface_ref, dep.service_id
        );
        for md in &dep.method_deployments {
            println!("    method: {} id={}", md.short_name, md.method_id);
        }
        for ed in &dep.event_deployments {
            println!("    event: {} id={}", ed.short_name, ed.event_id);
        }
        for eg in &dep.event_groups {
            println!(
                "    event_group: {} id={}",
                eg.short_name, eg.event_group_id
            );
        }
    }

    println!("Services: {}", project.services.len());
    if let Some(svc) = project.services.first() {
        println!("  service_id after merge: {:?}", svc.service_id);
        for m in &svc.methods {
            println!("  method: {} id={:?}", m.name, m.method_id);
        }
        for e in &svc.events {
            println!(
                "  event: {} id={:?} group={:?}",
                e.name, e.event_id, e.event_group_id
            );
        }
    }
}
