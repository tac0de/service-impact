use crate::model::{Registry, ValidationIssue, ValidationReport};
use std::collections::BTreeSet;

pub fn validate_registry(registry: &Registry) -> ValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = BTreeSet::new();
    let service_ids = registry
        .services
        .iter()
        .map(|service| service.service_id.clone())
        .collect::<BTreeSet<_>>();
    let provides = registry
        .services
        .iter()
        .flat_map(|service| {
            service.provides.iter().map(move |provide| {
                (
                    service.service_id.clone(),
                    provide.kind.clone(),
                    provide.name.clone(),
                )
            })
        })
        .collect::<BTreeSet<_>>();

    for service in &registry.services {
        if !seen.insert(service.service_id.clone()) {
            errors.push(ValidationIssue {
                service_id: service.service_id.clone(),
                code: "duplicate_service_id".to_string(),
                message: format!("service_id {} is duplicated", service.service_id),
            });
        }
        for dependency in &service.depends_on {
            if !service_ids.contains(dependency) {
                errors.push(ValidationIssue {
                    service_id: service.service_id.clone(),
                    code: "unknown_dependency".to_string(),
                    message: format!("depends_on references unknown service {}", dependency),
                });
            }
        }
        for consume in &service.consumes {
            if let Some(target) = &consume.service_id {
                if !service_ids.contains(target) {
                    errors.push(ValidationIssue {
                        service_id: service.service_id.clone(),
                        code: "unknown_consume_target".to_string(),
                        message: format!("consume target {} does not exist", target),
                    });
                } else if !provides.contains(&(
                    target.clone(),
                    consume.kind.clone(),
                    consume.name.clone(),
                )) {
                    warnings.push(ValidationIssue {
                        service_id: service.service_id.clone(),
                        code: "unmatched_consume".to_string(),
                        message: format!(
                            "consume {}:{} does not match a provide on {}",
                            consume.kind, consume.name, target
                        ),
                    });
                }
            }
        }
        for hook in &service.verification_hooks {
            if hook.name.trim().is_empty() {
                errors.push(ValidationIssue {
                    service_id: service.service_id.clone(),
                    code: "empty_hook_name".to_string(),
                    message: "verification hook name must not be empty".to_string(),
                });
            }
            if hook.command.trim().is_empty() {
                errors.push(ValidationIssue {
                    service_id: service.service_id.clone(),
                    code: "empty_hook_command".to_string(),
                    message: format!("verification hook {} has an empty command", hook.name),
                });
            }
        }
        if service.provides.is_empty()
            && service.consumes.is_empty()
            && service.depends_on.is_empty()
        {
            warnings.push(ValidationIssue {
                service_id: service.service_id.clone(),
                code: "isolated_service".to_string(),
                message: "service has no provides, consumes, or depends_on edges".to_string(),
            });
        }
    }

    ValidationReport {
        valid: errors.is_empty(),
        summary: format!(
            "Validation completed with {} error(s) and {} warning(s)",
            errors.len(),
            warnings.len()
        ),
        errors,
        warnings,
    }
}
