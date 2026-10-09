// Presentation-only adapters. No solver imports, integration or inferred fields.
const LIMIT = 8 * 1024 * 1024;
const check = (ok, label) => { if (!ok) throw Error(label); };
const finite = x => typeof x === 'number' && Number.isFinite(x) && Math.abs(x) <= 1e12;
const vector = (v, n = 3) => Array.isArray(v) && v.length === n && Array.from(v).every(finite);
const list = (v, max, label, min = 1) => { check(Array.isArray(v) && v.length >= min && v.length <= max && Array.from(v).every((_, i) => Object.hasOwn(v, i)), label); return v; };
function freeze(value) {
  if (value && typeof value === 'object') { Object.values(value).forEach(freeze); Object.freeze(value); }
  return value;
}
function metric(label, value) { check(finite(value), `Invalid ${label}`); return [label, value]; }
function timeFrames(frames) {
  let previous = -Infinity;
  for (const f of frames) { check(finite(f.time) && f.time >= 0 && f.time > previous, 'Invalid or unordered recorded times'); previous = f.time; }
  return frames;
}
function rigid(raw) {
  check(['free', 'uniform', 'moving', 'pressure'].includes(raw.mode), 'Unsupported rigid-motion mode');
  check(finite(raw.mass_kg) && raw.mass_kg > 0 && finite(raw.spherical_inertia_kg_m2) && raw.spherical_inertia_kg_m2 > 0, 'Invalid rigid-body metadata');
  const steps = list(raw.steps, 64, 'Invalid rigid-motion steps', 0);
  const count = list(raw.initial?.vertices, 8, 'Invalid initial vertices').length;
  check(count === 8, 'Only the documented eight-vertex rigid fixture is supported');
  const triangles = list(raw.triangles, 12, 'Invalid triangles');
  check(triangles.length === 12 && triangles.every(t => Array.isArray(t) && t.length === 3 && Array.from(t).every(i => Number.isInteger(i) && i >= 0 && i < count)), 'Invalid triangle indices');
  function frame(s, step, prior) {
    check(s && vector(s.center_of_mass) && vector(s.orientation, 4) && vector(s.velocity_m_s) && vector(s.angular_velocity_rad_s), 'Invalid recorded rigid frame');
    check(list(s.vertices, count, 'Invalid recorded mesh vertices').length === count && s.vertices.every(v => vector(v)), 'Invalid recorded mesh vertices');
    const metrics = [['Mass (kg)', raw.mass_kg], ['Velocity (m/s)', s.velocity_m_s], ['Angular velocity (rad/s)', s.angular_velocity_rad_s]];
    let force;
    if (step) {
      check(vector(step.force_n) && vector(step.torque_n_m), 'Invalid recorded loads');
      force = {value: step.force_n, origin: prior.center_of_mass};
      metrics.push(['Preceding update force (N)', step.force_n], ['Preceding update torque (N m)', step.torque_n_m], metric('Kinetic energy (J)', step.kinetic_after_j), metric('Energy defect (J)', step.energy_defect_j));
    }
    return {time: s.time_s, vertices: s.vertices, triangles, center: s.center_of_mass, force, metrics};
  }
  const frames = [frame(raw.initial)]; let prior = raw.initial;
  for (const s of steps) { frames.push(frame(s.stored, s, prior)); prior = s.stored; }
  return {kind: 'rigid', label: `Rigid mesh · ${raw.mode}`, cases: [{label: raw.mode, frames: timeFrames(frames)}]};
}
function obstacle(raw) {
  check(raw.schema === 'rheon-obstacle-flow-records-v1', 'Unsupported obstacle record version');
  const cases = list(raw.shear_cases, 32, 'Invalid shear cases').map(c => {
    check(typeof c.id === 'string' && c.id.length <= 120 && finite(c.dt) && c.dt > 0, 'Invalid shear metadata');
    const centers = list(c.centers, 512, 'Invalid shear centers'); check(centers.every(finite), 'Invalid shear centers');
    const frames = list(c.frames, 512, 'Invalid shear frames').map(f => {
      check(Number.isSafeInteger(f.step) && f.step >= 0 && vector(f.velocity, centers.length), 'Invalid shear samples');
      const metrics = [['Recorded step', f.step], ['Density (kg/m³)', c.density], ['Viscosity (Pa s)', c.viscosity]].map(([k, v]) => metric(k, v));
      if (f.energy) { check(vector(f.energy, 8), 'Invalid energy ledger'); metrics.push(['Kinetic energy (J)', f.energy[1]], ['Body work (J)', f.energy[3]]); }
      return {time: f.step * c.dt, profile: centers.map((y, i) => [y, f.velocity[i]]), metrics};
    });
    return {label: c.id, frames: timeFrames(frames)};
  });
  return {kind: 'profile', label: 'Reduced obstacle shear · recorded', cases};
}
function column(raw) {
  const cases = list(raw.cases, 32, 'Invalid column cases').map(c => {
    check(typeof c.name === 'string' && c.name.length <= 120, 'Invalid column name');
    const frames = list(c.snapshots, 512, 'Invalid column snapshots').map(s => {
      const rows = list(s.profiles, 512, 'Invalid column profiles');
      check(rows.every(r => Array.isArray(r) && r.length >= 2 && r.length <= 8 && Array.from(r).every(finite)), 'Invalid column samples');
      return {time: s.time, profile: rows.map(r => r.slice(0, 2)), metrics: [metric('Kinetic energy (J)', s.energy), metric('Bulk loss (J)', s.bulk)]};
    });
    return {label: c.name, frames: timeFrames(frames)};
  });
  return {kind: 'profile', label: 'Native fixed-slab profiles · recorded', cases};
}
export function adaptRecording(raw, provenance) {
  check(raw && typeof raw === 'object', 'A native JSON object is required');
  // A clone ensures adapters never freeze or change the caller's data.
  const copy = structuredClone(raw);
  check(copy.schema === undefined || copy.schema === 'rheon-obstacle-flow-records-v1', 'Unsupported record version');
  const data = copy.schema === 'rheon-obstacle-flow-records-v1' ? obstacle(copy)
    : copy.initial && copy.steps ? rigid(copy) : copy.cases ? column(copy) : null;
  check(data, 'Unsupported record format. Use rigid_motion, obstacle-flow shear or native column JSON.');
  return freeze({...data, provenance: {...provenance, qualification: 'Imported visualization; not independently qualified by this viewer'}});
}
export async function readRecording(file) {
  check(file.size > 0 && file.size <= LIMIT, 'Record file must be at most 8 MiB');
  const bytes = await file.arrayBuffer(); check(bytes.byteLength <= LIMIT, 'Record file exceeds 8 MiB');
  const digest = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))].map(x => x.toString(16).padStart(2, '0')).join('');
  const raw = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes));
  return adaptRecording(raw, {name: file.name, sha256: digest, source: 'Local file'});
}
export const KENOMA_COMMIT = '3e7ff0887d01a1f4c440d3808770ea3eac7ec8d4';
export function mountKenoma(host, config) {
  check(config?.commit === KENOMA_COMMIT && config.protocol === 1 && config.rig_version === 1, 'Unsupported Kenoma component identity');
  const url = new URL('./kenoma/index.html', import.meta.url);
  const frame = document.createElement('iframe'); frame.title = 'Kenoma simple-human pose editor';
  frame.src = url.href; host.append(frame);
  // No parent messages, solver coupling, scene interpretation or state injection.
  return () => frame.remove();
}
