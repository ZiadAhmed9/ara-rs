use cargo_arxml::parser::ir::*;
use cargo_arxml::validator::{self, ValidationError};

/// Helper: build a minimal valid service with explicit IDs.
fn valid_service() -> ServiceInterface {
    ServiceInterface {
        name: "TestService".into(),
        short_name: "TestService".into(),
        path: "/services/TestService".into(),
        service_id: Some(0x1000),
        major_version: 1,
        minor_version: 0,
        methods: vec![Method {
            name: "DoSomething".into(),
            method_id: Some(0x0001),
            input_params: vec![Parameter {
                name: "value".into(),
                type_ref: "/types/uint32".into(),
                direction: ParamDirection::In,
            }],
            output_params: vec![Parameter {
                name: "result".into(),
                type_ref: "/types/uint16".into(),
                direction: ParamDirection::Out,
            }],
            fire_and_forget: false,
            description: None,
        }],
        events: vec![],
        fields: vec![],
        description: None,
    }
}

fn empty_project() -> ArxmlProject {
    ArxmlProject {
        services: vec![],
        data_types: vec![],
        source_files: vec![],
        deployments: vec![],
    }
}

// -----------------------------------------------------------------------
// Positive: valid projects pass cleanly
// -----------------------------------------------------------------------

#[test]
fn test_valid_service_passes_all_checks() {
    let project = ArxmlProject {
        services: vec![valid_service()],
        ..empty_project()
    };
    let errors = validator::validate(&project);
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}

#[test]
fn test_valid_fixture_still_passes() {
    // Ensures the battery_service fixture continues to validate cleanly.
    let parser = cargo_arxml::parser::ArxmlParser::load(std::path::Path::new(
        "tests/fixtures/battery_service.arxml",
    ))
    .expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let errors = validator::validate(&project);
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}

// -----------------------------------------------------------------------
// MissingTypeRef: unresolved references are detected
// -----------------------------------------------------------------------

#[test]
fn test_missing_type_ref_in_method_param() {
    let mut svc = valid_service();
    svc.methods[0].input_params[0].type_ref = "/types/NonExistentType".into();

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    assert!(
        errors.iter().any(|e| matches!(e, ValidationError::MissingTypeRef { type_ref, .. } if type_ref == "/types/NonExistentType")),
        "should detect unresolved type ref, got: {:?}",
        errors
    );
}

#[test]
fn test_missing_type_ref_in_output_param() {
    let mut svc = valid_service();
    svc.methods[0].output_params[0].type_ref = "/types/GhostType".into();

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    assert!(
        errors.iter().any(|e| matches!(e, ValidationError::MissingTypeRef { type_ref, .. } if type_ref == "/types/GhostType")),
        "should detect unresolved output type ref, got: {:?}",
        errors
    );
}

#[test]
fn test_missing_type_ref_in_struct_field() {
    let project = ArxmlProject {
        services: vec![],
        data_types: vec![DataType {
            name: "MyStruct".into(),
            path: "/types/MyStruct".into(),
            kind: DataTypeKind::Structure {
                fields: vec![StructField {
                    name: "broken_field".into(),
                    type_ref: "/types/DoesNotExist".into(),
                }],
            },
            description: None,
        }],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    assert!(
        errors.iter().any(|e| matches!(e, ValidationError::MissingTypeRef { type_ref, .. } if type_ref == "/types/DoesNotExist")),
        "should detect unresolved struct field type ref, got: {:?}",
        errors
    );
}

#[test]
fn test_resolvable_primitive_type_ref_passes() {
    // Primitive paths like /types/uint32 should not trigger MissingTypeRef.
    let svc = valid_service(); // uses /types/uint32 and /types/uint16

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    let type_ref_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, ValidationError::MissingTypeRef { .. }))
        .collect();
    assert!(
        type_ref_errors.is_empty(),
        "primitive types should not trigger MissingTypeRef: {:?}",
        type_ref_errors
    );
}

#[test]
fn test_known_data_type_path_resolves() {
    // A struct field referencing another known DataType should pass.
    let project = ArxmlProject {
        services: vec![],
        data_types: vec![
            DataType {
                name: "Inner".into(),
                path: "/types/Inner".into(),
                kind: DataTypeKind::Primitive(PrimitiveType::U32),
                description: None,
            },
            DataType {
                name: "Outer".into(),
                path: "/types/Outer".into(),
                kind: DataTypeKind::Structure {
                    fields: vec![StructField {
                        name: "inner".into(),
                        type_ref: "/types/Inner".into(),
                    }],
                },
                description: None,
            },
        ],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    let type_ref_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, ValidationError::MissingTypeRef { .. }))
        .collect();
    assert!(
        type_ref_errors.is_empty(),
        "known type path should resolve: {:?}",
        type_ref_errors
    );
}

// -----------------------------------------------------------------------
// InvalidMethodId: reserved ranges and duplicates
// -----------------------------------------------------------------------

#[test]
fn test_method_id_zero_is_invalid() {
    let mut svc = valid_service();
    svc.methods[0].method_id = Some(0x0000);

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidMethodId { id, .. } if *id == 0x0000)),
        "method ID 0x0000 should be rejected, got: {:?}",
        errors
    );
}

#[test]
fn test_method_id_ffff_is_invalid() {
    let mut svc = valid_service();
    svc.methods[0].method_id = Some(0xFFFF);

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidMethodId { id, .. } if *id == 0xFFFF)),
        "method ID 0xFFFF should be rejected, got: {:?}",
        errors
    );
}

#[test]
fn test_method_id_in_event_range_is_invalid() {
    let mut svc = valid_service();
    svc.methods[0].method_id = Some(0x8001); // event range

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidMethodId { id, .. } if *id == 0x8001)),
        "method ID in event range should be rejected, got: {:?}",
        errors
    );
}

#[test]
fn test_valid_method_id_boundary() {
    let mut svc = valid_service();
    svc.methods[0].method_id = Some(0x7FFF); // max valid method ID

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    let method_id_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, ValidationError::InvalidMethodId { .. }))
        .collect();
    assert!(
        method_id_errors.is_empty(),
        "method ID 0x7FFF should be valid, got: {:?}",
        method_id_errors
    );
}

#[test]
fn test_duplicate_method_ids_within_service() {
    let mut svc = valid_service();
    svc.methods.push(Method {
        name: "AnotherMethod".into(),
        method_id: Some(0x0001), // same as DoSomething
        input_params: vec![],
        output_params: vec![],
        fire_and_forget: true,
        description: None,
    });

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateMethodId { id, .. } if *id == 0x0001)),
        "duplicate method IDs should be detected, got: {:?}",
        errors
    );
}

// -----------------------------------------------------------------------
// Auto-ID assignment: warnings are surfaced
// -----------------------------------------------------------------------

#[test]
fn test_auto_id_assignment_returns_warnings() {
    use cargo_arxml::codegen::assign_default_ids;

    let mut project = ArxmlProject {
        services: vec![ServiceInterface {
            name: "NoIds".into(),
            short_name: "NoIds".into(),
            path: "/services/NoIds".into(),
            service_id: None,
            major_version: 1,
            minor_version: 0,
            methods: vec![Method {
                name: "Foo".into(),
                method_id: None,
                input_params: vec![],
                output_params: vec![],
                fire_and_forget: true,
                description: None,
            }],
            events: vec![Event {
                name: "Bar".into(),
                event_id: None,
                event_group_id: None,
                data_type_ref: None,
                description: None,
            }],
            fields: vec![],
            description: None,
        }],
        ..empty_project()
    };

    let assignments = assign_default_ids(&mut project);

    // Should have auto-assigned: service_id, method_id, event_id, event_group_id
    assert_eq!(
        assignments.len(),
        4,
        "expected 4 auto-assignments, got: {:?}",
        assignments
    );

    // IDs should now be set
    assert!(project.services[0].service_id.is_some());
    assert!(project.services[0].methods[0].method_id.is_some());
    assert!(project.services[0].events[0].event_id.is_some());
    assert!(project.services[0].events[0].event_group_id.is_some());
}

#[test]
fn test_explicit_ids_produce_no_warnings() {
    use cargo_arxml::codegen::assign_default_ids;

    let mut project = ArxmlProject {
        services: vec![valid_service()],
        ..empty_project()
    };

    let assignments = assign_default_ids(&mut project);
    assert!(
        assignments.is_empty(),
        "explicit IDs should produce no auto-assignments: {:?}",
        assignments
    );
}

#[test]
fn test_codegen_with_explicit_ids_returns_no_warnings() {
    let parser = cargo_arxml::parser::ArxmlParser::load(std::path::Path::new(
        "tests/fixtures/battery_service.arxml",
    ))
    .expect("failed to load fixture");
    let project = parser.extract_ir().expect("failed to extract IR");
    let gen = cargo_arxml::codegen::CodeGenerator::new(&project);
    let (_files, auto_assignments) = gen.generate().expect("code generation failed");

    assert!(
        auto_assignments.is_empty(),
        "battery_service fixture has full deployment — no auto-assignments expected: {:?}",
        auto_assignments
    );
}

// -----------------------------------------------------------------------
// Regression: auto-ID must not collide with existing explicit IDs
// -----------------------------------------------------------------------

#[test]
fn test_auto_id_skips_existing_explicit_method_id() {
    use cargo_arxml::codegen::assign_default_ids;

    // Method 0 has explicit ID 0x0002, method 1 is missing an ID.
    // Auto-assignment must NOT assign 0x0002 to method 1.
    let mut project = ArxmlProject {
        services: vec![ServiceInterface {
            name: "MixedIds".into(),
            short_name: "MixedIds".into(),
            path: "/services/MixedIds".into(),
            service_id: Some(0x2000),
            major_version: 1,
            minor_version: 0,
            methods: vec![
                Method {
                    name: "ExplicitMethod".into(),
                    method_id: Some(0x0002),
                    input_params: vec![],
                    output_params: vec![],
                    fire_and_forget: true,
                    description: None,
                },
                Method {
                    name: "MissingMethod".into(),
                    method_id: None,
                    input_params: vec![],
                    output_params: vec![],
                    fire_and_forget: true,
                    description: None,
                },
            ],
            events: vec![],
            fields: vec![],
            description: None,
        }],
        ..empty_project()
    };

    let _assignments = assign_default_ids(&mut project);
    let svc = &project.services[0];

    // Both methods must have IDs, and they must differ.
    let id0 = svc.methods[0].method_id.unwrap();
    let id1 = svc.methods[1].method_id.unwrap();
    assert_eq!(id0, 0x0002, "explicit ID should be preserved");
    assert_ne!(
        id0, id1,
        "auto-assigned ID must not collide with explicit ID"
    );
}

#[test]
fn test_auto_id_skips_existing_explicit_event_id() {
    use cargo_arxml::codegen::assign_default_ids;

    let mut project = ArxmlProject {
        services: vec![ServiceInterface {
            name: "MixedEvents".into(),
            short_name: "MixedEvents".into(),
            path: "/services/MixedEvents".into(),
            service_id: Some(0x3000),
            major_version: 1,
            minor_version: 0,
            methods: vec![],
            events: vec![
                Event {
                    name: "ExplicitEvent".into(),
                    event_id: Some(0x8001), // takes the first auto-assign slot
                    event_group_id: Some(1),
                    data_type_ref: None,
                    description: None,
                },
                Event {
                    name: "MissingEvent".into(),
                    event_id: None,
                    event_group_id: None,
                    data_type_ref: None,
                    description: None,
                },
            ],
            fields: vec![],
            description: None,
        }],
        ..empty_project()
    };

    let _assignments = assign_default_ids(&mut project);
    let svc = &project.services[0];

    let eid0 = svc.events[0].event_id.unwrap();
    let eid1 = svc.events[1].event_id.unwrap();
    assert_eq!(eid0, 0x8001);
    assert_ne!(eid0, eid1, "auto-assigned event ID must not collide");

    let gid0 = svc.events[0].event_group_id.unwrap();
    let gid1 = svc.events[1].event_group_id.unwrap();
    assert_eq!(gid0, 1);
    assert_ne!(gid0, gid1, "auto-assigned event group ID must not collide");
}

#[test]
fn test_auto_id_skips_existing_explicit_service_id() {
    use cargo_arxml::codegen::assign_default_ids;

    let mut project = ArxmlProject {
        services: vec![
            ServiceInterface {
                name: "Explicit".into(),
                short_name: "Explicit".into(),
                path: "/services/Explicit".into(),
                service_id: Some(0x1000), // takes the first auto-assign slot
                major_version: 1,
                minor_version: 0,
                methods: vec![],
                events: vec![],
                fields: vec![Field {
                    name: "dummy".into(),
                    data_type_ref: "/types/uint8".into(),
                    has_getter: true,
                    has_setter: false,
                    has_notifier: false,
                    getter_method_id: None,
                    setter_method_id: None,
                    notifier_event_id: None,
                    description: None,
                }],
                description: None,
            },
            ServiceInterface {
                name: "Missing".into(),
                short_name: "Missing".into(),
                path: "/services/Missing".into(),
                service_id: None,
                major_version: 1,
                minor_version: 0,
                methods: vec![],
                events: vec![],
                fields: vec![Field {
                    name: "dummy".into(),
                    data_type_ref: "/types/uint8".into(),
                    has_getter: true,
                    has_setter: false,
                    has_notifier: false,
                    getter_method_id: None,
                    setter_method_id: None,
                    notifier_event_id: None,
                    description: None,
                }],
                description: None,
            },
        ],
        ..empty_project()
    };

    let _assignments = assign_default_ids(&mut project);
    let sid0 = project.services[0].service_id.unwrap();
    let sid1 = project.services[1].service_id.unwrap();
    assert_eq!(sid0, 0x1000);
    assert_ne!(sid0, sid1, "auto-assigned service ID must not collide");
}

// -----------------------------------------------------------------------
// Regression: case-insensitive primitive refs and int* aliases
// -----------------------------------------------------------------------

#[test]
fn test_uppercase_primitive_ref_passes_validation() {
    let svc = ServiceInterface {
        name: "UpperCaseSvc".into(),
        short_name: "UpperCaseSvc".into(),
        path: "/services/UpperCaseSvc".into(),
        service_id: Some(0x5000),
        major_version: 1,
        minor_version: 0,
        methods: vec![Method {
            name: "Foo".into(),
            method_id: Some(1),
            input_params: vec![Parameter {
                name: "val".into(),
                type_ref: "/AUTOSAR/UINT32".into(), // uppercase
                direction: ParamDirection::In,
            }],
            output_params: vec![],
            fire_and_forget: true,
            description: None,
        }],
        events: vec![],
        fields: vec![],
        description: None,
    };

    let project = ArxmlProject {
        services: vec![svc],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    let type_ref_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, ValidationError::MissingTypeRef { .. }))
        .collect();
    assert!(
        type_ref_errors.is_empty(),
        "uppercase UINT32 should be accepted: {:?}",
        type_ref_errors
    );
}

#[test]
fn test_int_aliases_pass_validation() {
    // The parser accepts int8/int16/int32/int64 — the validator must too.
    let methods: Vec<Method> = ["int8", "int16", "int32", "int64"]
        .iter()
        .enumerate()
        .map(|(i, alias)| Method {
            name: format!("Method{}", i),
            method_id: Some(1 + i as u16),
            input_params: vec![Parameter {
                name: "val".into(),
                type_ref: format!("/types/{}", alias),
                direction: ParamDirection::In,
            }],
            output_params: vec![],
            fire_and_forget: true,
            description: None,
        })
        .collect();

    let project = ArxmlProject {
        services: vec![ServiceInterface {
            name: "IntAlias".into(),
            short_name: "IntAlias".into(),
            path: "/services/IntAlias".into(),
            service_id: Some(0x6000),
            major_version: 1,
            minor_version: 0,
            methods,
            events: vec![],
            fields: vec![],
            description: None,
        }],
        ..empty_project()
    };

    let errors = validator::validate(&project);
    let type_ref_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, ValidationError::MissingTypeRef { .. }))
        .collect();
    assert!(
        type_ref_errors.is_empty(),
        "int8/int16/int32/int64 aliases should be accepted: {:?}",
        type_ref_errors
    );
}
