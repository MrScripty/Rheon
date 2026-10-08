//! Machine-readable native fixture. Arguments (18): counts3 spacing3 origin3
//! lower-plane-indices3 upper-plane-indices3 density viscosity.
//! Hex scalar fields are exact binary64 bits; ACTION is base stiffness K u.
use rheon::{
    AlignedStrain, AlignedStrainBoundary, Axis, GridGeometry, StaticObstacleGeometry,
    SurfaceSettings, SurfaceStamp, TriangleSurface,
};
use std::{
    error::Error,
    io::{BufWriter, Write},
    path::Path,
};
fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 && args.len() != 18 {
        return Err("expected NEW external dump path, optionally preceded by counts3 spacing3 origin3 lo3 hi3 rho mu".into());
    }
    let output = args.pop().unwrap();
    let path = Path::new(&output);
    let parent = path.parent().unwrap_or(Path::new("."));
    let parent = parent.canonicalize()?;
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
    if parent.starts_with(repo) {
        return Err("dump destination must be outside the Git checkout".into());
    }
    let counts: [u64; 3] = if args.is_empty() {
        [4, 5, 3]
    } else {
        [args[0].parse()?, args[1].parse()?, args[2].parse()?]
    };
    let spacing: [f64; 3] = if args.is_empty() {
        [0.5, 0.75, 1.25]
    } else {
        [args[3].parse()?, args[4].parse()?, args[5].parse()?]
    };
    let origin: [f64; 3] = if args.is_empty() {
        [0.0; 3]
    } else {
        [args[6].parse()?, args[7].parse()?, args[8].parse()?]
    };
    let lo: [usize; 3] = if args.is_empty() {
        [1, 2, 1]
    } else {
        [args[9].parse()?, args[10].parse()?, args[11].parse()?]
    };
    let hi: [usize; 3] = if args.is_empty() {
        [3, 3, 2]
    } else {
        [args[12].parse()?, args[13].parse()?, args[14].parse()?]
    };
    let rho: f64 = if args.is_empty() {
        2.0
    } else {
        args[15].parse()?
    };
    let mu: f64 = if args.is_empty() {
        0.375
    } else {
        args[16].parse()?
    };
    let grid = GridGeometry::new(counts, spacing, origin)?;
    let lower = std::array::from_fn::<_, 3, _>(|d| origin[d] + lo[d] as f64 * spacing[d]);
    let upper = std::array::from_fn::<_, 3, _>(|d| origin[d] + hi[d] as f64 * spacing[d]);
    let vertices = (0..8)
        .map(|c| {
            std::array::from_fn(|d| {
                if c & (1 << d) == 0 {
                    lower[d]
                } else {
                    upper[d]
                }
            })
        })
        .collect();
    let triangles = vec![
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
    ];
    let surface = TriangleSurface::new(
        SurfaceStamp { id: 73, version: 1 },
        vertices,
        triangles,
        SurfaceSettings::default(),
    )?;
    let geometry = StaticObstacleGeometry::new(grid, surface, 64 * 1024 * 1024, |_, _| false)?;
    let op = AlignedStrain::new(&geometry, rho, mu, 64 * 1024 * 1024, |_, _| false)?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let mut dump = BufWriter::new(file);
    write!(dump, "META")?;
    for x in counts {
        write!(dump, "\t{x}")?;
    }
    for x in spacing {
        write!(dump, "\t{:016x}", x.to_bits())?;
    }
    for x in origin {
        write!(dump, "\t{:016x}", x.to_bits())?;
    }
    for x in lo {
        write!(dump, "\t{x}")?;
    }
    for x in hi {
        write!(dump, "\t{x}")?;
    }
    writeln!(dump, "\t{:016x}\t{:016x}", rho.to_bits(), mu.to_bits())?;
    let axis = |a| match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    };
    for (i, f) in op.active_faces().iter().enumerate() {
        write!(dump, "ACTIVE\t{i}\t{}\t{}", axis(f.axis), f.face)?;
        for x in f.coordinates {
            write!(dump, "\t{x}")?;
        }
        for x in f.position {
            write!(dump, "\t{:016x}", x.to_bits())?;
        }
        writeln!(
            dump,
            "\t{:016x}\t{:016x}\t{:016x}",
            f.area.to_bits(),
            f.distance.to_bits(),
            f.mass.to_bits()
        )?;
    }
    for (i, r) in op.rows().iter().enumerate() {
        write!(dump, "ROW\t{i}\t{}\t{}", axis(r.axes[0]), axis(r.axes[1]))?;
        for x in r.coordinates {
            write!(dump, "\t{x}")?;
        }
        let boundary = match r.boundary {
            AlignedStrainBoundary::Interior => 0,
            AlignedStrainBoundary::ObstacleFlat => 1,
            AlignedStrainBoundary::ObstacleCorner => 2,
            AlignedStrainBoundary::OuterFreeSlip => 3,
            AlignedStrainBoundary::Normal => 4,
        };
        write!(
            dump,
            "\t{}\t{:016x}\t{boundary}\t{}",
            r.quadrant,
            r.weight.to_bits(),
            r.terms().len()
        )?;
        for t in r.terms() {
            write!(dump, "\t{}\t{:016x}", t.active, t.coefficient.to_bits())?;
        }
        writeln!(dump)?;
    }
    let n = op.active_faces().len();
    let mut basis = vec![0.0; n];
    let mut action = vec![0.0; n];
    for j in 0..n {
        basis[j] = 1.0;
        op.apply(&basis, &mut action, |_, _| false)?;
        for (i, &v) in action.iter().enumerate() {
            if v != 0.0 {
                writeln!(dump, "MATRIX\t{i}\t{j}\t{:016x}", v.to_bits())?;
            }
        }
        basis[j] = 0.0;
    }
    let field: Vec<f64> = (0..n)
        .map(|i| ((i * 17 % 29) as f64 - 14.0) / 16.0)
        .collect();
    op.apply(&field, &mut action, |_, _| false)?;
    for (i, (&v, &a)) in field.iter().zip(&action).enumerate() {
        writeln!(dump, "FIELD\t{i}\t{:016x}", v.to_bits())?;
        writeln!(dump, "ACTION\t{i}\t{:016x}", a.to_bits())?;
    }
    let ledger = op.diagnose(&field, &mut action, |_, _| false)?;
    writeln!(
        dump,
        "LEDGER\t{:016x}\t{:016x}\t{:016x}\t{:016x}",
        ledger.dissipation.to_bits(),
        ledger.force_work.to_bits(),
        ledger.identity_error.to_bits(),
        ledger.unenclosed_b_estimate.to_bits()
    )?;
    dump.flush()?;
    Ok(())
}
