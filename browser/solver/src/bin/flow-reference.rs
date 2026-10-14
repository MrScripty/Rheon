//! Export actual native states; reference data are never a browser trajectory.
use rheon_solver::flow::{FIXTURE, FlowOutcome, FlowSimulation, FlowSnapshot, MAX_STEPS, N};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

fn frame(
    sink: &mut impl Write,
    offset: &mut u64,
    kind: &str,
    outcome: FlowOutcome,
) -> Result<Value, Box<dyn std::error::Error>> {
    let FlowOutcome {
        state,
        report,
        status,
        code,
        detail,
    } = outcome;
    let start = *offset;
    for field in [
        &state.bits.x,
        &state.bits.y,
        &state.bits.z,
        &state.bits.tracer,
    ] {
        for bits in field {
            sink.write_all(&bits.to_le_bytes())?;
            *offset += 4;
        }
    }
    let mut summary = serde_json::to_value(state)?;
    let object = summary
        .as_object_mut()
        .ok_or("snapshot must be an object")?;
    for key in ["x", "y", "z", "tracer", "bits"] {
        object.remove(key);
    }
    Ok(
        json!({"kind":kind,"status":status,"code":code,"detail":detail,"byte_offset":start,"byte_length":*offset-start,"state":summary,"report":report}),
    )
}

fn snapshot(state: FlowSnapshot) -> FlowOutcome {
    FlowOutcome {
        state,
        report: None,
        status: "snapshot",
        code: "snapshot",
        detail: None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err("usage: flow-reference <fresh-output-directory>".into());
    }
    let out = Path::new(&args[1]);
    fs::create_dir(out)?;
    let mut sink = BufWriter::new(File::create(out.join("native-flow-fields.bin"))?);
    let mut offset = 0_u64;
    let mut runs = Vec::new();
    for method in ["jacobi-pcg-v1", "sgs-pcg-v1"] {
        let mut sim = FlowSimulation::new(method)?;
        let mut frames = vec![frame(
            &mut sink,
            &mut offset,
            "baseline",
            snapshot(sim.state()),
        )?];
        let cancelled_rest = sim.advance(true);
        if cancelled_rest.code != "cancelled_before_commit" {
            return Err("native rest cancellation failed".into());
        }
        let cancelled_rest = frame(&mut sink, &mut offset, "cancelled_rest", cancelled_rest)?;
        let mut cancelled_nonzero = None;
        for step in 1..=MAX_STEPS {
            let outcome = sim.advance(false);
            if outcome.status != "accepted" {
                return Err(format!("native flow step {step} failed: {:?}", outcome.detail).into());
            }
            frames.push(frame(&mut sink, &mut offset, "accepted", outcome)?);
            if step == 30 {
                let cancelled = sim.advance(true);
                if cancelled.code != "cancelled_before_commit" {
                    return Err("native nonzero cancellation failed".into());
                }
                cancelled_nonzero = Some(frame(
                    &mut sink,
                    &mut offset,
                    "cancelled_nonzero",
                    cancelled,
                )?);
            }
        }
        let limit = sim.advance(false);
        if limit.code != "step_limit" {
            return Err("native flow exceeded its accepted step cap".into());
        }
        let limit_rejection = frame(&mut sink, &mut offset, "limit_rejection", limit)?;
        let reset = frame(
            &mut sink,
            &mut offset,
            "reset",
            snapshot(sim.reset_state()?),
        )?;
        let retried = sim.advance(false);
        if retried.status != "accepted" {
            return Err("native retry after reset failed".into());
        }
        let after_reset = frame(&mut sink, &mut offset, "after_reset", retried)?;
        runs.push(json!({"method":method,"frames":frames,"cancelled_rest":cancelled_rest,"cancelled_nonzero":cancelled_nonzero,"limit_rejection":limit_rejection,"reset":reset,"after_reset":after_reset}));
    }
    sink.flush()?;
    let face_len = (N + 1) * N * N;
    let reference = json!({"schema":1,"fixture":FIXTURE,"encoding":{"format":"little-endian-f32","field_order":["x","y","z","tracer"],"field_lengths":[face_len,face_len,face_len,N*N*N],"bytes_per_state":4*(3*face_len+N*N*N)},"binary_bytes":offset,"runs":runs});
    let mut manifest = BufWriter::new(File::create(out.join("native-flow-reference.json"))?);
    serde_json::to_writer_pretty(&mut manifest, &reference)?;
    manifest.write_all(b"\n")?;
    manifest.flush()?;
    Ok(())
}
