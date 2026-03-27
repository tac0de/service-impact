use crate::model::{
    AnalysisMode, ImpactReason, ImpactResult, ImpactSummary, ImpactedService, PlannedHook,
    Registry, ServiceManifest, VerificationPlan,
};
use crate::pathing::matches_paths;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ImpactEngine {
    pub(crate) services: BTreeMap<String, ServiceManifest>,
}

impl ImpactEngine {
    pub fn from_registry(registry: Registry) -> Result<Self> {
        let mut services = BTreeMap::new();
        for service in registry.services {
            if services
                .insert(service.service_id.clone(), service)
                .is_some()
            {
                return Err(anyhow!("duplicate service_id"));
            }
        }
        Ok(Self { services })
    }

    pub fn impacted_services(
        &self,
        service_id: &str,
        changed_paths: &[impl AsRef<str>],
    ) -> Result<ImpactResult> {
        self.impacted_services_with_mode(service_id, changed_paths, AnalysisMode::Conservative)
    }

    pub fn impacted_services_with_mode(
        &self,
        service_id: &str,
        changed_paths: &[impl AsRef<str>],
        mode: AnalysisMode,
    ) -> Result<ImpactResult> {
        let source = self
            .services
            .get(service_id)
            .ok_or_else(|| anyhow!("unknown service_id: {}", service_id))?;
        let changed_paths = changed_paths
            .iter()
            .map(|path| path.as_ref().to_string())
            .collect::<Vec<_>>();
        let active_provides = source
            .provides
            .iter()
            .filter(|provided| matches_paths(&provided.paths, &changed_paths))
            .cloned()
            .collect::<Vec<_>>();

        let mut impacted_services = Vec::new();
        for (other_id, other) in &self.services {
            if other_id == service_id {
                continue;
            }
            let mut reasons = Vec::new();
            if mode == AnalysisMode::Conservative
                && other.depends_on.iter().any(|item| item == service_id)
            {
                reasons.push(ImpactReason {
                    reason_type: "depends_on".to_string(),
                    kind: None,
                    name: None,
                    via: Some(service_id.to_string()),
                });
            }
            for consumed in &other.consumes {
                if let Some(target) = &consumed.service_id {
                    if target != service_id {
                        continue;
                    }
                }
                for provided in &active_provides {
                    if consumed.kind == provided.kind && consumed.name == provided.name {
                        reasons.push(ImpactReason {
                            reason_type: "consumes".to_string(),
                            kind: Some(consumed.kind.clone()),
                            name: Some(consumed.name.clone()),
                            via: Some(service_id.to_string()),
                        });
                    }
                }
            }
            if !reasons.is_empty() {
                impacted_services.push(ImpactedService {
                    service_id: other_id.clone(),
                    reasons,
                    verification_hooks: other.verification_hooks.clone(),
                });
            }
        }
        impacted_services.sort_by(|left, right| left.service_id.cmp(&right.service_id));
        let impacted_service_count = impacted_services.len();
        let hook_count = impacted_services
            .iter()
            .map(|service| service.verification_hooks.len())
            .sum::<usize>();
        Ok(ImpactResult {
            service_id: service_id.to_string(),
            changed_paths,
            active_provides,
            impacted_services,
            summary: ImpactSummary {
                mode,
                impacted_service_count,
                verification_hook_count: hook_count,
                summary: format!(
                    "Found {} impacted service(s) and {} verification hook(s) in {:?} mode",
                    impacted_service_count, hook_count, mode
                ),
            },
        })
    }

    pub fn verification_plan(
        &self,
        service_id: &str,
        changed_paths: &[impl AsRef<str>],
    ) -> Result<VerificationPlan> {
        self.verification_plan_with_mode(service_id, changed_paths, AnalysisMode::Conservative)
    }

    pub fn verification_plan_with_mode(
        &self,
        service_id: &str,
        changed_paths: &[impl AsRef<str>],
        mode: AnalysisMode,
    ) -> Result<VerificationPlan> {
        let impact = self.impacted_services_with_mode(service_id, changed_paths, mode)?;
        let hooks = impact
            .impacted_services
            .iter()
            .flat_map(|service| {
                service.verification_hooks.iter().map(|hook| PlannedHook {
                    service_id: service.service_id.clone(),
                    name: hook.name.clone(),
                    trigger: hook.trigger.clone(),
                    command: hook.command.clone(),
                })
            })
            .collect::<Vec<_>>();
        let hook_count = hooks.len();
        let impacted_service_count = impact.impacted_services.len();
        Ok(VerificationPlan {
            mode,
            source_service: impact.service_id,
            changed_paths: impact.changed_paths,
            directly_impacted_services: impact
                .impacted_services
                .iter()
                .map(|service| service.service_id.clone())
                .collect(),
            hooks,
            summary: format!(
                "Run {} hook(s) across {} impacted service(s)",
                hook_count, impacted_service_count
            ),
        })
    }
}
