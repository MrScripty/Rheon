//! Tiny aligned coupled M/B/S/W reference v1; no production/cut/free-surface claim.
use rheon::{
    AlignedStrain, Axis, GridGeometry, StaticObstacleGeometry, SurfaceSettings, SurfaceStamp,
    TriangleSurface,
};
use std::{error::Error, io::Write, path::Path};
type Matrix = Vec<Vec<f64>>;
fn axis(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
#[derive(Clone, Copy)]
struct Fixture {
    name: &'static str,
    n: [u64; 3],
    h: [f64; 3],
    origin: [f64; 3],
    lo: [usize; 3],
    hi: [usize; 3],
}
fn fixtures() -> [Fixture; 2] {
    [
        Fixture {
            name: "unit",
            n: [3; 3],
            h: [1.0; 3],
            origin: [0.0; 3],
            lo: [1; 3],
            hi: [2; 3],
        },
        Fixture {
            name: "anisotropic",
            n: [3, 4, 3],
            h: [0.5, 0.75, 1.25],
            origin: [-0.5, 0.25, -1.0],
            lo: [1; 3],
            hi: [2, 3, 2],
        },
    ]
}
fn owner(f: Fixture) -> Result<StaticObstacleGeometry, Box<dyn Error>> {
    let lo: [f64; 3] = std::array::from_fn(|d| f.origin[d] + f.lo[d] as f64 * f.h[d]);
    let hi: [f64; 3] = std::array::from_fn(|d| f.origin[d] + f.hi[d] as f64 * f.h[d]);
    let mesh = TriangleSurface::new(
        SurfaceStamp {
            id: 132,
            version: 1,
        },
        (0..8)
            .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { lo[d] } else { hi[d] }))
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
        GridGeometry::new(f.n, f.h, f.origin)?,
        mesh,
        16_000_000,
        |_, _| false,
    )?)
}
fn zero(n: usize, m: usize) -> Matrix {
    vec![vec![0.0; m]; n]
}
fn action(a: &Matrix, x: &[f64]) -> Vec<f64> {
    a.iter()
        .map(|r| r.iter().zip(x).map(|(v, x)| v * x).sum())
        .collect()
}
/// Direct partial-pivoted tiny reference, with no small-pivot floor.
fn solve(a: &Matrix, b: &[f64]) -> Result<Vec<f64>, String> {
    let n = b.len();
    if n == 0
        || n > 256
        || a.len() != n
        || a.iter().any(|r| r.len() != n)
        || a.iter().flatten().chain(b).any(|x| !x.is_finite())
    {
        return Err("finite square tiny system required".into());
    }
    let mut rows = a.clone();
    for (r, v) in rows.iter_mut().zip(b) {
        r.push(*v);
    }
    for k in 0..n {
        let p = (k..n)
            .max_by(|&i, &j| rows[i][k].abs().total_cmp(&rows[j][k].abs()))
            .unwrap();
        if rows[p][k] == 0.0 {
            return Err("singular coupled reference; no pivot floor".into());
        }
        rows.swap(k, p);
        let pivot = rows[k][k];
        for x in &mut rows[k][k..] {
            *x /= pivot;
        }
        let basis = rows[k].clone();
        for (i, row) in rows.iter_mut().enumerate() {
            if i != k {
                let factor = row[k];
                for (x, p) in row[k..].iter_mut().zip(&basis[k..]) {
                    *x -= factor * p;
                }
            }
        }
        if rows.iter().flatten().any(|x| !x.is_finite()) {
            return Err("nonfinite direct elimination".into());
        }
    }
    Ok(rows.iter().map(|r| r[n]).collect())
}
struct System {
    cells: Vec<usize>,
    gauges: Vec<usize>,
    b: Matrix,
    k: Matrix,
    a: Matrix,
    coupled: Matrix,
    old: Vec<f64>,
    mass: Vec<f64>,
    free_pressure: Vec<usize>,
}
fn assemble(s: &AlignedStrain<'_>, dt: f64) -> Result<System, String> {
    if !dt.is_finite() || dt <= 0.0 {
        return Err("positive finite dt required".into());
    }
    let g = s.geometry();
    let cells: Vec<_> = g
        .fluid_volumes()
        .iter()
        .enumerate()
        .filter_map(|(i, v)| (*v > 0.0).then_some(i))
        .collect();
    let mut map = vec![usize::MAX; g.fluid_volumes().len()];
    for (i, &c) in cells.iter().enumerate() {
        map[c] = i;
    }
    let mut gauges = vec![usize::MAX; g.component_count()];
    for (i, &c) in cells.iter().enumerate() {
        let label = g.component_labels()[c];
        if label >= gauges.len() {
            return Err("invalid component label".into());
        }
        gauges[label] = gauges[label].min(i);
    }
    if gauges.is_empty() || gauges.contains(&usize::MAX) {
        return Err("missing pressure component".into());
    }
    let nv = s.active_faces().len();
    if nv + cells.len() - gauges.len() > 256 {
        return Err("tiny coupled reference dimension limit".into());
    }
    let mut b = zero(cells.len(), nv);
    let mut mass = Vec::with_capacity(nv);
    let mut old = Vec::with_capacity(nv);
    for (j, f) in s.active_faces().iter().enumerate() {
        if !f.mass.is_finite()
            || f.mass <= 0.0
            || map[f.negative_cell] == usize::MAX
            || map[f.positive_cell] == usize::MAX
            || g.component_labels()[f.negative_cell] != g.component_labels()[f.positive_cell]
        {
            return Err("invalid active mass/incidence".into());
        }
        b[map[f.negative_cell]][j] = f.area;
        b[map[f.positive_cell]][j] = -f.area;
        mass.push(f.mass);
        let [x, y, z] = f.position;
        let d = axis(f.axis) as f64;
        old.push(
            (0.7 * x - 0.4 * y + 0.3 * z + 0.2 * d).sin()
                + 0.25 * (x + 0.6 * y - 0.2 * z + 0.4 * d).cos(),
        );
    }
    let mut k = zero(nv, nv);
    for row in s.rows() {
        let w = s.viscosity() * row.weight;
        if !w.is_finite() || w < 0.0 {
            return Err("invalid strain weight".into());
        }
        for p in row.terms() {
            for q in row.terms() {
                k[p.active][q.active] += w * p.coefficient * q.coefficient;
            }
        }
    }
    let mut a = k.clone();
    for (i, row) in a.iter_mut().enumerate() {
        row[i] += mass[i] / dt;
    }
    let free_pressure: Vec<_> = (0..cells.len()).filter(|i| !gauges.contains(i)).collect();
    let mut coupled = zero(nv + free_pressure.len(), nv + free_pressure.len());
    for (i, row) in a.iter().enumerate() {
        coupled[i][..nv].copy_from_slice(row);
    }
    for (q, &c) in free_pressure.iter().enumerate() {
        for (j, &value) in b[c].iter().enumerate() {
            coupled[j][nv + q] = -value;
            coupled[nv + q][j] = -value;
        }
    }
    if coupled.iter().flatten().any(|x| !x.is_finite()) {
        return Err("nonfinite coupled assembly".into());
    }
    Ok(System {
        cells,
        gauges,
        b,
        k,
        a,
        coupled,
        old,
        mass,
        free_pressure,
    })
}
struct ResultRow {
    momentum: f64,
    continuity: f64,
    work: f64,
    corrected_work: f64,
    residual_work_bound: f64,
    old_energy: f64,
    energy: f64,
    dissipation: f64,
    pressure_work: f64,
}
fn run(root: &Path, f: Fixture, rho: f64, mu: f64, dt: f64) -> Result<ResultRow, Box<dyn Error>> {
    std::fs::create_dir(root)?;
    let g = owner(f)?;
    let s = AlignedStrain::new(&g, rho, mu, 16_000_000, |_, _| false)?;
    let system = assemble(&s, dt)?;
    let nv = system.mass.len();
    let mut rhs: Vec<_> = system
        .mass
        .iter()
        .zip(&system.old)
        .map(|(m, u)| m * u / dt)
        .collect();
    rhs.resize(system.coupled.len(), 0.0);
    let solved = solve(&system.coupled, &rhs)?;
    let u = &solved[..nv];
    let mut p = vec![0.0; system.cells.len()];
    for (q, &c) in system.free_pressure.iter().enumerate() {
        p[c] = solved[nv + q];
    }
    let ku = action(&system.k, u);
    let flux = action(&system.b, u);
    if flux.iter().any(|v| !v.is_finite()) {
        return Err("nonfinite full cell flux".into());
    }
    let mut r = vec![0.0; nv];
    let mut momentum: f64 = 0.0;
    let lref = 1.0;
    let uref = 1.0;
    let pref = rho * uref * lref / dt + mu * uref / lref;
    for (i, ri) in r.iter_mut().enumerate() {
        let pressure: f64 = system.b.iter().zip(&p).map(|(b, p)| b[i] * p).sum();
        *ri = system.mass[i] * (u[i] - system.old[i]) / dt + ku[i] - pressure;
        let scale = system.mass[i] * uref / dt
            + system.k[i].iter().map(|x| x.abs() * uref).sum::<f64>()
            + system.b.iter().map(|b| b[i].abs() * pref).sum::<f64>();
        if !ri.is_finite() || !scale.is_finite() || scale <= 0.0 {
            return Err("nonfinite momentum residual or invalid scale".into());
        }
        momentum = momentum.max(ri.abs() / scale);
    }
    if system.b.iter().any(|row| {
        let scale = row.iter().map(|x| x.abs()).sum::<f64>();
        !scale.is_finite() || scale <= 0.0
    }) {
        return Err("unsupported isolated pressure row or invalid flux scale".into());
    }
    let continuity = flux
        .iter()
        .zip(&system.b)
        .map(|(v, b)| v.abs() / (b.iter().map(|x| x.abs()).sum::<f64>() * uref))
        .fold(0.0_f64, f64::max);
    let e = |v: &[f64]| {
        system
            .mass
            .iter()
            .zip(v)
            .map(|(m, x)| 0.5 * m * x * x)
            .sum::<f64>()
    };
    let energy = e(u);
    let old_energy = e(&system.old);
    let delta: Vec<_> = u.iter().zip(&system.old).map(|(x, y)| x - y).collect();
    let dissipation: f64 = u.iter().zip(&ku).map(|(u, k)| u * k).sum();
    let pressure_work: f64 = p.iter().zip(&flux).map(|(p, b)| p * b).sum();
    let residual_work: f64 = u.iter().zip(&r).map(|(u, r)| u * r).sum();
    let echar = system.mass.iter().sum::<f64>() * uref * uref / 2.0;
    let defect = energy - old_energy + e(&delta) + dt * dissipation;
    let work = defect.abs() / echar;
    let corrected_work = (defect - dt * (pressure_work + residual_work)).abs() / echar;
    let residual_work_bound = dt
        * (p.iter().zip(&flux).map(|(p, b)| (p * b).abs()).sum::<f64>()
            + u.iter().zip(&r).map(|(u, r)| (u * r).abs()).sum::<f64>())
        / echar;
    let mut meta = std::io::BufWriter::new(std::fs::File::create(root.join("meta.csv"))?);
    writeln!(
        meta,
        "key,value\nfixture,{}\nrho,{rho:.17e}\nmu,{mu:.17e}\ndt,{dt:.17e}\nnv,{nv}\nnc,{}\nns,{}\ncomponents,{}",
        f.name,
        system.cells.len(),
        s.rows().len(),
        g.component_count()
    )?;
    for d in 0..3 {
        writeln!(
            meta,
            "n{d},{}\nh{d},{:.17e}\norigin{d},{:.17e}\nlo{d},{}\nhi{d},{}",
            f.n[d], f.h[d], f.origin[d], f.lo[d], f.hi[d]
        )?;
    }
    meta.flush()?;
    let mut faces = std::io::BufWriter::new(std::fs::File::create(root.join("faces.csv"))?);
    writeln!(
        faces,
        "active,axis,face,i,j,k,x,y,z,negative,positive,area,distance,mass,old,u"
    )?;
    for (i, v) in s.active_faces().iter().enumerate() {
        writeln!(
            faces,
            "{i},{},{},{},{},{},{:.17e},{:.17e},{:.17e},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            axis(v.axis),
            v.face,
            v.coordinates[0],
            v.coordinates[1],
            v.coordinates[2],
            v.position[0],
            v.position[1],
            v.position[2],
            v.negative_cell,
            v.positive_cell,
            v.area,
            v.distance,
            v.mass,
            system.old[i],
            u[i]
        )?;
    }
    faces.flush()?;
    let mut cells = std::io::BufWriter::new(std::fs::File::create(root.join("cells.csv"))?);
    writeln!(cells, "cell,volume,label,pressure_index,gauge,p")?;
    for (c, &v) in g.fluid_volumes().iter().enumerate() {
        let index = system.cells.iter().position(|&i| i == c);
        let label = if v > 0.0 {
            g.component_labels()[c].to_string()
        } else {
            "solid".into()
        };
        writeln!(
            cells,
            "{c},{v:.17e},{label},{},{},{:.17e}",
            index.map_or_else(String::new, |i| i.to_string()),
            index.is_some_and(|i| system.gauges.contains(&i)),
            index.map_or(0.0, |i| p[i])
        )?;
    }
    cells.flush()?;
    let mut rows = std::io::BufWriter::new(std::fs::File::create(root.join("rows.csv"))?);
    let mut terms = std::io::BufWriter::new(std::fs::File::create(root.join("terms.csv"))?);
    writeln!(rows, "row,a,b,i,j,k,quadrant,boundary,weight")?;
    writeln!(terms, "row,active,coefficient")?;
    for (i, row) in s.rows().iter().enumerate() {
        writeln!(
            rows,
            "{i},{},{},{},{},{},{},{:?},{:.17e}",
            axis(row.axes[0]),
            axis(row.axes[1]),
            row.coordinates[0],
            row.coordinates[1],
            row.coordinates[2],
            row.quadrant,
            row.boundary,
            row.weight
        )?;
        for t in row.terms() {
            writeln!(terms, "{i},{},{:.17e}", t.active, t.coefficient)?;
        }
    }
    rows.flush()?;
    terms.flush()?;
    let mut matrices = std::io::BufWriter::new(std::fs::File::create(root.join("matrices.csv"))?);
    writeln!(matrices, "block,row,column,value")?;
    for (name, m) in [
        ("B", &system.b),
        ("K", &system.k),
        ("A", &system.a),
        ("coupled", &system.coupled),
    ] {
        for (i, row) in m.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                writeln!(matrices, "{name},{i},{j},{v:.17e}")?;
            }
        }
    }
    matrices.flush()?;
    let values = [
        momentum,
        continuity,
        work,
        corrected_work,
        residual_work_bound,
        energy,
        old_energy,
        dissipation,
        pressure_work,
    ];
    if values.iter().any(|v| !v.is_finite())
        || momentum > 1e-10
        || continuity > 1e-10
        || work > 1e-10
        || corrected_work > 1e-10
        || residual_work_bound > 1e-10
        || dissipation < 0.0
        || energy > old_energy + 1e-10 * echar
    {
        return Err("fixed coupled residual/work gate failed".into());
    }
    Ok(ResultRow {
        momentum,
        continuity,
        work,
        corrected_work,
        residual_work_bound,
        old_energy,
        energy,
        dissipation,
        pressure_work,
    })
}
fn main() -> Result<(), Box<dyn Error>> {
    let arg = std::env::args_os()
        .nth(1)
        .ok_or("fresh output root required")?;
    let root = Path::new(&arg);
    std::fs::create_dir(root)?;
    let mut summary = std::io::BufWriter::new(std::fs::File::create(root.join("summary.csv"))?);
    writeln!(
        summary,
        "fixture,rho,mu,dt,momentum,continuity,work,corrected_work,residual_work_bound,old_energy,energy,dissipation,pressure_work"
    )?;
    for f in fixtures() {
        for (rho, mu) in [(1.0, 0.1), (998.206, 0.001002)] {
            for dt in [0.125, 1.0] {
                let tag = format!("{}-rho{rho}-dt{dt}", f.name);
                let r = run(&root.join(&tag), f, rho, mu, dt).map_err(|e| format!("{tag}: {e}"))?;
                writeln!(
                    summary,
                    "{},{rho:.17e},{mu:.17e},{dt:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                    f.name,
                    r.momentum,
                    r.continuity,
                    r.work,
                    r.corrected_work,
                    r.residual_work_bound,
                    r.old_energy,
                    r.energy,
                    r.dissipation,
                    r.pressure_work
                )?;
                eprintln!(
                    "coupled {tag}: momentum={:.3e}, continuity={:.3e}, work={:.3e}",
                    r.momentum, r.continuity, r.work
                );
            }
        }
    }
    summary.flush()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pivoting_and_singular_refusal() {
        assert_eq!(
            solve(&vec![vec![0.0, 1.0], vec![1.0, 0.0]], &[2.0, 3.0]).unwrap(),
            vec![3.0, 2.0]
        );
        assert!(solve(&vec![vec![1.0, 1.0], vec![1.0, 1.0]], &[1.0, 1.0]).is_err());
        assert!(solve(&vec![vec![f64::NAN]], &[0.0]).is_err());
    }
    #[test]
    fn assembly_is_coupled_and_all_component_fluxes_cancel() {
        let g = owner(fixtures()[0]).unwrap();
        let s = AlignedStrain::new(&g, 1.0, 0.1, 16_000_000, |_, _| false).unwrap();
        let a = assemble(&s, 0.125).unwrap();
        assert!(assemble(&s, 0.0).is_err());
        assert_eq!(a.gauges.len(), g.component_count());
        for j in 0..a.mass.len() {
            assert_eq!(a.b.iter().map(|r| r[j]).sum::<f64>(), 0.0);
        }
        assert_eq!(
            a.coupled.len(),
            a.mass.len() + a.cells.len() - a.gauges.len()
        );
        let nv = a.mass.len();
        assert!((0..nv).any(|j| a.coupled[j][nv] != 0.0));
    }
}
