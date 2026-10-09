//! --algebra evaluates explicitly SUPPLIED synthetic q, never a solve.
//! --physical executes one separately authorized body-force steady solve.
//! No reference velocity/traction is accepted or compiled into this provider.
#[path = "flat_wall_ritz/sha256.rs"]
mod sha256;
use rheon::*;
use std::{
    cell::Cell,
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Write},
    path::Path,
    rc::Rc,
    time::Instant,
};
const CAP: usize = FLAT_WALL_RITZ_ENVELOPE;
const FILE_CAP: usize = 1 << 20;
struct LimitedWriter {
    file: File,
    written: Rc<Cell<usize>>,
}
impl Write for LimitedWriter {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if self
            .written
            .get()
            .checked_add(b.len())
            .is_none_or(|n| n > FILE_CAP)
        {
            return Err(io::ErrorKind::FileTooLarge.into());
        }
        let n = self.file.write(b)?;
        self.written.set(self.written.get() + n);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
fn output(path: &Path, total: &Rc<Cell<usize>>) -> io::Result<BufWriter<LimitedWriter>> {
    Ok(BufWriter::with_capacity(
        8192,
        LimitedWriter {
            file: OpenOptions::new().write(true).create_new(true).open(path)?,
            written: Rc::clone(total),
        },
    ))
}
pub fn geometry(n: usize) -> Result<StaticObstacleGeometry, Box<dyn Error>> {
    if ![6, 9, 12].contains(&n) {
        return Err("unsupported roster".into());
    }
    let lower = [1.; 3];
    let upper = [2.; 3];
    let surface = TriangleSurface::new(
        SurfaceStamp {
            id: 20261009,
            version: 1,
        },
        (0..8)
            .map(|c| {
                std::array::from_fn(|d| {
                    if c & (1 << d) == 0 {
                        lower[d]
                    } else {
                        upper[d]
                    }
                })
            })
            .collect(),
        vec![
            [0, 2, 3],
            [0, 3, 1],
            [4, 5, 7],
            [4, 7, 6],
            [0, 1, 5],
            [0, 5, 4],
            [2, 6, 7],
            [2, 7, 3],
            [0, 4, 6],
            [0, 6, 2],
            [1, 3, 7],
            [1, 7, 5],
        ],
        SurfaceSettings::default(),
    )?;
    Ok(StaticObstacleGeometry::new(
        GridGeometry::new([n as u64; 3], [3. / n as f64; 3], [0.; 3])?,
        surface,
        CAP,
        |_, _| false,
    )?)
}
fn evidence(id: u64, hash: [u8; 32]) -> Result<ObstacleEvidenceRef, Box<dyn Error>> {
    Ok(ObstacleEvidenceRef::new(id, 0, hash)?)
}
fn inputs(source: ObstacleEvidenceRef) -> Result<ObstaclePhysicalInputs, Box<dyn Error>> {
    Ok(ObstaclePhysicalInputs{units:ObstacleStateUnits::Si,model:ObstacleStateModel::TransientStokes,
        material:ObstacleStateMaterial{density:1.,dynamic_viscosity:1.},
        problem:evidence(1,sha256::digest(&b"flat-wall lower strip physical body-force problem; SI rho=mu=1; [0,3]^3 minus[1,2]^3"[..])?)?,
        boundary_evidence:evidence(2,sha256::digest(&b"stationary no-slip solid; sealed/free-slip outer; outer shear even continuation"[..])?)?,
        boundary:ObstacleStateBoundary::StationaryNoSlipSolidSealedFreeSlipOuter,
        initial:ObstacleInitialData::KnownRest(evidence(3,sha256::digest(&b"declared coefficient rest; not the solved physical field"[..])?)?),initial_time:0.,
        forcing:ObstacleForcingHistory{kind:ObstacleForcingKind::DeclaredExternalForcing,evidence:source,start_time:0.,end_time:0.}})
}
fn source(path: &Path) -> Result<FlatWallPolynomialForce, Box<dyn Error>> {
    if fs::metadata(path)?.len() > 65536 {
        return Err("force source exceeds64KiB".into());
    }
    let digest = sha256::digest(File::open(path)?)?;
    let mut text = String::new();
    File::open(path)?.take(65537).read_to_string(&mut text)?;
    if text.len() > 65536 {
        return Err("force source grew beyond cap".into());
    }
    let count = text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .count();
    if count > 1024 {
        return Err("too many source terms".into());
    }
    let mut terms = Vec::new();
    terms.try_reserve_exact(count)?;
    for line in text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let mut parts = line.split(',');
        let axis = match parts.next() {
            Some("0") => Axis::X,
            Some("1") => Axis::Y,
            Some("2") => Axis::Z,
            _ => return Err("invalid force component".into()),
        };
        let coefficient = parts.next().ok_or("missing coefficient")?.parse()?;
        let mut powers = [0; 3];
        for v in &mut powers {
            *v = parts.next().ok_or("missing exponent")?.parse()?;
        }
        if parts.next().is_some() {
            return Err("extra force input".into());
        }
        terms.push(FlatWallForceTerm {
            component: axis,
            coefficient,
            powers,
        });
    }
    drop(text);
    Ok(FlatWallPolynomialForce::new(terms, evidence(4, digest)?)?)
}
fn axis(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
fn record(
    w: &mut impl Write,
    p: &FlatWallRitzProvider<'_>,
    q: &[f64],
    state: &ObstacleFlowState<'_>,
    mode: &str,
    head: &str,
) -> Result<(), Box<dyn Error>> {
    let plan = p.plan();
    let g = plan.geometry();
    let n = g.grid().counts()[0];
    let a = p.allocation();
    writeln!(
        w,
        "{{\"kind\":\"header\",\"mode\":\"{mode}\",\"head\":\"{head}\",\"n\":{n},\"h\":{:?},\"q\":{q:?},\"retained\":{},\"assembly_peak\":{},\"acquisition_peak\":{},\"source_sha256\":\"{}\",\"pressure_available\":false,\"physical_qualified\":false}}",
        g.grid().spacing()[0],
        a.retained_managed_bytes,
        a.assembly_peak_managed_bytes,
        a.acquisition_peak_managed_bytes,
        sha256::hex(p.source().evidence().sha256())
    )?;
    for term in p.source().terms() {
        writeln!(
            w,
            "{{\"kind\":\"source_term\",\"component\":{},\"coefficient\":{:?},\"powers\":{:?}}}",
            axis(term.component),
            term.coefficient,
            term.powers
        )?;
    }
    for (i, col) in plan.columns().iter().enumerate() {
        write!(
            w,
            "{{\"kind\":\"column\",\"index\":{i},\"node\":{:?},\"terms\":[",
            col.node
        )?;
        for (j, t) in col.terms.iter().enumerate() {
            if j != 0 {
                write!(w, ",")?;
            }
            write!(w, "[{},{},{:?}]", axis(t.component), t.face, t.coefficient)?;
        }
        writeln!(w, "]}}")?;
    }
    let grad = ObstacleVelocityGradient::new(state, plan.sites(), CAP, |_, _| false)?;
    for (i, row) in grad.rows().iter().enumerate() {
        let (cell, q) = match row.site {
            ObstacleGradientSite::Normal { cell, .. } => (cell, -1),
            ObstacleGradientSite::Cross { edge, quadrant, .. } => (edge, quadrant as i32),
            _ => unreachable!(),
        };
        write!(
            w,
            "{{\"kind\":\"row\",\"index\":{i},\"component\":{},\"derivative\":{},\"cell\":{cell:?},\"quadrant\":{q},\"weight\":{:?},\"endpoints\":[",
            axis(row.component),
            axis(row.derivative),
            row.weight
        )?;
        for (j, e) in row.endpoints.iter().enumerate() {
            if j != 0 {
                write!(w, ",")?;
            }
            let face = match e.source {
                ObstacleGradientSource::VelocityFace { face } => face as i64,
                _ => -1,
            };
            write!(
                w,
                "{{\"face\":{face},\"coefficient\":{:?},\"position\":{:?}}}",
                e.coefficient, e.position
            )?;
        }
        writeln!(w, "]}}")?;
    }
    drop(grad);
    let nq = q.len();
    for i in 0..nq {
        writeln!(
            w,
            "{{\"kind\":\"matrix\",\"index\":{i},\"values\":{:?},\"rhs\":{:?}}}",
            &p.matrix()[i * nq..(i + 1) * nq],
            p.rhs()[i]
        )?;
    }
    for a in [Axis::X, Axis::Y, Axis::Z] {
        for face in 0..g.grid().face_len(a) {
            let integral = p.source().integrate_face(g, a, face)?;
            if integral.clipped_geometric_volume != 0. {
                writeln!(
                    w,
                    "{{\"kind\":\"source_integral\",\"component\":{},\"face\":{face},\"force\":{:?},\"interval\":[{:?},{:?}],\"geometric_volume\":{:?},\"stored_volume\":{:?}}}",
                    axis(a),
                    integral.force,
                    integral.arithmetic_interval.lower,
                    integral.arithmetic_interval.upper,
                    integral.clipped_geometric_volume,
                    integral.stored_face_volume
                )?;
            }
            let v = state.velocity()[axis(a)][face];
            if v != 0. {
                writeln!(
                    w,
                    "{{\"kind\":\"velocity\",\"component\":{},\"face\":{face},\"value\":{v:?}}}",
                    axis(a)
                )?;
            }
        }
    }
    for (label, scheme) in [
        ("p1", FlatWallNormalTraction::FirstRowP1),
        ("normal_p2", FlatWallNormalTraction::TwoPlaneP2),
    ] {
        let r = flat_wall_owned_traction(state, scheme, |_, _| false)?;
        let fi = r.arithmetic_force.map(|i| [i.lower, i.upper]);
        let ti = r.arithmetic_torque.map(|i| [i.lower, i.upper]);
        writeln!(
            w,
            "{{\"kind\":\"traction\",\"scheme\":\"{label}\",\"force\":{:?},\"torque\":{:?},\"force_interval\":{fi:?},\"torque_interval\":{ti:?},\"wall_area\":{:?},\"basis_area\":{:?},\"sector_area\":{:?}}}",
            r.force,
            r.torque,
            r.geometric_wall_area,
            r.tangential_basis_area,
            r.energy_sector_effective_area
        )?;
    }
    let (d, bound) = flat_wall_divergence(g, state.velocity())?;
    writeln!(
        w,
        "{{\"kind\":\"divergence\",\"observed\":{d:?},\"arithmetic_bound\":{bound:?}}}"
    )?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 || !["--algebra", "--physical"].contains(&args[1].as_str()) {
        return Err("usage: flat_wall_ritz --algebra|--physical N FORCE_CSV NEW_OUT_DIR; physical mode requires separate approval".into());
    }
    let head = option_env!("RHEON_SOURCE_HEAD").ok_or("build must bind RHEON_SOURCE_HEAD")?;
    if head.len() != 40 || !head.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid bound source head".into());
    }
    let n: usize = args[2].parse()?;
    let out = Path::new(&args[4]);
    fs::create_dir(out)?;
    let source = source(Path::new(&args[3]))?;
    let inputs = inputs(source.evidence())?;
    let g = geometry(n)?;
    let argc_bytes = args.capacity() * std::mem::size_of::<String>()
        + args.iter().map(|s| s.capacity()).sum::<usize>();
    if argc_bytes > 8192 {
        return Err("argument payload exceeds8KiB".into());
    }
    let face_count = [Axis::X, Axis::Y, Axis::Z]
        .into_iter()
        .map(|a| g.grid().face_len(a))
        .sum::<usize>();
    let extras = 36 * 8
        + if args[1] == "--algebra" {
            face_count * 8
        } else {
            0
        }
        + argc_bytes
        + 65536
        + 2 * 8192;
    let provider_limit = CAP.checked_sub(extras).ok_or("runner budget overflow")?;
    let begin = Instant::now();
    let mut p = FlatWallRitzProvider::new(&g, source, inputs, provider_limit, |_, _| {
        begin.elapsed().as_secs() >= 180
    })?;
    // Runner budget includes copied owner, input/output journals and two8KiB
    // buffers; source text is already dropped. Never stores all row records.
    let mut q = Vec::new();
    q.try_reserve_exact(p.plan().columns().len())?;
    let mut v = if args[1] == "--algebra" {
        Some([
            vec![0.; g.grid().face_len(Axis::X)],
            vec![0.; g.grid().face_len(Axis::Y)],
            vec![0.; g.grid().face_len(Axis::Z)],
        ])
    } else {
        None
    };
    let mut solver_report = None;
    let mode = if args[1] == "--algebra" {
        q.extend((0..p.plan().columns().len()).map(|i| (i + 1) as f64 / 64.));
        let [x, y, z] = v.as_mut().unwrap();
        p.plan().apply_flux_curl(&q, [x, y, z])?;
        "supplied_synthetic_q_algebra_no_solve"
    } else {
        solver_report = Some(p.solve(|_, _| begin.elapsed().as_secs() >= 180)?);
        q.extend_from_slice(p.solved_coefficients()?);
        "numerical_reduced_ritz_solve"
    };
    let total = Rc::new(Cell::new(0));
    let journal_path = out.join("acquisition.txt");
    {
        let binary_hash = sha256::digest(File::open(std::env::current_exe()?)?)?;
        let mut journal = output(&journal_path, &total)?;
        writeln!(
            journal,
            "mode={mode}\nsource_head={head}\nbinary_sha256={}\nsource_sha256={}\nproblem_sha256={}\nboundary_sha256={}\nrho=1\nmu=1\ntime=0\ngeneration=0\npressure_available=false\nerrors=Unknown\ngeometry=counts:{:?},spacing:{:?},origin:{:?},box:{:?},stamp:{:?}\ncoverage={:?}\nsolver_report={solver_report:?}\nq={q:?}",
            sha256::hex(binary_hash),
            sha256::hex(p.source().evidence().sha256()),
            sha256::hex(inputs.problem.sha256()),
            sha256::hex(inputs.boundary_evidence.sha256()),
            g.grid().counts(),
            g.grid().spacing(),
            g.grid().origin(),
            g.box_bounds(),
            g.stamp(),
            p.coverage()
        )?;
        for (i, c) in p.plan().columns().iter().enumerate() {
            writeln!(journal, "C,{i},{c:?}")?;
        }
        for (i, row) in p.matrix().chunks(q.len()).enumerate() {
            writeln!(journal, "matrix,{i},{row:?}\nrhs,{i},{:?}", p.rhs()[i])?;
        }
        let velocity = if let Some(v) = &v {
            [&v[0][..], &v[1][..], &v[2][..]]
        } else {
            p.solved_velocity()?
        };
        for (a, values) in velocity.into_iter().enumerate() {
            for (face, &value) in values.iter().enumerate() {
                if value != 0. {
                    writeln!(journal, "v,{a},{face},{value:?}")?;
                }
            }
        }
        journal.flush()?;
    }
    let acquisition = evidence(5, sha256::digest(File::open(&journal_path)?)?)?;
    let state = if let Some(v) = &v {
        let mut supplied = inputs;
        supplied.initial = ObstacleInitialData::Supplied(acquisition);
        ObstacleFlowState::new(
            &g,
            supplied,
            ObstacleStateFrame {
                time: 0.,
                generation: 0,
                origin: ObstacleStateOrigin::InitialData,
                errors: ObstacleVelocityErrors::unknown(),
            },
            [&v[0], &v[1], &v[2]],
            CAP,
        )?
    } else {
        p.acquire_initial_state(acquisition)?
    };
    let actual_extras = q.capacity() * 8
        + v.as_ref()
            .map_or(0, |v| v.iter().map(|v| v.capacity() * 8).sum::<usize>())
        + argc_bytes
        + 65536
        + 2 * 8192;
    if actual_extras > extras {
        return Err("unexpected runner capacity".into());
    }
    let bound = p.allocation().acquisition_peak_managed_bytes + actual_extras;
    if bound > CAP {
        return Err("runner managed payload refusal".into());
    }
    let mut data = output(&out.join("records.jsonl"), &total)?;
    record(&mut data, &p, &q, &state, mode, head)?;
    if let Some(r) = solver_report {
        let d = p.diagnose_owned(&state, |_, _| begin.elapsed().as_secs() >= 180)?;
        writeln!(
            data,
            "{{\"kind\":\"solve\",\"projected_residual\":{:?},\"minimum_pivot\":{:?},\"full_active_residual\":{:?},\"matrix_action_defect\":{:?},\"work\":{:?},\"dissipation\":{:?}}}",
            r.projected_residual_max,
            r.minimum_pivot,
            d.full_active_face_residual_max,
            d.matrix_action_discrepancy_max,
            d.work.force_work,
            d.work.dissipation
        )?;
    }
    writeln!(
        data,
        "{{\"kind\":\"runner_bound\",\"managed_peak\":{bound},\"geometry_constructor_peak\":{},\"elapsed_seconds\":{:?},\"stack_allocator_RSS_excluded\":true}}",
        g.allocation().constructor_peak_bytes,
        begin.elapsed().as_secs_f64()
    )?;
    data.flush()?;
    drop(data);
    let mut checkpoint = output(&out.join("owned-state.bin"), &total)?;
    state.write_checkpoint(&mut checkpoint)?;
    checkpoint.flush()?;
    Ok(())
}
