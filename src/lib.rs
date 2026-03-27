mod engine;
mod model;
mod pathing;
mod replay;
mod validate;

pub use engine::ImpactEngine;
pub use model::{
    AnalysisMode, Consume, ImpactReason, ImpactResult, ImpactSummary, ImpactedService, PlannedHook,
    Provide, Registry, ReplayCase, ReplayCaseResult, ReplaySummary, ServiceManifest,
    ValidationIssue, ValidationReport, VerificationHook, VerificationPlan,
};
pub use replay::{load_replay_cases, run_replay};
pub use validate::validate_registry;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pathing::{matches_paths, normalize_path};

    fn engine() -> ImpactEngine {
        let registry = Registry {
            services: vec![
                ServiceManifest {
                    service_id: "api".to_string(),
                    provides: vec![Provide {
                        kind: "http".to_string(),
                        name: "billing".to_string(),
                        paths: vec!["src/billing".to_string()],
                    }],
                    consumes: vec![],
                    depends_on: vec![],
                    verification_hooks: vec![],
                },
                ServiceManifest {
                    service_id: "worker".to_string(),
                    provides: vec![],
                    consumes: vec![Consume {
                        service_id: Some("api".to_string()),
                        kind: "http".to_string(),
                        name: "billing".to_string(),
                    }],
                    depends_on: vec!["api".to_string()],
                    verification_hooks: vec![VerificationHook {
                        name: "worker-smoke".to_string(),
                        trigger: "impact".to_string(),
                        command: "cargo test -p worker smoke".to_string(),
                    }],
                },
            ],
        };
        ImpactEngine::from_registry(registry).unwrap()
    }

    #[test]
    fn computes_impacted_services() {
        let result = engine()
            .impacted_services("api", &["src/billing/mod.rs"])
            .unwrap();
        assert_eq!(result.impacted_services.len(), 1);
        assert_eq!(result.impacted_services[0].service_id, "worker");
        assert_eq!(result.summary.impacted_service_count, 1);
    }

    #[test]
    fn builds_verification_plan() {
        let plan = engine()
            .verification_plan("api", &["src/billing/mod.rs"])
            .unwrap();
        assert_eq!(plan.directly_impacted_services, vec!["worker"]);
        assert_eq!(plan.hooks.len(), 1);
    }

    #[test]
    fn strict_mode_drops_plain_dependency_only_matches() {
        let registry = Registry {
            services: vec![
                ServiceManifest {
                    service_id: "api".to_string(),
                    provides: vec![Provide {
                        kind: "http".to_string(),
                        name: "billing".to_string(),
                        paths: vec!["src/http".to_string()],
                    }],
                    consumes: vec![],
                    depends_on: vec![],
                    verification_hooks: vec![],
                },
                ServiceManifest {
                    service_id: "consumer".to_string(),
                    provides: vec![],
                    consumes: vec![],
                    depends_on: vec!["api".to_string()],
                    verification_hooks: vec![],
                },
            ],
        };
        let engine = ImpactEngine::from_registry(registry).unwrap();
        let conservative = engine
            .impacted_services_with_mode("api", &["src/http/router.rs"], AnalysisMode::Conservative)
            .unwrap();
        let strict = engine
            .impacted_services_with_mode("api", &["src/http/router.rs"], AnalysisMode::Strict)
            .unwrap();
        assert_eq!(conservative.impacted_services.len(), 1);
        assert!(strict.impacted_services.is_empty());
    }

    #[test]
    fn validates_registry() {
        let registry = Registry {
            services: vec![ServiceManifest {
                service_id: "api".to_string(),
                provides: vec![],
                consumes: vec![Consume {
                    service_id: Some("missing".to_string()),
                    kind: "event".to_string(),
                    name: "created".to_string(),
                }],
                depends_on: vec!["missing".to_string()],
                verification_hooks: vec![VerificationHook {
                    name: "".to_string(),
                    trigger: "impact".to_string(),
                    command: "".to_string(),
                }],
            }],
        };
        let report = validate_registry(&registry);
        assert!(!report.valid);
        assert!(report.errors.len() >= 3);
    }

    #[test]
    fn path_matching_normalizes_separators_and_dot_prefix() {
        let prefixes = vec!["./services/api/src/events/".to_string()];
        let changed = vec!["services\\api\\src\\events\\publisher.rs".to_string()];
        assert!(matches_paths(&prefixes, &changed));
        assert_eq!(normalize_path("./services/api/src/"), "services/api/src");
    }

    #[test]
    fn empty_changed_paths_do_not_activate_scoped_provides() {
        let prefixes = vec!["src/api".to_string()];
        let changed = Vec::<String>::new();
        assert!(!matches_paths(&prefixes, &changed));
    }

    #[test]
    fn empty_provide_paths_match_as_unscoped() {
        let prefixes = Vec::<String>::new();
        let changed = vec!["src/anywhere.rs".to_string()];
        assert!(matches_paths(&prefixes, &changed));
    }

    #[test]
    fn unknown_service_id_returns_error() {
        let error = engine().impacted_services("missing", &["src/billing/mod.rs"]);
        assert!(error.is_err());
    }
}
