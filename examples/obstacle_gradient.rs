//! Explicit checkpoint/request/test consumer. No manufactured velocity generation.
#[path = "viscous_boundary_wrench.rs"]
#[allow(dead_code)]
mod fixture;
use rheon::*;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
fn axis(s: &str) -> Result<Axis, Box<dyn Error>> {
    Ok(*AXES.get(s.parse::<usize>()?).ok_or("axis must be 0,1,2")?)
}
fn bounded_text(file: &Path) -> Result<String, Box<dyn Error>> {
    const MAX_TEXT: u64 = 65_536;
    let mut text = String::new();
    File::open(file)?
        .take(MAX_TEXT + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > MAX_TEXT {
        return Err("bounded input text exceeds 65536 bytes".into());
    }
    Ok(text)
}
fn requests(file: &Path) -> Result<(Vec<ObstacleGradientSite>, usize), Box<dyn Error>> {
    let text = bounded_text(file)?;
    let mut sites = Vec::new();
    let mut parse_peak = text.capacity();
    for line in text.lines() {
        let t = line.split_whitespace().collect::<Vec<_>>();
        if t.is_empty() {
            continue;
        }
        sites.push(match t[0] {
            "N" if t.len() == 5 => ObstacleGradientSite::Normal {
                axis: axis(t[1])?,
                cell: [t[2].parse()?, t[3].parse()?, t[4].parse()?],
            },
            "C" if t.len() == 7 => ObstacleGradientSite::Cross {
                component: axis(t[1])?,
                derivative: axis(t[2])?,
                edge: [t[3].parse()?, t[4].parse()?, t[5].parse()?],
                quadrant: t[6].parse()?,
            },
            "CF" if t.len() == 1 => ObstacleGradientSite::CoarseFine,
            _ => {
                return Err(
                    "request must be N axis i j k / C component derivative i j k quadrant / CF"
                        .into(),
                );
            }
        });
        parse_peak = parse_peak.max(
            text.capacity()
                + t.capacity() * std::mem::size_of::<&str>()
                + sites.capacity() * std::mem::size_of::<ObstacleGradientSite>(),
        );
    }
    Ok((sites, parse_peak))
}
fn array(w: &mut impl Write, values: &[f64]) -> Result<(), Box<dyn Error>> {
    write!(w, "[")?;
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            write!(w, ",")?;
        }
        write!(w, "{v}")?;
    }
    write!(w, "]")?;
    Ok(())
}
fn geometry(kind: &str) -> Result<StaticObstacleGeometry, Box<dyn Error>> {
    if kind == "nonmidpoint" {
        return fixture::geometry([6; 3], [0.3, 0.7, 0.2], [0.1, -0.3, 1.1], [2; 3], [4; 3]);
    }
    let n: usize = kind.parse()?;
    if ![3, 6, 12].contains(&n) {
        return Err("only N3/6/12 or nonmidpoint fixtures".into());
    }
    fixture::geometry(
        [n as u64; 3],
        [3. / n as f64; 3],
        [0.; 3],
        [n / 3; 3],
        [2 * n / 3; 3],
    )
}
fn integers(w: &mut impl Write, values: &[usize]) -> Result<(), Box<dyn Error>> {
    write!(w, "[")?;
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            write!(w, ",")?;
        }
        write!(w, "{v}")?;
    }
    write!(w, "]")?;
    Ok(())
}
fn describe_geometry(kind: &str, file: &Path) -> Result<(), Box<dyn Error>> {
    if file
        .parent()
        .ok_or("geometry output needs parent")?
        .canonicalize()?
        .starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize()?)
    {
        return Err("geometry evidence must stay outside Git".into());
    }
    let g = geometry(kind)?;
    let mut w = BufWriter::with_capacity(
        8192,
        OpenOptions::new().write(true).create_new(true).open(file)?,
    );
    write!(w, "{{\"counts\":")?;
    integers(&mut w, &g.grid().counts())?;
    for (name, values) in [
        ("origin", g.grid().origin()),
        ("spacing", g.grid().spacing()),
        ("lower", g.box_bounds().0),
        ("upper", g.box_bounds().1),
    ] {
        write!(w, ",\"{name}\":")?;
        array(&mut w, &values)?;
    }
    write!(
        w,
        ",\"stamp\":[{},{}],\"tolerance\":{},\"components\":{},\"vertices\":[",
        g.stamp().id,
        g.stamp().version,
        g.surface().relative_tolerance(),
        g.component_count()
    )?;
    for (i, vertex) in g.surface().vertices().iter().enumerate() {
        if i > 0 {
            write!(w, ",")?;
        }
        array(&mut w, vertex)?;
    }
    write!(w, "],\"triangles\":[")?;
    for (i, triangle) in g.surface().triangles().iter().enumerate() {
        if i > 0 {
            write!(w, ",")?;
        }
        integers(&mut w, triangle)?;
    }
    write!(w, "],\"volumes\":")?;
    array(&mut w, g.fluid_volumes())?;
    write!(w, ",\"areas\":[")?;
    for (d, axis) in AXES.iter().enumerate() {
        if d > 0 {
            write!(w, ",")?;
        }
        array(&mut w, g.open_areas(*axis))?;
    }
    write!(w, "],\"labels\":")?;
    integers(&mut w, g.component_labels())?;
    writeln!(w, "}}")?;
    w.flush()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.len() == 3 && a[0] == "describe_geometry" {
        return describe_geometry(&a[1], Path::new(&a[2]));
    }
    if a.len() != 5 {
        return Err("usage: obstacle_gradient KIND EXPLICIT_STATE_CHECKPOINT REQUEST_FILE ROW_TEST_FILE NEW_EXTERNAL_OUTPUT_DIR; or describe_geometry KIND NEW_EXTERNAL_FILE".into());
    }
    let out = Path::new(&a[4]);
    let parent = out
        .parent()
        .ok_or("output needs a parent")?
        .canonicalize()?;
    if parent.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize()?) {
        return Err("generated evidence must stay outside Git".into());
    }
    let g = geometry(&a[0])?;
    let n = g.grid().counts()[0];
    let s = ObstacleFlowState::read_checkpoint(
        &g,
        &mut BufReader::with_capacity(8192, File::open(&a[1])?),
        MAX_OBSTACLE_STATE_BYTES,
    )?;
    let (sites, request_parse_payload) = requests(Path::new(&a[2]))?;
    let q_text = bounded_text(Path::new(&a[3]))?;
    let q = q_text
        .split_whitespace()
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()?;
    let parse_phase_payload_bytes = s.combined_payload_bytes()
        + request_parse_payload.max(
            sites.capacity() * std::mem::size_of::<ObstacleGradientSite>()
                + q_text.capacity()
                + q.capacity() * 8,
        );
    if parse_phase_payload_bytes > MAX_OBSTACLE_STATE_BYTES {
        return Err("parse-phase managed payload exceeds unchanged cap".into());
    }
    drop(q_text);
    let op = ObstacleVelocityGradient::new(&s, &sites, MAX_OBSTACLE_STATE_BYTES, |_, _| false)?;
    let planned_scratch = op
        .rows()
        .len()
        .checked_mul(8)
        .and_then(|b| b.checked_add(s.velocity().iter().map(|v| v.len() * 8).sum::<usize>()))
        .ok_or("scratch size overflow")?;
    let preflight = op.combined_payload_bytes()
        + sites.capacity() * std::mem::size_of::<ObstacleGradientSite>()
        + q.capacity() * 8
        + planned_scratch
        + 8192;
    if preflight > MAX_OBSTACLE_STATE_BYTES {
        return Err("explicit managed example payload exceeds unchanged cap".into());
    }
    let mut gradient = vec![0.; op.rows().len()];
    op.gather(&mut gradient, |_, _| false)?;
    let mut transpose: [Vec<f64>; 3] = std::array::from_fn(|d| vec![0.; s.velocity()[d].len()]);
    let [x, y, z] = &mut transpose;
    let work = op.diagnose(&q, [x, y, z], |_, _| false)?;
    let retained_action_payload = op.combined_payload_bytes()
        + sites.capacity() * std::mem::size_of::<ObstacleGradientSite>()
        + q.capacity() * 8
        + gradient.capacity() * 8
        + transpose.iter().map(|v| v.capacity() * 8).sum::<usize>()
        + 8192;
    if retained_action_payload > MAX_OBSTACLE_STATE_BYTES {
        return Err("actual managed example payload exceeds unchanged cap".into());
    }
    std::fs::create_dir(out)?;
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("gradient-record.json"))?;
    let mut w = BufWriter::with_capacity(8192, file);
    write!(
        w,
        "{{\"n\":{n},\"qualification\":\"Unqualified\",\"pressure_available\":false,\"physical_load_qualified\":false,\"physical_errors\":\"not_qualified\",\"state_combined_payload_bytes\":{},\"operator_owned_payload_bytes\":{},\"combined_payload_bytes\":{},\"retained_action_payload_bytes\":{retained_action_payload},\"parse_phase_payload_bytes\":{parse_phase_payload_bytes},\"cap_bytes\":{MAX_OBSTACLE_STATE_BYTES},\"input_origin\":\"{:?}\",\"input_errors\":\"{:?}\",\"initial_reference\":\"{:?}\",\"rows\":[",
        s.combined_payload_bytes(),
        op.owned_payload_bytes(),
        op.combined_payload_bytes(),
        s.frame().origin,
        s.frame().errors,
        s.physical_inputs().initial
    )?;
    for (i, row) in op.rows().iter().enumerate() {
        if i > 0 {
            write!(w, ",")?;
        }
        write!(
            w,
            "{{\"component\":{},\"derivative\":{},\"boundary\":\"{:?}\",\"weight\":{},\"active_term_count\":{},\"endpoints\":[",
            fixture::axis(row.component),
            fixture::axis(row.derivative),
            row.boundary,
            row.weight,
            row.active_term_count()
        )?;
        for (k, e) in row.endpoints.iter().enumerate() {
            if k > 0 {
                write!(w, ",")?;
            }
            write!(w, "{{\"position\":")?;
            array(&mut w, &e.position)?;
            write!(w, ",\"coefficient\":{},\"source\":", e.coefficient)?;
            match e.source {
                ObstacleGradientSource::VelocityFace { face } => {
                    write!(w, "\"VelocityFace\",\"face\":{face}")?
                }
                ObstacleGradientSource::StationarySolid => {
                    write!(w, "\"StationarySolid\",\"face\":null")?
                }
                ObstacleGradientSource::StationaryOuterNormal => {
                    write!(w, "\"StationaryOuterNormal\",\"face\":null")?
                }
            }
            write!(w, "}}")?;
        }
        write!(w, "]}}")?;
    }
    write!(w, "],\"gradient\":")?;
    array(&mut w, &gradient)?;
    write!(w, ",\"row_test\":")?;
    array(&mut w, &q)?;
    write!(w, ",\"velocity\":[")?;
    for d in 0..3 {
        if d > 0 {
            write!(w, ",")?;
        }
        array(&mut w, s.velocity()[d])?;
    }
    write!(w, "],\"transpose\":[")?;
    for (d, values) in transpose.iter().enumerate() {
        if d > 0 {
            write!(w, ",")?;
        }
        array(&mut w, values)?;
    }
    writeln!(
        w,
        "],\"work\":{{\"row_pairing\":{},\"face_pairing\":{},\"unenclosed_defect\":{}}}}}",
        work.row_pairing, work.face_pairing, work.unenclosed_defect
    )?;
    w.flush()?;
    println!(
        "{{\"record\":\"{}\",\"rows\":{},\"retained_action_payload_bytes\":{retained_action_payload},\"parse_phase_payload_bytes\":{parse_phase_payload_bytes},\"qualification\":\"Unqualified\"}}",
        out.join("gradient-record.json").display(),
        op.rows().len()
    );
    Ok(())
}
