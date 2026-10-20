use super::geometry::{Geometry, Moments, Point, Region, Segment};
use std::{io::Write, path::Path};
pub fn cubic(p: Point) -> Point {
    [p[0].powi(3), p[1].powi(3)]
}
pub fn solenoidal(p: Point) -> Point {
    [
        p[0] * p[0] + 2.0 * p[0] * p[1],
        -2.0 * p[0] * p[1] - p[1] * p[1],
    ]
}
pub fn traction(s: Segment, mu: f64, pressure: bool) -> Point {
    let [x, y] = s.middle();
    let p = if pressure {
        1.0 + mu * x - 3.0 * mu * y
    } else {
        0.0
    };
    let sigma = [
        [4.0 * mu * (x + y) - p, 2.0 * mu * (x - y)],
        [2.0 * mu * (x - y), -4.0 * mu * (x + y) - p],
    ];
    let n = s.normal_measure();
    std::array::from_fn(|d| sigma[d][0] * n[0] + sigma[d][1] * n[1])
}
fn moment_row(out: &mut impl Write, kind: &str, index: usize, m: Moments) -> std::io::Result<()> {
    writeln!(
        out,
        "{kind},{index},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
        m.area, m.first[0], m.first[1], m.second[0], m.second[1], m.second[2]
    )
}
fn domain_row(out: &mut impl Write, kind: &str, index: usize, r: &Region) -> std::io::Result<()> {
    writeln!(
        out,
        "{kind},{index},{},{},{},{},{}",
        r.lo[0].to_bits(),
        r.lo[1].to_bits(),
        r.hi[0].to_bits(),
        r.hi[1].to_bits(),
        r.moments.area.to_bits()
    )
}
#[derive(Default)]
pub struct Metrics {
    pub area: f64,
    pub first: Point,
    pub radial_second: f64,
    pub second: [f64; 3],
    pub closure: f64,
    pub linear_closure: f64,
    pub cubic_defect: f64,
    pub solenoidal_defect: f64,
    pub traction_defect: f64,
    pub face_mismatch: f64,
    pub flux: f64,
    pub force: Point,
    pub wrong_pressure_force: Point,
    pub dual_relative_defect: f64,
    pub min_positive_volume: f64,
}
pub fn run(root: &Path, g: &Geometry, mu: f64) -> Result<Metrics, Box<dyn std::error::Error>> {
    std::fs::create_dir(root)?;
    let n = g.grid.counts()[0];
    let h = g.grid.spacing()[0];
    let origin = g.grid.origin();
    let mut source = std::io::BufWriter::new(std::fs::File::create(root.join("source.csv"))?);
    writeln!(source, "vertex,x,y")?;
    for (i, p) in g.source.vertices().iter().enumerate() {
        writeln!(source, "{i},{:.17e},{:.17e}", p[0], p[1])?;
    }
    let mut moments = std::io::BufWriter::new(std::fs::File::create(root.join("moments.csv"))?);
    writeln!(moments, "kind,index,volume,mx,my,mxx,mxy,myy")?;
    let mut domains = std::io::BufWriter::new(std::fs::File::create(root.join("domains.csv"))?);
    writeln!(
        domains,
        "kind,index,lo_x_bits,lo_y_bits,hi_x_bits,hi_y_bits,volume_bits"
    )?;
    let mut faces = std::io::BufWriter::new(std::fs::File::create(root.join("faces.csv"))?);
    writeln!(
        faces,
        "axis,index,ax,ay,bx,by,area,mx,my,cubic_flux,solenoidal_flux"
    )?;
    let mut topology = std::io::BufWriter::new(std::fs::File::create(root.join("topology.csv"))?);
    writeln!(
        topology,
        "axis,index,position_bits,lo_bits,hi_bits,status,lower_num,lower_den,upper_num,upper_den,represented_length,dual_volume"
    )?;
    for axis in 0..2 {
        for (index, f) in g.faces[axis].iter().enumerate() {
            let (normal, tangent) = if axis == 0 {
                (index % (n + 1), index / (n + 1))
            } else {
                (index / n, index % n)
            };
            let position = origin[axis] + normal as f64 * h;
            let lo = origin[1 - axis] + tangent as f64 * h;
            let hi = origin[1 - axis] + (tangent + 1) as f64 * h;
            let bounds = f
                .exact
                .bounds()
                .map(|(l, u)| {
                    [
                        l.numer().to_string(),
                        l.denom().to_string(),
                        u.numer().to_string(),
                        u.denom().to_string(),
                    ]
                })
                .unwrap_or_default();
            writeln!(
                topology,
                "{axis},{index},{},{},{},{},{},{},{},{},{:.17e},{:.17e}",
                position.to_bits(),
                lo.to_bits(),
                hi.to_bits(),
                f.exact.status(),
                bounds[0],
                bounds[1],
                bounds[2],
                bounds[3],
                f.segment.map_or(0.0, |s| s.normal_measure()[axis]),
                f.dual.moments.area
            )?;
            moment_row(
                &mut moments,
                if axis == 0 { "dual-x" } else { "dual-y" },
                index,
                f.dual.moments,
            )?;
            domain_row(
                &mut domains,
                if axis == 0 { "dual-x" } else { "dual-y" },
                index,
                &f.dual,
            )?;
            if let Some(s) = f.segment {
                let length = s.normal_measure()[axis];
                let mid = s.middle();
                writeln!(
                    faces,
                    "{axis},{index},{:.17e},{:.17e},{:.17e},{:.17e},{length:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                    s.a[0],
                    s.a[1],
                    s.b[0],
                    s.b[1],
                    length * mid[0],
                    length * mid[1],
                    s.integrate(cubic),
                    s.integrate(solenoidal)
                )?;
            }
        }
    }
    for (index, m) in g.shear.iter().enumerate() {
        moment_row(&mut moments, "shear", index, m.moments)?;
        domain_row(&mut domains, "shear", index, m)?;
    }
    let mut boundary = std::io::BufWriter::new(std::fs::File::create(root.join("boundary.csv"))?);
    writeln!(
        boundary,
        "cell,edge,ax,ay,bx,by,nx_ds,ny_ds,mx_nx,my_nx,mx_ny,my_ny,cubic_flux,solenoidal_flux,pressure,tx,ty,wrong_tx,wrong_ty"
    )?;
    let mut ledger = std::io::BufWriter::new(std::fs::File::create(root.join("cell-ledger.csv"))?);
    writeln!(
        ledger,
        "cell,volume,normal_x,normal_y,x_nx,y_nx,x_ny,y_ny,cubic_flux,cubic_expected,solenoidal_flux,traction_x,traction_y,traction_expected,wrong_tx,wrong_ty,face_mismatch"
    )?;
    let mut m = Metrics {
        min_positive_volume: f64::INFINITY,
        ..Metrics::default()
    };
    for (index, cell) in g.cells.iter().enumerate() {
        moment_row(&mut moments, "cell", index, cell.region.moments)?;
        domain_row(&mut domains, "cell", index, &cell.region)?;
        let i = index % n;
        let j = index / n;
        let mut normal = [0.0; 2];
        let mut first = [[0.0; 2]; 2];
        let mut cubic_flux = 0.0;
        let mut div_free_flux = 0.0;
        let mut force = [0.0; 2];
        let mut wrong = [0.0; 2];
        let mut face_error: f64 = 0.0;
        let mut add = |s: Segment| {
            let ns = s.normal_measure();
            let mid = s.middle();
            let t = traction(s, mu, true);
            let z = traction(s, mu, false);
            for d in 0..2 {
                normal[d] += ns[d];
                force[d] += t[d];
                wrong[d] += z[d];
                for e in 0..2 {
                    first[d][e] += ns[d] * mid[e];
                }
            }
            cubic_flux += s.integrate(cubic);
            div_free_flux += s.integrate(solenoidal);
        };
        for (axis, face, sign) in g.cell_faces(i, j) {
            let expected = g.faces[axis][face]
                .segment
                .map_or(0.0, |s| s.normal_measure()[axis]);
            let plane = origin[axis]
                + (if axis == 0 { i } else { j }) as f64 * h
                + if sign > 0.0 { h } else { 0.0 };
            let actual: f64 = super::geometry::edges(&cell.clipped)
                .filter(|(a, b)| a[axis] == plane && b[axis] == plane)
                .map(|(a, b)| (b[1 - axis] - a[1 - axis]).abs())
                .sum();
            face_error = face_error.max((actual - expected).abs());
            if let Some(mut s) = g.faces[axis][face].segment {
                if sign < 0.0 {
                    std::mem::swap(&mut s.a, &mut s.b);
                }
                add(s);
            }
        }
        for piece in &cell.physical {
            let s = piece.segment;
            add(s);
            let ns = s.normal_measure();
            let mid = s.middle();
            let t = traction(s, mu, true);
            let z = traction(s, mu, false);
            writeln!(
                boundary,
                "{index},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                piece.edge,
                s.a[0],
                s.a[1],
                s.b[0],
                s.b[1],
                ns[0],
                ns[1],
                ns[0] * mid[0],
                ns[0] * mid[1],
                ns[1] * mid[0],
                ns[1] * mid[1],
                s.integrate(cubic),
                s.integrate(solenoidal),
                1.0 + mu * mid[0] - 3.0 * mu * mid[1],
                t[0],
                t[1],
                z[0],
                z[1]
            )?;
            m.flux += s.integrate(cubic);
            for d in 0..2 {
                m.force[d] += t[d];
                m.wrong_pressure_force[d] += z[d];
            }
        }
        let area = cell.region.moments.area;
        let cubic_expected = 3.0 * (cell.region.moments.second[0] + cell.region.moments.second[2]);
        writeln!(
            ledger,
            "{index},{area:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{cubic_flux:.17e},{cubic_expected:.17e},{div_free_flux:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{face_error:.17e}",
            normal[0],
            normal[1],
            first[0][0],
            first[0][1],
            first[1][0],
            first[1][1],
            force[0],
            force[1],
            mu * area,
            wrong[0],
            wrong[1]
        )?;
        m.area += area;
        for d in 0..2 {
            m.first[d] += cell.region.moments.first[d];
        }
        m.radial_second += cell.region.moments.second[0] + cell.region.moments.second[2];
        for (d, value) in cell.region.moments.second.into_iter().enumerate() {
            m.second[d] += value;
        }
        if area > 0.0 {
            m.min_positive_volume = m.min_positive_volume.min(area);
        }
        for d in 0..2 {
            m.closure = m.closure.max(normal[d].abs() / h);
            m.traction_defect = m
                .traction_defect
                .max((force[d] - mu * area).abs() / (h * h));
            for (e, value) in first[d].iter().enumerate() {
                m.linear_closure = m
                    .linear_closure
                    .max((*value - if d == e { area } else { 0.0 }).abs() / (h * h));
            }
        }
        m.face_mismatch = m.face_mismatch.max(face_error / h);
        m.cubic_defect = m
            .cubic_defect
            .max((cubic_flux - cubic_expected).abs() / (h * h));
        m.solenoidal_defect = m.solenoidal_defect.max(div_free_flux.abs() / (h * h));
    }
    let dual_sums = [
        g.faces[0].iter().map(|f| f.dual.moments.area).sum::<f64>(),
        g.faces[1].iter().map(|f| f.dual.moments.area).sum::<f64>(),
        g.shear.iter().map(|r| r.moments.area).sum::<f64>(),
    ];
    for sum in dual_sums {
        m.dual_relative_defect = m.dual_relative_defect.max((sum - m.area).abs() / m.area);
    }
    source.flush()?;
    moments.flush()?;
    domains.flush()?;
    faces.flush()?;
    topology.flush()?;
    boundary.flush()?;
    ledger.flush()?;
    Ok(m)
}
