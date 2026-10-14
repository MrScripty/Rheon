//! Deterministic raw-array I/O probe, not a solver/numerical qualification.
//! Run release binary under /usr/bin/time separately for write and read peak RSS.
use rheon::{SessionIdentity, SnapshotStage, recording::*};
use std::{error::Error, path::Path, time::Instant};
fn field(name: &str, dims: [u64; 3], width: usize, generation: usize) -> RecordedField {
    let tuples = dims.iter().product::<u64>() as usize;
    let mut data = Vec::with_capacity(tuples * width);
    for i in 0..tuples {
        let seed = (i as u32)
            .wrapping_mul(1664525)
            .wrapping_add(generation as u32);
        if width == 4 {
            data.extend_from_slice(&f32::from_bits(0x3f000000 | (seed & 0x007fffff)).to_le_bytes());
        } else {
            data.extend_from_slice(
                &f64::from_bits(0x3fe0000000000000 | ((seed as u64) << 20)).to_le_bytes(),
            );
        }
    }
    RecordedField {
        name: name.into(),
        units: if name == "tracer" {
            "1"
        } else if width == 8 {
            "Pa"
        } else {
            "m/s"
        }
        .into(),
        association: format!("synthetic {name}; x-fast"),
        stage: if generation == 0 {
            "constructor"
        } else {
            "accepted"
        }
        .into(),
        evaluation_time: None,
        dimensions: dims,
        components: 1,
        scalar_type: if width == 4 { "f32" } else { "f64" }.into(),
        data,
    }
}
fn percentile(mut data: Vec<f64>, p: f64) -> f64 {
    data.sort_by(f64::total_cmp);
    data[((data.len() - 1) as f64 * p).ceil() as usize]
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: recording_storage_probe write|read path edge accepted_steps".into());
    }
    let p = Path::new(&args[2]);
    let n: u64 = args[3].parse()?;
    let steps: usize = args[4].parse()?;
    if ![32, 64].contains(&n) || !(1..=64).contains(&steps) {
        return Err("bounded probe requires 32/64 and1..64 accepted steps".into());
    }
    let limits = RecordingLimits {
        max_database_bytes: 1024 * 1024 * 1024,
        max_records: steps + 1,
        ..Default::default()
    };
    if args[1] == "write" {
        let run=RecordingRun {identity:SessionIdentity {run_id:n,source_revision: "synthetic-storage-probe; not physics".into()},
            solver:"raw-array I/O probe".into(),configuration:format!("edge={n},accepted_steps={steps},no compression,FULL synchronous"),
            counts:[n;3],spacing:[1.0/n as f64;3],origin:[0.0;3],provenance:"release; deterministic finite synthetic f32 faces/tracer and f64 pressure; not solver evidence".into()};
        let mut writer = StepRecorder::create(p, run, limits)?;
        let mut encode = Vec::new();
        let mut commits = Vec::new();
        let mut payload = 0;
        for generation in 0..=steps {
            let begin = Instant::now();
            let mut fields = vec![
                field("velocity_x", [n + 1, n, n], 4, generation),
                field("velocity_y", [n, n + 1, n], 4, generation),
                field("velocity_z", [n, n, n + 1], 4, generation),
                field("tracer", [n; 3], 4, generation),
            ];
            if generation > 0 {
                fields.push(field("interval_pressure", [n; 3], 8, generation));
            }
            payload = fields.iter().map(|f| f.data.len()).sum::<usize>();
            let step = RecordedStep {
                ordinal: generation,
                generation: generation as u64,
                time: generation as f64 / 64.0,
                dt: (generation > 0).then_some(1.0 / 64.0),
                stage: if generation == 0 {
                    SnapshotStage::Constructor
                } else {
                    SnapshotStage::Accepted
                },
                controls: "synthetic generation; no forces".into(),
                fields,
            };
            let encoded = begin.elapsed().as_secs_f64() * 1000.0;
            let begin = Instant::now();
            writer.append(&step)?;
            let committed = begin.elapsed().as_secs_f64() * 1000.0;
            if generation > 0 {
                encode.push(encoded);
                commits.push(committed);
            }
        }
        writer.finish(
            "completed",
            "all synthetic publications recorded",
            Some(steps as u64),
        )?;
        drop(writer);
        println!(
            "mode=write edge={n} records={} accepted_payload_bytes={payload} database_bytes={} encode_median_ms={:.3} encode_p95_ms={:.3} append_median_ms={:.3} append_p95_ms={:.3}",
            steps + 1,
            std::fs::metadata(p)?.len(),
            percentile(encode.clone(), 0.5),
            percentile(encode, 0.95),
            percentile(commits.clone(), 0.5),
            percentile(commits, 0.95)
        );
    } else if args[1] == "read" {
        let begin = Instant::now();
        let reader = RecordingReader::open(p, limits)?;
        let open_ms = begin.elapsed().as_secs_f64() * 1000.0;
        let mut reads = Vec::new();
        let mut bytes = 0;
        for ordinal in (1..=steps).rev() {
            let begin = Instant::now();
            let step = reader.select(ordinal)?;
            let elapsed = begin.elapsed().as_secs_f64() * 1000.0;
            // Verify every selected raw bit against independent deterministic inventory.
            for f in &step.fields {
                let expected = field(
                    &f.name,
                    f.dimensions,
                    if f.scalar_type == "f64" { 8 } else { 4 },
                    ordinal,
                );
                if f.data != expected.data {
                    return Err("raw precision mismatch".into());
                }
                bytes += f.data.len();
            }
            reads.push(elapsed);
        }
        let begin = Instant::now();
        reader.verify()?;
        let audit_ms = begin.elapsed().as_secs_f64() * 1000.0;
        println!(
            "mode=read edge={n} open_quick_check_ms={open_ms:.3} read_bytes={bytes} select_verified_median_ms={:.3} select_verified_p95_ms={:.3} audit_ms={audit_ms:.3}",
            percentile(reads.clone(), 0.5),
            percentile(reads, 0.95)
        );
    } else {
        return Err("unknown mode".into());
    }
    if let Ok(status) = std::fs::read_to_string("/proc/self/status")
        && let Some(peak) = status.lines().find(|line| line.starts_with("VmHWM:"))
    {
        println!("linux_binary_{peak}");
    }

    Ok(())
}
