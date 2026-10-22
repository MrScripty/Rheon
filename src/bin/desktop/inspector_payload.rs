//! Enumerated retained numerical payload; no decoding or numerical construction.
//! UI text/selection/export controls, SQLite and allocator internals are excluded.
use super::*;
use rheon::recording::provider::{RecordedProviderRejection, runtime::RuntimeOutcome};
use rheon::solver_provider::configured::ContextProviderIdentity;
use std::mem::{size_of, size_of_val};

#[derive(Default)]
struct Capacity(usize);
impl Capacity {
    fn add(&mut self, n: usize) -> Result<(), String> {
        self.0 = self
            .0
            .checked_add(n)
            .ok_or("Retained Inspector capacity overflow")?;
        Ok(())
    }
    fn vector<T>(&mut self, values: &Vec<T>) -> Result<(), String> {
        self.add(
            values
                .capacity()
                .checked_mul(size_of::<T>())
                .ok_or("Retained Inspector capacity overflow")?,
        )
    }
    fn strings<const N: usize>(&mut self, values: [&String; N]) -> Result<(), String> {
        for text in values {
            self.add(text.capacity())?;
        }
        Ok(())
    }
    fn arc(&mut self) -> Result<(), String> {
        self.add(2 * size_of::<usize>())
    }
    fn binding(&mut self, binding: &ContextProviderIdentity) -> Result<(), String> {
        self.strings([
            &binding.id,
            &binding.version,
            &binding.runtime,
            &binding.source_revision,
        ])?;
        if let Some(text) = &binding.training_run_id {
            self.add(text.capacity())?;
        }
        Ok(())
    }
    fn force(&mut self, fields: &ForceSnapshot) -> Result<(), String> {
        self.arc()?;
        let scene = fields.scene();
        self.add(size_of_val(scene))?;
        self.vector(&scene.objects)?;
        for object in &scene.objects {
            self.add(object.name.capacity())?;
        }
        Ok(())
    }
}

impl Inspector {
    /// Charges shared selections conservatively; aliases may be counted twice.
    pub fn retained_payload_bytes(&self) -> Result<usize, String> {
        let s = self
            .0
            .lock()
            .map_err(|_| "Retained Inspector lock unavailable")?;
        let mut bytes = Capacity(size_of::<Self>());
        if let Some(path) = &s.path {
            bytes.add(path.capacity())?;
        }
        if let Some(reader) = &s.reader {
            bytes.add(size_of_val(reader))?;
            let run = reader.run();
            bytes.strings([
                &run.identity.source_revision,
                &run.solver,
                &run.configuration,
                &run.provenance,
            ])?;
        }
        if let Some(raw) = &s.record {
            bytes.arc()?;
            bytes.add(raw.allocated_bytes().map_err(|e| e.to_string())?)?;
        }
        if let Some(voxel) = &s.voxel {
            bytes.arc()?;
            bytes.add(voxel.allocated_bytes().map_err(|e| e.to_string())?)?;
        }
        if let Some(carrier) = &s.carrier {
            bytes.arc()?;
            bytes.add(size_of_val(carrier.as_ref()))?;
            for field in [&carrier.x, &carrier.y, &carrier.z, &carrier.tracer] {
                bytes.vector(field)?;
            }
            if let Some((_, pressure)) = &carrier.interval_pressure {
                bytes.vector(pressure)?;
            }
            if let Some(fields) = &carrier.force_fields {
                bytes.force(fields)?;
            }
        }
        if let Some(material) = &s.material {
            bytes.arc()?;
            bytes.add(size_of_val(material.as_ref()))?;
            bytes.vector(&material.particles)?;
            if let Some((_, diagnostic)) = &material.diagnostics {
                bytes.vector(diagnostic)?;
            }
            if let Some(fields) = &material.force_fields {
                bytes.force(fields)?;
            }
        }
        if let Some(configured) = &s.configured {
            bytes.arc()?;
            bytes.add(size_of_val(configured.as_ref()))?;
            for field in [
                &configured.x,
                &configured.y,
                &configured.z,
                &configured.tracer,
            ] {
                bytes.vector(field)?;
            }
            bytes.vector(&configured.fraction)?;
            if let Some((_, pressure)) = &configured.interval_pressure {
                bytes.vector(pressure)?;
            }
            if let Some(receipt) = &configured.receipt {
                if let Some(binding) = &receipt.binding {
                    bytes.binding(binding)?;
                }
                if let Some(attempted) = &receipt.attempted {
                    bytes.strings([
                        &attempted.id,
                        &attempted.version,
                        &attempted.stage,
                        &attempted.domain,
                        &attempted.input.layout,
                        &attempted.input.scalar,
                        &attempted.input.units,
                        &attempted.output.layout,
                        &attempted.output.scalar,
                        &attempted.output.units,
                    ])?;
                }
                if let Some(
                    RecordedProviderRejection::Unsupported(text)
                    | RecordedProviderRejection::Execution(text),
                ) = &receipt.rejection
                {
                    bytes.add(text.capacity())?;
                }
            }
            if let Some(runtime) = &configured.runtime {
                bytes.binding(&runtime.request.binding)?;
                if let RuntimeOutcome::NoCandidate { detail, .. } = &runtime.outcome {
                    bytes.add(detail.capacity())?;
                }
            }
        }
        Ok(bytes.0)
    }

    /// Read one already verified raw/typed constructor selection under one lock.
    /// This acknowledges durable constructor custody, never an accepted frame.
    pub fn voxel_constructor_matches(&self, digest: [u8; 32]) -> Result<bool, String> {
        let s = self
            .0
            .lock()
            .map_err(|_| "Voxel constructor Inspector lock unavailable")?;
        let (Some(raw), Some(voxel), Some(_reader)) = (&s.record, &s.voxel, &s.reader) else {
            return Ok(false);
        };
        Ok(s.selected_step == Some(0)
            && raw.ordinal == 0
            && raw.generation == 0
            && raw.time.to_bits() == 0.0f64.to_bits()
            && raw.stage == rheon::SnapshotStage::Constructor
            && raw.dt.is_none()
            && raw.controls == "constructor; no accepted interval"
            && voxel.generation == 0
            && voxel.time.to_bits() == 0.0f64.to_bits()
            && voxel.dt.is_none()
            && voxel.interval_pressure.is_none()
            && voxel.metadata.initial_input_sha256 == digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_acknowledgment_requires_matching_typed_constructor_not_only_raw_record() {
        let (_, frozen) =
            crate::voxel_material_integration_tests::recipe(0.001, 1, [0.0; 3], 1024 * 1024);
        let digest = frozen.digest();
        let mut sim = frozen.construct().unwrap();
        let run = rheon::recording::voxel_material_run(
            SessionIdentity {
                run_id: 71,
                source_revision: "constructor-acknowledgment-test".into(),
            },
            &sim,
            digest,
            "owner acknowledgment fixture",
        )
        .unwrap();
        let path = super::super::tests::test_path("voxel-constructor-ack");
        let limits = frozen.execution().recording.limits();
        let mut writer = StepRecorder::create(&path, run, limits).unwrap();
        writer
            .append(
                &rheon::recording::voxel_material_publication(&sim, 0, limits.max_record_bytes)
                    .unwrap(),
            )
            .unwrap();
        let inspector = Inspector::default();
        inspector.reload(&path, limits).unwrap();
        // Archive/frame custody without typed selection cannot acknowledge.
        assert!(!inspector.voxel_constructor_matches(digest).unwrap());
        inspector.select_timestep(0).unwrap();
        assert!(inspector.voxel_constructor_matches(digest).unwrap());
        let mut wrong = digest;
        wrong[0] ^= 1;
        assert!(!inspector.voxel_constructor_matches(wrong).unwrap());
        let selected = inspector.0.lock().unwrap().voxel.take().unwrap();
        // Keep the actual raw constructor and reader, remove only typed custody.
        assert!(!inspector.voxel_constructor_matches(digest).unwrap());
        inspector.0.lock().unwrap().voxel = Some(selected);
        assert!(inspector.voxel_constructor_matches(digest).unwrap());
        sim.step(0.001, |_| false).unwrap();
        writer
            .append(
                &rheon::recording::voxel_material_publication(&sim, 1, limits.max_record_bytes)
                    .unwrap(),
            )
            .unwrap();
        inspector.select_timestep(1).unwrap();
        assert_eq!(inspector.with_voxel(|v| v.generation), Some(1));
        assert!(!inspector.voxel_constructor_matches(digest).unwrap());
        writer
            .finish("completed", "owner acknowledgment fixture", Some(1))
            .unwrap();
        drop(writer);
        drop(inspector);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn native_and_configured_profile_capacity_counts_spare_text_and_member_slots() {
        use crate::run_sets::{
            RunSetRevision,
            configured::{ConfiguredRunSetRevision, ConfiguredSetKind},
        };
        // Empty metadata owners deliberately exercise ownership without claiming
        // these incomplete profiles are valid runnable intents.
        let mut native = RunSetRevision {
            schema_version: 1,
            set_id: String::new(),
            parent_sha256: None,
            baseline_member: String::new(),
            members: Vec::new(),
        };
        let before = native.owned_capacity().unwrap();
        native.set_id.reserve_exact(1009);
        native.baseline_member.reserve_exact(503);
        native.members.reserve_exact(7);
        let added = native.set_id.capacity()
            + native.baseline_member.capacity()
            + native.members.capacity() * size_of::<crate::run_sets::RunSetMember>();
        assert_eq!(native.owned_capacity().unwrap(), before + added);

        let mut configured = ConfiguredRunSetRevision {
            schema_version: 2,
            kind: ConfiguredSetKind::ConfiguredAcceptedSessionSet,
            set_id: String::new(),
            parent_sha256: None,
            baseline_member: String::new(),
            members: Vec::new(),
        };
        let before = configured.owned_capacity().unwrap();
        configured.set_id.reserve_exact(1013);
        configured.baseline_member.reserve_exact(509);
        configured.members.reserve_exact(11);
        let added = configured.set_id.capacity()
            + configured.baseline_member.capacity()
            + configured.members.capacity()
                * size_of::<crate::run_sets::configured::ConfiguredRunSetMember>();
        assert_eq!(configured.owned_capacity().unwrap(), before + added);
    }

    #[test]
    fn retained_capacity_charges_legacy_raw_and_particle_spare_capacity() {
        let inspector = Inspector::default();
        {
            let mut state = inspector.0.lock().unwrap();
            state.material = Some(Arc::new(RecordedMaterial {
                generation: 0,
                time: 0.0,
                dt: None,
                particles: Vec::new(),
                diagnostics: Some((0.0, Vec::new())),
                force_fields: None,
            }));
            state.record = Some(Arc::new(RecordedStep {
                ordinal: 0,
                generation: 0,
                time: 0.0,
                dt: None,
                stage: rheon::SnapshotStage::Constructor,
                controls: String::new(),
                fields: Vec::new(),
            }));
        }
        let before = inspector.retained_payload_bytes().unwrap();
        let delta = {
            let mut state = inspector.0.lock().unwrap();
            let material = Arc::get_mut(state.material.as_mut().unwrap()).unwrap();
            material.particles.reserve_exact(257);
            let diagnostics = &mut material.diagnostics.as_mut().unwrap().1;
            diagnostics.reserve_exact(313);
            let added = material.particles.capacity() * size_of::<rheon::FluidParticle>()
                + diagnostics.capacity() * size_of::<rheon::ParticleDiagnostic>();
            let raw = Arc::get_mut(state.record.as_mut().unwrap()).unwrap();
            raw.controls.reserve_exact(1009);
            added + raw.controls.capacity()
        };
        assert_eq!(inspector.retained_payload_bytes().unwrap(), before + delta);
        // No reader/typed voxel selection can acknowledge this legacy fixture.
        assert!(!inspector.voxel_constructor_matches([0; 32]).unwrap());
    }

    #[test]
    fn retained_capacity_lock_poison_refuses_instead_of_claiming_empty() {
        let inspector = Arc::new(Inspector::default());
        let worker = Arc::clone(&inspector);
        assert!(
            std::thread::spawn(move || {
                let _guard = worker.0.lock().unwrap();
                panic!("intentional owner-lock fixture");
            })
            .join()
            .is_err()
        );
        assert!(
            inspector
                .retained_payload_bytes()
                .unwrap_err()
                .contains("lock")
        );
        assert!(inspector.voxel_constructor_matches([0; 32]).is_err());
    }
}
