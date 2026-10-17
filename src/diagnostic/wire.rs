//! Current named-port protocol projection shared by desktop and lab callers.
use super::{Input, PredictionFailure};
use crate::recording::diagnostic::NoCandidateReason;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSemantic {
    Rhs,
    PreviousPressure,
    InitialResidual,
    X,
    Y,
    Z,
    Density,
    Dt,
}

fn prediction_failure(
    reason: NoCandidateReason,
    detail: impl Into<String>,
    dispatched: bool,
    invalid_payload_sha256: Option<[u8; 32]>,
) -> PredictionFailure {
    PredictionFailure {
        reason,
        detail: detail
            .into()
            .chars()
            .filter(|c| !c.is_control())
            .take(2000)
            .collect(),
        dispatched,
        invalid_payload_sha256,
    }
}
impl Input<'_> {
    /// Project the validated physical context into the current named-port wire.
    /// This pure operation neither loads a package nor starts a process. The
    /// caller owns package support validation and the bounded wire transport.
    pub fn adapter_request(
        &self,
        inputs: &BTreeMap<String, InputSemantic>,
    ) -> Result<Value, PredictionFailure> {
        let fail = || {
            prediction_failure(
                NoCandidateReason::UnsupportedContext,
                "Invalid named-port request shape, values or identity",
                false,
                None,
            )
        };
        let c = &self.context;
        let id = usize::try_from(c.generation()).map_err(|_| fail())?;
        if c.generation() >= (1_u64 << 53)
            || c.grid().cell_len() > 65_536
            || self.residual.len() != c.grid().cell_len()
            || self.residual.iter().any(|x| !x.is_finite())
            || inputs.is_empty()
            || inputs.len() > 32
            || inputs.keys().any(|name| {
                name.is_empty()
                    || name.len() > 128
                    || !name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
            })
        {
            return Err(fail());
        }
        let counts = c.grid().counts();
        let mut fields = serde_json::Map::new();
        for (name, semantic) in inputs {
            let (values, units, scalar) = match semantic {
                InputSemantic::Rhs => (c.rhs().to_vec(), "m3/s2", false),
                InputSemantic::InitialResidual => (self.residual.to_vec(), "m3/s2", false),
                InputSemantic::PreviousPressure => (
                    c.prior()
                        .ok_or_else(|| {
                            prediction_failure(
                                NoCandidateReason::UnsupportedContext,
                                "Previous pressure unavailable",
                                false,
                                None,
                            )
                        })?
                        .values
                        .to_vec(),
                    "Pa",
                    false,
                ),
                InputSemantic::Density => (vec![c.density()], "kg/m3", true),
                InputSemantic::Dt => (vec![c.dt()], "s", true),
                InputSemantic::X | InputSemantic::Y | InputSemantic::Z => {
                    let axis = match semantic {
                        InputSemantic::X => 0,
                        InputSemantic::Y => 1,
                        _ => 2,
                    };
                    let mut values = Vec::with_capacity(c.grid().cell_len());
                    for k in 0..counts[2] {
                        for j in 0..counts[1] {
                            for i in 0..counts[0] {
                                values.push(
                                    c.grid().cell_position([i, j, k]).ok_or_else(|| {
                                        prediction_failure(
                                            NoCandidateReason::UnsupportedContext,
                                            "Coordinate unavailable",
                                            false,
                                            None,
                                        )
                                    })?[axis],
                                );
                            }
                        }
                    }
                    (values, "m", false)
                }
            };
            fields.insert(name.clone(),json!({"semantic":semantic,"units":units,"association":if scalar {"global"}else{"cell"},"dimensions":if scalar {[1,1,1]}else{counts},"scalar":"f64","layout":"x_fast","values":values}));
        }
        Ok(
            json!({"kind":"predict","schema":"rheon-pressure-context-request-v1","id":id,"stage":crate::solver_provider::context::STAGE_CONTRACT,
        "context":{"run_id":c.run_id(),"generation":c.generation(),"mode":match c.mode(){crate::solver_provider::context::PressureContextMode::AcceptedCandidate=>"accepted_candidate",crate::solver_provider::context::PressureContextMode::DiagnosticCandidate=>"diagnostic_candidate"},"interval_start_s":c.interval_start(),"interval_end_s":c.interval_end(),"dt_s":c.dt(),"grid":{"counts_xyz":counts,"spacing_xyz_m":c.grid().spacing(),"origin_xyz_m":c.grid().origin()},
        "prior":c.prior().map(|p|json!({"generation":p.generation,"interval_end_s":p.interval_end,"status":match p.status {crate::solver_provider::context::PressureHistoryStatus::Accepted=>json!({"kind":"accepted"}),crate::solver_provider::context::PressureHistoryStatus::Diagnostic{qualified}=>json!({"kind":"diagnostic","qualified":qualified})}}))},"inputs":fields}),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputRole {
    FullPressure,
    PressureCorrection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterErrorCode {
    Invalid,
    Unavailable,
    Unsupported,
    UnsupportedContext,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AdapterReply {
    AdapterReady {
        id: String,
        version: String,
        manifest_sha256: String,
        device: String,
        optimizer_updates: u64,
    },
    Candidate {
        id: usize,
        role: OutputRole,
        #[serde(deserialize_with = "bounded_pressure")]
        pressure: Vec<f64>,
        device: String,
        optimizer_updates: u64,
    },
    Loading {
        phase: String,
    },
    Error {
        code: AdapterErrorCode,
        id: Option<usize>,
        message: String,
    },
}
fn bounded_pressure<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<f64>, D::Error> {
    struct Values;
    impl<'de> serde::de::Visitor<'de> for Values {
        type Value = Vec<f64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most 65,536 finite pressure values")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while let Some(value) = seq.next_element::<f64>()? {
                if values.len() == 65_536 || !value.is_finite() {
                    return Err(serde::de::Error::custom(
                        "Pressure array exceeds finite value budget",
                    ));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(Values)
}

impl AdapterErrorCode {
    /// Bounded detail shared by startup and prediction protocol owners.
    pub fn detail(self, message: String) -> String {
        format!(
            "CPU bridge {self:?}: {}",
            message
                .chars()
                .filter(|c| !c.is_control())
                .take(2000)
                .collect::<String>()
        )
    }
}
impl Input<'_> {
    /// Decode one complete current reply line, retaining its exact invalid-wire
    /// digest. The caller owns transport cancellation, deadlines and reaping.
    pub fn adapter_reply(
        &self,
        raw: &[u8],
        expected_role: OutputRole,
    ) -> Result<Vec<f64>, PredictionFailure> {
        use sha2::{Digest, Sha256};
        if raw.len() > 8 * 1024 * 1024 || !raw.ends_with(b"\n") {
            return Err(prediction_failure(
                NoCandidateReason::InvalidOutput,
                "CPU reply requires a bounded complete line",
                true,
                None,
            ));
        }
        let digest: [u8; 32] = Sha256::digest(raw).into();
        let reply = serde_json::from_slice(raw).map_err(|_| {
            prediction_failure(
                NoCandidateReason::InvalidOutput,
                "Invalid current CPU adapter reply",
                true,
                Some(digest),
            )
        })?;
        self.validate_adapter_reply(reply, expected_role, digest)
    }
    /// Validate an already decoded current reply using the same reconstruction
    /// owner as `adapter_reply`; process hosts retain the original packet digest.
    pub fn validate_adapter_reply(
        &self,
        reply: AdapterReply,
        expected_role: OutputRole,
        payload_sha256: [u8; 32],
    ) -> Result<Vec<f64>, PredictionFailure> {
        let request_id = usize::try_from(self.context.generation()).map_err(|_| {
            prediction_failure(
                NoCandidateReason::UnsupportedContext,
                "Request id overflow",
                false,
                None,
            )
        })?;
        let n = self.context.grid().cell_len();
        let after = |reason, detail| prediction_failure(reason, detail, true, None);
        let digest = Some(payload_sha256);
        match reply {
            AdapterReply::Error {
                code,
                id: Some(id),
                message,
            } if id == request_id => {
                let reason = match code {
                    AdapterErrorCode::Unsupported => NoCandidateReason::Unsupported,
                    AdapterErrorCode::UnsupportedContext | AdapterErrorCode::Invalid => {
                        NoCandidateReason::UnsupportedContext
                    }
                    AdapterErrorCode::Unavailable => NoCandidateReason::RuntimeFailure,
                };
                Err(prediction_failure(reason, code.detail(message), true, None))
            }
            AdapterReply::Candidate {
                id,
                role,
                mut pressure,
                device,
                optimizer_updates,
            } if id == request_id
                && expected_role == role
                && device == "cpu"
                && optimizer_updates == 0
                && pressure.len() == n
                && pressure.first() == Some(&0.0)
                && pressure.iter().all(|p| p.is_finite()) =>
            {
                if role == OutputRole::PressureCorrection {
                    let prior = self.context.prior().ok_or_else(|| {
                        after(
                            NoCandidateReason::UnsupportedContext,
                            "Correction requires prior pressure",
                        )
                    })?;
                    for (p, previous) in pressure.iter_mut().zip(prior.values) {
                        *p += *previous;
                        if !p.is_finite() {
                            return Err(prediction_failure(
                                NoCandidateReason::InvalidOutput,
                                "Nonfinite reconstructed pressure",
                                true,
                                digest,
                            ));
                        }
                    }
                }
                Ok(pressure)
            }
            _ => Err(prediction_failure(
                NoCandidateReason::InvalidOutput,
                "CPU pressure reply shape/gauge/correlation mismatch",
                true,
                digest,
            )),
        }
    }
}
