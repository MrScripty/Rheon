use super::*;
const N: usize = 32;
const CELLS: usize = N * N * N;

/// Native f64 execution of the real lab's frozen ModalPressure forward pass.
/// No inverse eigenvalues, optimizer, reference pressure or analytic solve enter
/// inference. Export with tools/export_ai_pressure.py from the pinned checkpoint.
/// Memory: 270328 model bytes + two 262144-byte scratch arrays, excluding allocator
/// overhead and the caller-owned input/output. No allocations during predict.
pub struct LabModalPressure {
    basis: Vec<f64>,
    gains: Vec<f64>,
    a: Vec<f64>,
    b: Vec<f64>,
}
impl LabModalPressure {
    /// Accept the fixed export layout and an accidental-corruption checksum.
    /// FNV is NOT authentication: the exporter verifies checkpoint SHA256 before
    /// extraction. Only use exports from that tool and a trusted source checkout.
    pub fn from_export(bytes: &[u8]) -> Result<Self, ProviderRejection> {
        if bytes.len() != 8 + (N * N + CELLS - 1) * 8 || &bytes[..8] != b"RHEONAI1" {
            return Err(ProviderRejection::Execution(
                "invalid frozen artifact layout",
            ));
        }
        let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |h, v| {
            (h ^ u64::from(*v)).wrapping_mul(0x100000001b3)
        });
        if hash != 0x10cd0171c4128275 {
            return Err(ProviderRejection::Execution(
                "frozen artifact checksum mismatch",
            ));
        }
        let decode = |bytes: &[u8]| -> Result<Vec<f64>, ProviderRejection> {
            let mut values = Vec::new();
            values
                .try_reserve_exact(bytes.len() / 8)
                .map_err(|_| ProviderRejection::Execution("model allocation"))?;
            for v in bytes.chunks_exact(8) {
                values.push(f64::from_le_bytes(v.try_into().unwrap()));
            }
            Ok(values)
        };
        let basis = decode(&bytes[8..8 + N * N * 8])?;
        let gains = decode(&bytes[8 + N * N * 8..])?;
        if basis.iter().chain(&gains).any(|v| !v.is_finite()) || gains.iter().any(|v| *v <= 0.0) {
            return Err(ProviderRejection::NonFiniteOutput);
        }
        let allocate = || {
            let mut values = Vec::new();
            values
                .try_reserve_exact(CELLS)
                .map_err(|_| ProviderRejection::Execution("model scratch allocation"))?;
            values.resize(CELLS, 0.0);
            Ok(values)
        };
        Ok(Self {
            basis,
            gains,
            a: allocate()?,
            b: allocate()?,
        })
    }
    fn transform(
        &mut self,
        inverse: bool,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        for stride in [1, N, N * N] {
            if cancelled() {
                return Err(ProviderRejection::Cancelled);
            }
            for index in 0..CELLS {
                let coordinate = (index / stride) % N;
                let base = index - coordinate * stride;
                let mut value = 0.0;
                for k in 0..N {
                    let weight = if inverse {
                        self.basis[k * N + coordinate]
                    } else {
                        self.basis[coordinate * N + k]
                    };
                    value += self.a[base + k * stride] * weight;
                }
                self.b[index] = value;
            }
            std::mem::swap(&mut self.a, &mut self.b);
        }
        Ok(())
    }
}
impl PressureProvider for LabModalPressure {
    fn metadata(&self) -> SolverMetadata {
        SolverMetadata {
            id: "lab-modal-pressure-v1",
            version: "1",
            stage: "pressure",
            input: TensorMetadata {
                shape_xyz: [N; 3],
                layout: "x-fastest-cell-centered",
                scalar: "f64",
                units: "m^3/s^2",
            },
            output: TensorMetadata {
                shape_xyz: [N; 3],
                layout: "x-fastest-cell-centered",
                scalar: "f64",
                units: "Pa",
            },
            domain: "sealed uniform unit box; density 1.25 kg/m^3; dt 1/60 s; zero-cell gauge",
            provenance: Some(ModelProvenance {
                repository: "https://github.com/MrScripty/rheon-surrogate-lab",
                commit: "e8a90bfef6b2955c502008ccc62443e29dd9f2ec",
                checkpoint_sha256: "25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9",
                export_sha256: "902ad8d77b16ae42f22f90bf71c22c19ae2e3cbe0cc0f4cc3d770930d4049aea",
                runtime: "rheon-native-f64-cpu-v1",
            }),
        }
    }
    fn supports(&self, request: &PressureRequest<'_>) -> Result<(), ProviderRejection> {
        if request.grid.counts() != [N; 3]
            || request.grid.spacing() != [1.0 / N as f64; 3]
            || request.grid.origin() != [0.0; 3]
            || request.density != 1.25
            || request.dt != 1.0 / 60.0
        {
            return Err(ProviderRejection::Unsupported(
                "frozen 32^3 unit-box operator only",
            ));
        }
        Ok(())
    }
    fn predict(
        &mut self,
        request: &PressureRequest<'_>,
        pressure: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<(), ProviderRejection> {
        self.supports(request)?;
        if request.rhs.len() != CELLS
            || pressure.len() != CELLS
            || request.rhs.iter().any(|v| !v.is_finite())
        {
            return Err(ProviderRejection::Execution("invalid tensor"));
        }
        self.a.copy_from_slice(request.rhs);
        self.transform(false, cancelled)?;
        self.a[0] = 0.0;
        for (v, gain) in self.a[1..].iter_mut().zip(&self.gains) {
            *v *= gain;
        }
        self.transform(true, cancelled)?;
        let gauge = self.a[0];
        for (out, value) in pressure.iter_mut().zip(&self.a) {
            *out = value - gauge;
        }
        Ok(())
    }
}
