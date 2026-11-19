//! Read-only bounded interior slice of accepted f64 volume fractions.
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use rheon::recording::RecordedVoxelMaterial;
#[derive(Default)]
pub(super) struct Slice {
    axis: usize,
    layer: usize,
    probe: [usize; 3],
}
fn color(f: f64) -> Color32 {
    let liquid = [45.0, 125.0, 215.0];
    let gas = [135.0, 103.0, 49.0];
    let c = std::array::from_fn::<_, 3, _>(|d| (f * liquid[d] + (1.0 - f) * gas[d]).round() as u8);
    Color32::from_rgb(c[0], c[1], c[2])
}
pub(super) fn show(
    ui: &mut egui::Ui,
    record: &RecordedVoxelMaterial,
    slice: &mut Slice,
) -> Option<usize> {
    let grid = &record.grid;
    let n = grid.counts();
    let pair = record.metadata.materials;
    ui.heading("Voxel materials");
    ui.small("Filled incompressible liquid/gas · mixed cells retain both fractions · color is display only");
    let volume = grid.cell_volume();
    let liquid = record.liquid_fraction.iter().sum::<f64>() * volume;
    let gas = record.liquid_fraction.iter().map(|f| 1.0 - f).sum::<f64>() * volume;
    let masses = [liquid * pair.liquid().density(), gas * pair.gas().density()];
    if [liquid, gas, masses[0], masses[1]]
        .iter()
        .all(|v| v.is_finite())
    {
        ui.label(format!("Liquid: {liquid:.9e} m³ · {:.9e} kg", masses[0]));
        ui.label(format!("Occupied gas: {gas:.9e} m³ · {:.9e} kg", masses[1]));
    } else {
        ui.label("Volume/mass display unavailable at this scale");
    }
    ui.small(format!(
        "Captured ρ Liquid {:.9e}, Occupied gas {:.9e} kg/m³ · μ Liquid {:.9e}, Occupied gas {:.9e} Pa·s",
        pair.liquid().density(),
        pair.gas().density(),
        pair.liquid().viscosity(),
        pair.gas().viscosity()
    ));
    ui.horizontal(|ui| {
        ui.colored_label(color(1.0), "Liquid");
        ui.colored_label(color(0.0), "Occupied gas");
        ui.small("Mixed cells blend display color");
    });
    let previous = (slice.axis, slice.layer);
    ui.horizontal(|ui| {
        ui.label("Slice axis");
        for (d, label) in ["X", "Y", "Z"].into_iter().enumerate() {
            ui.selectable_value(&mut slice.axis, d, label);
        }
    });
    slice.layer = slice.layer.min(n[slice.axis] - 1);
    ui.add(egui::Slider::new(&mut slice.layer, 0..=n[slice.axis] - 1).text("Recorded layer"));
    let mut changed = previous != (slice.axis, slice.layer);
    if changed {
        slice.probe[slice.axis] = slice.layer;
    }
    ui.small("Cell-index slice; numeric probe reports physical coordinates");
    let axes = match slice.axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let size = Vec2::new(ui.available_width().clamp(1.0, 280.0), 180.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let painter = ui.painter_at(rect);
    for j in 0..n[axes[1]] {
        for i in 0..n[axes[0]] {
            let mut cell = [0; 3];
            cell[slice.axis] = slice.layer;
            cell[axes[0]] = i;
            cell[axes[1]] = j;
            let f = record.liquid_fraction[grid.cell_index(cell).unwrap()];
            let at = |x: usize, y: usize| {
                Pos2::new(
                    rect.left() + x as f32 / n[axes[0]] as f32 * rect.width(),
                    rect.bottom() - y as f32 / n[axes[1]] as f32 * rect.height(),
                )
            };
            painter.rect_filled(
                Rect::from_two_pos(at(i, j), at(i + 1, j + 1)),
                0.0,
                color(f),
            );
        }
    }
    if response.clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        slice.probe[slice.axis] = slice.layer;
        slice.probe[axes[0]] = (((p.x - rect.left()) / rect.width() * n[axes[0]] as f32).floor()
            as usize)
            .min(n[axes[0]] - 1);
        slice.probe[axes[1]] = (((rect.bottom() - p.y) / rect.height() * n[axes[1]] as f32).floor()
            as usize)
            .min(n[axes[1]] - 1);
        changed = true;
    }
    ui.horizontal_wrapped(|ui| {
        for (d, label) in ["Cell i", "Cell j", "Cell k"].into_iter().enumerate() {
            slice.probe[d] = slice.probe[d].min(n[d] - 1);
            let label = ui.label(label);
            changed |= ui
                .add(egui::DragValue::new(&mut slice.probe[d]).range(0..=n[d] - 1))
                .labelled_by(label.id)
                .changed();
        }
    });
    let cell = grid.cell_index(slice.probe).unwrap();
    let f = record.liquid_fraction[cell];
    ui.label(format!(
        "Cell {:?} · center {:?} m",
        slice.probe,
        grid.cell_position(slice.probe).unwrap()
    ));
    ui.label(format!(
        "Liquid F={f:.17e} · Occupied gas F={:.17e}",
        1.0 - f
    ));
    ui.small("Exact original fraction, pressure and staggered velocities are available in recorded fields / CSV below. This view is not a restart state.");
    changed.then_some(cell)
}
