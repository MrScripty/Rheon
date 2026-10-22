use super::{
    assembly::Operators,
    basis::{affine, work_vectors},
    clip,
    observe::Report,
    trace::Traces,
};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};
fn stream(
    root: &Path,
    name: &str,
    header: &str,
    body: impl FnOnce(&mut BufWriter<File>) -> io::Result<()>,
) -> io::Result<()> {
    let mut w = BufWriter::new(File::create(root.join(name))?);
    writeln!(w, "{header}")?;
    body(&mut w)?;
    w.flush()
}
pub fn inputs(
    root: &Path,
    vertices: &[[f64; 2]],
    n: usize,
    origin: [f64; 2],
    extent: f64,
) -> io::Result<()> {
    stream(root, "source.csv", "vertex,x,y", |w| {
        for (i, p) in vertices.iter().enumerate() {
            writeln!(w, "{i},{:.17e},{:.17e}", p[0], p[1])?;
        }
        Ok(())
    })?;
    stream(root, "input-meta.csv", "key,value", |w| {
        writeln!(
            w,
            "n,{n}\norigin_x,{:.17e}\norigin_y,{:.17e}\nextent,{extent:.17e}\nrho,1\nmu,0.1\ndepth,1\nmodel,CV1-operator-only",
            origin[0], origin[1]
        )
    })
}
pub fn all(root: &Path, op: &Operators, tr: &Traces, r: &Report) -> io::Result<()> {
    stream(root, "meta.csv", "key,value", |w| {
        writeln!(
            w,
            "n,{}\nh,{:.17e}\norigin_x,{:.17e}\norigin_y,{:.17e}\nactive,{}\npressure_cells,{}\npatches,{}\nquadrature,{}\nsegments,{}\ntrace_points,{}\nrho,1\nmu,0.1\ndepth,1\nvelocity_basis,staggered-continuous-Q1\npressure_basis,primal-cut-P0\noperators_only,true\npressure_gauge,none",
            op.layout.n,
            op.layout.h,
            op.layout.origin[0],
            op.layout.origin[1],
            op.active.len(),
            op.cells.len(),
            op.patches.len(),
            op.quadrature.len(),
            tr.records.len(),
            tr.points.len()
        )
    })?;
    stream(
        root,
        "nodes.csv",
        "global,axis,face,i,j,x,y,active,old_dual,mass_diagonal,a,b,affine",
        |w| {
            for p in &op.layout.nodes {
                let i = op.map[p.global];
                let (active, dual, mass) = if i == usize::MAX {
                    (String::new(), String::new(), String::new())
                } else {
                    (
                        i.to_string(),
                        format!("{:.17e}", op.old_dual[i]),
                        format!("{:.17e}", op.mass[i][i]),
                    )
                };
                let [a, b] = work_vectors(p);
                writeln!(
                    w,
                    "{},{},{},{},{},{:.17e},{:.17e},{active},{dual},{mass},{a:.17e},{b:.17e},{:.17e}",
                    p.global,
                    p.axis,
                    p.face,
                    p.index[0],
                    p.index[1],
                    p.point[0],
                    p.point[1],
                    affine(p)
                )?;
            }
            Ok(())
        },
    )?;
    stream(
        root,
        "patches.csv",
        "patch,i,j,cell,lo_x_bits,lo_y_bits,hi_x_bits,hi_y_bits,area_num,area_den,vertices,triangles",
        |w| {
            for (id, p) in op.patches.iter().enumerate() {
                let a = clip::area(&p.polygon);
                writeln!(
                    w,
                    "{id},{},{},{},{},{},{},{},{},{},{},{}",
                    p.i,
                    p.j,
                    p.cell,
                    p.lo[0].to_bits(),
                    p.lo[1].to_bits(),
                    p.hi[0].to_bits(),
                    p.hi[1].to_bits(),
                    a.numer(),
                    a.denom(),
                    p.polygon.len(),
                    p.triangles.len()
                )?;
            }
            Ok(())
        },
    )?;
    stream(
        root,
        "vertices.csv",
        "patch,index,x_num,x_den,y_num,y_den",
        |w| {
            for (id, p) in op.patches.iter().enumerate() {
                for (i, q) in p.polygon.iter().enumerate() {
                    writeln!(
                        w,
                        "{id},{i},{},{},{},{}",
                        q[0].numer(),
                        q[0].denom(),
                        q[1].numer(),
                        q[1].denom()
                    )?;
                }
            }
            Ok(())
        },
    )?;
    stream(
        root,
        "triangles.csv",
        "patch,index,ax,ay,bx,by,cx,cy,cross,cross_num,cross_den",
        |w| {
            for (id, p) in op.patches.iter().enumerate() {
                for (i, t) in p.triangles.iter().enumerate() {
                    let [a, b, c] = t.points;
                    writeln!(
                        w,
                        "{id},{i},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{}",
                        a[0],
                        a[1],
                        b[0],
                        b[1],
                        c[0],
                        c[1],
                        t.cross,
                        t.exact_cross.numer(),
                        t.exact_cross.denom()
                    )?;
                }
            }
            Ok(())
        },
    )?;
    stream(root, "quadrature.csv", "q,patch,cell,x,y,weight", |w| {
        for (id, q) in op.quadrature.iter().enumerate() {
            writeln!(
                w,
                "{id},{},{},{:.17e},{:.17e},{:.17e}",
                q.patch, q.cell, q.point[0], q.point[1], q.weight
            )?;
        }
        Ok(())
    })?;
    stream(
        root,
        "bulk-basis.csv",
        "q,active,global,axis,phi,dx,dy",
        |w| {
            for (id, q) in op.quadrature.iter().enumerate() {
                for v in &q.values {
                    writeln!(
                        w,
                        "{id},{},{},{},{:.17e},{:.17e},{:.17e}",
                        op.map[v.global], v.global, v.axis, v.phi, v.dx, v.dy
                    )?;
                }
            }
            Ok(())
        },
    )?;
    stream(
        root,
        "cells.csv",
        "cell,pressure,volume,x_center,y_center,q_pressure",
        |w| {
            for (c, &volume) in op.volume.iter().enumerate() {
                let pressure = op.cell_map[c];
                let pressure = if pressure == usize::MAX {
                    String::new()
                } else {
                    pressure.to_string()
                };
                let x = op.layout.origin[0]
                    + (c % op.layout.n) as f64 * op.layout.h
                    + op.layout.h / 2.0;
                let y = op.layout.origin[1]
                    + (c / op.layout.n) as f64 * op.layout.h
                    + op.layout.h / 2.0;
                let q = 0.75 + x / 8.0 - y / 4.0;
                writeln!(w, "{c},{pressure},{volume:.17e},{x:.17e},{y:.17e},{q:.17e}")?;
            }
            Ok(())
        },
    )?;
    stream(root, "matrices.csv", "block,row,column,value", |w| {
        for (name, m) in [
            ("M", &op.mass),
            ("K", &op.stiffness),
            ("B", &op.divergence),
            ("grid_flux", &tr.grid),
            ("physical_flux", &tr.physical),
        ] {
            for (i, row) in m.iter().enumerate() {
                for (j, v) in row.iter().enumerate() {
                    writeln!(w, "{name},{i},{j},{v:.17e}")?;
                }
            }
        }
        Ok(())
    })?;
    stream(
        root,
        "maps.csv",
        "active,bulk_xx,bulk_xy,bulk_yy,trace_xx,trace_xy,trace_yy,moment_px,moment_py,moment_lz,full_load,omit_load",
        |w| {
            for i in 0..op.active.len() {
                let g = op.gradient[i];
                let t = tr.gradient[i];
                let m = op.moment[i];
                writeln!(
                    w,
                    "{i},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                    g[0],
                    g[1],
                    g[2],
                    t[0],
                    t[1],
                    t[2],
                    m[0],
                    m[1],
                    m[2],
                    r.full_load[i],
                    r.omit_load[i]
                )?;
            }
            Ok(())
        },
    )?;
    stream(
        root,
        "segments.csv",
        "segment,kind,owner,axis,edge,ax,ay,bx,by",
        |w| {
            for (i, s) in tr.records.iter().enumerate() {
                let edge = if s.edge == usize::MAX {
                    String::new()
                } else {
                    s.edge.to_string()
                };
                writeln!(
                    w,
                    "{i},{},{},{},{edge},{:.17e},{:.17e},{:.17e},{:.17e}",
                    s.kind,
                    s.owner,
                    s.axis,
                    s.segment.a[0],
                    s.segment.a[1],
                    s.segment.b[0],
                    s.segment.b[1]
                )?;
            }
            Ok(())
        },
    )?;
    stream(
        root,
        "trace-points.csv",
        "q,segment,x,y,nx_weight,ny_weight",
        |w| {
            for (i, q) in tr.points.iter().enumerate() {
                writeln!(
                    w,
                    "{i},{},{:.17e},{:.17e},{:.17e},{:.17e}",
                    q.record, q.point[0], q.point[1], q.normal_weight[0], q.normal_weight[1]
                )?;
            }
            Ok(())
        },
    )?;
    stream(root, "trace-basis.csv", "q,active,phi", |w| {
        for (i, q) in tr.points.iter().enumerate() {
            for (a, phi) in &q.values {
                writeln!(w, "{i},{a},{phi:.17e}")?;
            }
        }
        Ok(())
    })?;
    stream(root, "metrics.csv", "name,value,gate", |w| {
        for (k, v) in &r.gates {
            writeln!(w, "{k},{v:.17e},true")?;
        }
        for (k, v) in &r.observations {
            writeln!(w, "{k},{v:.17e},false")?;
        }
        Ok(())
    })?;
    Ok(())
}
