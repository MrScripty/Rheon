// Evaluate the immutable native finite-row specimen. No timestep or projection.
const $ = id => document.getElementById(id);
const NS = 'http://www.w3.org/2000/svg';
const AXES = ['x', 'y', 'z'];
const COLORS = ['#bb5e35', '#087f90', '#7761a7'];
const PRESET_NAMES = ['corner', 'normal', 'shear', 'rotation'];
const finite = x => typeof x === 'number' && Number.isFinite(x);
const compact = x => Math.abs(x) < 1e-13 ? '0' : Number(x.toPrecision(5)).toString();
const scientific = x => x === 0 ? '0' : Math.abs(x) < 0.0001 || Math.abs(x) >= 10000 ? x.toExponential(3) : compact(x);
const same = (a, b) => a.length === b.length && a.every((x, i) => x === b[i]);
function require(value, message) { if (!value) throw new Error(message); }
function freeze(value) {
  if (value && typeof value === 'object') { for (const child of Object.values(value)) freeze(child); Object.freeze(value); }
  return value;
}
function element(name, attrs = {}, content) {
  const node = document.createElementNS(NS, name);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, String(value));
  if (content !== undefined) node.textContent = content;
  return node;
}
function draw(root, name, attrs, content) { const node = element(name, attrs, content); root.append(node); return node; }
function metric(id, value, unit) { $(id).textContent = `${scientific(value)}${unit ? ` ${unit}` : ''}`; $(id).dataset.value = String(value); $(id).dataset.metric = id.replace('strain-', ''); }
function definition(root, label, value) {
  const dt = document.createElement('dt'), dd = document.createElement('dd');
  dt.textContent = label; dd.textContent = value; root.append(dt, dd); return dd;
}
function rowName(row) {
  const pair = row.axes.map(a => AXES[a]).join('');
  const kind = row.axes[0] === row.axes[1] ? `normal ${pair}` : `shear ${pair} · q${row.quadrant}`;
  return `#${row.id} · ${kind} · (${row.coordinates.join(', ')})${row.terms.length ? '' : ' · zero row'}`;
}
function bounded(value, limits, name) {
  require(finite(value) && value >= limits[0] && value <= limits[1], `Invalid ${name}`);
  return value;
}
function validate(packet) {
  require(packet && packet.schema === 'rheon-aligned-strain-education-packet-v1' && packet.meta && packet.source, 'Missing or unsupported source-bound native packet');
  require(same(packet.meta.counts, [3, 3, 3]), 'This view requires the qualified 3³ specimen');
  require(packet.active.length > 0 && packet.active.length <= 100 && packet.rows.length <= 400, 'Packet exceeds the bounded specimen');
  packet.active.forEach((face, id) => {
    require(face.id === id && [0, 1, 2].includes(face.axis), 'Active-face indexing');
    require(finite(face.mass) && face.mass > 0 && face.position.length === 3 && face.position.every(finite), 'Invalid stored face geometry');
  });
  packet.rows.forEach((row, id) => {
    require(row.id === id && row.axes.length === 2 && row.axes.every(a => [0, 1, 2].includes(a)), 'Row indexing');
    require(finite(row.weight) && row.weight > 0 && row.terms.length <= 4, 'Invalid stored row weight');
    row.terms.forEach(term => require(Number.isInteger(term.active) && term.active >= 0 && term.active < packet.active.length && finite(term.coefficient), 'Invalid stored row term'));
  });
  for (const name of PRESET_NAMES) {
    const preset = packet.presets[name];
    require(preset && preset.basis.length === 2 && preset.basis.every(basis => basis.length === packet.active.length && basis.every(finite)), `Invalid ${name} preset`);
  }
  for (const key of ['amplitude', 'secondary', 'mu']) require(packet.controlLimits[key].length === 2 && packet.controlLimits[key].every(finite), 'Invalid control limits');
}
function start() {
  const packet = window.ALIGNED_STRAIN_PACKET;
  validate(packet); freeze(packet);
  const rows = packet.rows, faces = packet.active, meta = packet.meta;
  const defaults = {corner: [1, 0.5], normal: [1, -0.5], shear: [1, 0.5], rotation: [1, 0]};
  const defaultCorner = packet.cornerEdges.find(edge => same(edge.axes, [0, 1]) && same(edge.coordinates, [2, 2, 1]));
  require(defaultCorner, 'Missing qualified default corner');
  const state = {preset: 'corner', cornerSelection: defaultCorner.id, amplitude: 1, secondary: 0.5, mu: 1, rowSelection: 0, slice: 1, showForce: false};
  const selectedCorner = () => packet.cornerEdges.find(edge => edge.id === state.cornerSelection);
  let current;
  const estimate = faces.map(() => 0);
  for (const row of rows) {
    const sum = row.terms.reduce((s, term) => s + Math.abs(term.coefficient), 0);
    for (const term of row.terms) estimate[term.active] += row.weight * Math.abs(term.coefficient) * sum;
  }
  const B = Math.max(0, ...estimate.map((value, id) => value / faces[id].mass));
  const field = () => {
    const [first, second] = state.preset === 'corner' ? selectedCorner().basis : packet.presets[state.preset].basis;
    return first.map((value, id) => state.amplitude * value + state.secondary * second[id]);
  };
  function evaluate() {
    const u = field(), strains = [], action = faces.map(() => 0), losses = [];
    let normalD = 0, shearD = 0;
    for (const row of rows) {
      const strain = row.terms.reduce((sum, term) => sum + term.coefficient * u[term.active], 0);
      strains.push(strain);
      const weighted = row.weight * strain;
      for (const term of row.terms) action[term.active] += term.coefficient * weighted;
      const loss = state.mu * row.weight * strain * strain;
      losses.push(loss);
      if (row.axes[0] === row.axes[1]) normalD += loss; else shearD += loss;
    }
    const forces = action.map(value => -state.mu * value);
    const D = normalD + shearD, work = u.reduce((sum, value, id) => sum + value * forces[id], 0);
    const energy = u.reduce((sum, value, id) => sum + 0.5 * faces[id].mass * value * value, 0);
    return {field: u, strains, action, forces, normalD, shearD, D, work, identity: D + work, B, energy, losses};
  }
  function preferredRow(preset) {
    if (preset === 'corner') {
      const edge = selectedCorner();
      return edge ? edge.rowIds.find(id => rows[id].terms.length === 2) : 0;
    }
    const focus = packet.presets[preset].focusRowIds;
    if (focus && focus.length) return focus[0];
    if (preset === 'normal') return rows.find(row => same(row.axes, [0, 0]) && row.terms.length === 2)?.id ?? 0;
    return rows.find(row => same(row.axes, [0, 1]) && same(row.coordinates, [1, 1, 0]))?.id ?? 0;
  }
  function selectRow(id, adjustSlice = true) {
    require(Number.isInteger(id) && id >= 0 && id < rows.length, 'Invalid row selection');
    state.rowSelection = id;
    if (adjustSlice) {
      const row = rows[id];
      state.slice = row.axes[0] !== row.axes[1] && row.axes.includes(2) ? Math.max(0, row.coordinates[2] - 1) : row.coordinates[2];
      state.slice = Math.min(meta.counts[2] - 1, state.slice);
    }
  }
  function controls() {
    const preset = packet.presets[state.preset];
    $('strain-preset').value = state.preset;
    $('strain-corner').value = state.cornerSelection; $('strain-corner').disabled = state.preset !== 'corner';
    $('strain-preset-description').textContent = preset.description;
    const labels = {
      corner: ['U · x-face speed', 'V · y-face speed', 'm/s'],
      normal: ['x stretch rate', 'y stretch rate', 's⁻¹'],
      shear: ['∂y uₓ sample rate', '∂x uᵧ sample rate', 's⁻¹'],
      rotation: ['ω · local rotation rate', 'Second component unused', 's⁻¹']
    }[state.preset];
    $('strain-amplitude-label').textContent = state.preset === 'corner' ? `U · u${AXES[selectedCorner().axes[0]]} face speed` : preset.primaryLabel ?? labels[0];
    $('strain-secondary-label').textContent = state.preset === 'corner' ? `V · u${AXES[selectedCorner().axes[1]]} face speed` : preset.secondaryLabel ?? labels[1];
    $('strain-amplitude').value = String(state.amplitude); $('strain-secondary').value = String(state.secondary); $('strain-mu').value = String(state.mu);
    $('strain-amplitude-value').textContent = `${compact(state.amplitude)} ${preset.units ?? labels[2]}`;
    $('strain-secondary-value').textContent = state.preset === 'rotation' ? '—' : `${compact(state.secondary)} ${preset.units ?? labels[2]}`;
    $('strain-secondary').disabled = state.preset === 'rotation';
    $('strain-mu-value').textContent = `${compact(state.mu)} Pa s`;
    $('strain-row').value = String(state.rowSelection); $('strain-slice').value = String(state.slice); $('strain-show-force').checked = state.showForce;
  }
  function scene() {
    const root = $('strain-view'); root.replaceChildren();
    draw(root, 'title', {id: 'strain-view-title'}, 'Actual aligned-box MAC face velocities and selected strain support');
    draw(root, 'desc', {id: 'strain-view-description'}, 'The solid cell is stationary. Colored arrows are active velocities; highlighted circles are coefficients of the selected row. Warm fluid quadrants show their actual loss.');
    const row = rows[state.rowSelection], n = meta.counts, origin = meta.origin, h = meta.spacing;
    const x0 = 92, y0 = 455, size = 125;
    const px = x => x0 + (x - origin[0]) / h[0] * size;
    const py = y => y0 - (y - origin[1]) / h[1] * size;
    const plane = (d, p) => origin[d] + p * h[d];
    const cellCenter = (d, p) => origin[d] + (p + 0.5) * h[d];
    const z = cellCenter(2, state.slice), solidLayer = z > meta.lower[2] && z < meta.upper[2];
    const solid = (i, j) => solidLayer && plane(0, i) >= meta.lower[0] && plane(0, i + 1) <= meta.upper[0] && plane(1, j) >= meta.lower[1] && plane(1, j + 1) <= meta.upper[1];
    draw(root, 'text', {x: x0, y: 28, fill: '#52676d', 'font-size': 12}, `x–y slice · z = ${compact(z)} m · ρ = ${compact(meta.density)} kg/m³`);
    const cellLoss = Array(9).fill(0);
    for (const candidate of rows) if (candidate.axes[0] === candidate.axes[1] && candidate.coordinates[2] === state.slice) cellLoss[candidate.coordinates[0] + n[0] * candidate.coordinates[1]] += current.losses[candidate.id];
    const maxCellLoss = Math.max(1e-30, ...cellLoss);
    for (let j = 0; j < n[1]; j++) for (let i = 0; i < n[0]; i++) {
      const dry = solid(i, j), value = cellLoss[i + n[0] * j];
      const fill = dry ? '#344d51' : `hsl(168 22% ${97 - 13 * value / maxCellLoss}%)`;
      draw(root, 'rect', {x: px(plane(0, i)), y: py(plane(1, j + 1)), width: size, height: size, fill, stroke: '#c6d6cf', 'stroke-width': 1});
      if (dry) {
        draw(root, 'rect', {x: px(plane(0, i)) + 3, y: py(plane(1, j + 1)) + 3, width: size - 6, height: size - 6, fill: 'none', stroke: '#dcb995', 'stroke-width': 3});
        draw(root, 'text', {x: px(cellCenter(0, i)), y: py(cellCenter(1, j)) - 3, 'text-anchor': 'middle', fill: '#fff7e9', 'font-size': 18}, 'Stationary');
        draw(root, 'text', {x: px(cellCenter(0, i)), y: py(cellCenter(1, j)) + 21, 'text-anchor': 'middle', fill: '#ddc6ae', 'font-size': 13}, 'no-slip box');
      }
    }
    const siblings = row.axes[0] !== row.axes[1] ? rows.filter(r => same(r.axes, row.axes) && same(r.coordinates, row.coordinates)) : [row];
    if (same(row.axes, [0, 1]) && row.coordinates[2] === state.slice) {
      const max = Math.max(1e-30, ...siblings.map(r => current.losses[r.id]));
      for (const sector of siblings) {
        const i = row.coordinates[0], j = row.coordinates[1], q = sector.quadrant;
        const edgeX = plane(0, i), edgeY = plane(1, j);
        const farX = cellCenter(0, i - (q & 1 ? 0 : 1)), farY = cellCenter(1, j - (q & 2 ? 0 : 1));
        const loss = current.losses[sector.id];
        const attrs = {x: Math.min(px(edgeX), px(farX)), y: Math.min(py(edgeY), py(farY)), width: Math.abs(px(edgeX) - px(farX)), height: Math.abs(py(edgeY) - py(farY)), fill: `hsl(35 65% ${94 - 28 * loss / max}%)`, stroke: sector.id === row.id ? '#aa5735' : '#d9b67f', 'stroke-width': sector.id === row.id ? 2.5 : 1, tabindex: 0, role: 'button', 'aria-label': `Select actual quadrant ${q}, row ${sector.id}`, 'data-row': sector.id};
        const patch = draw(root, 'rect', attrs);
        patch.addEventListener('click', () => { selectRow(sector.id); render(); });
        patch.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); selectRow(sector.id); render(); } });
        draw(root, 'text', {x: (px(edgeX) + px(farX)) / 2, y: (py(edgeY) + py(farY)) / 2 + 4, 'text-anchor': 'middle', 'font-size': 10, fill: '#7e4e21'}, `${compact(loss)} W`);
      }
      draw(root, 'circle', {cx: px(plane(0, row.coordinates[0])), cy: py(plane(1, row.coordinates[1])), r: 4.5, fill: '#aa5735'});
    } else if (row.axes[0] === row.axes[1] && row.coordinates[2] === state.slice) {
      draw(root, 'rect', {x: px(plane(0, row.coordinates[0])) + 2, y: py(plane(1, row.coordinates[1] + 1)) + 2, width: size - 4, height: size - 4, fill: 'none', stroke: '#aa5735', 'stroke-width': 2.5, 'stroke-dasharray': '5 4'});
    }
    draw(root, 'rect', {x: x0, y: y0 - 3 * size, width: 3 * size, height: 3 * size, fill: 'none', stroke: '#087f90', 'stroke-width': 2.5, 'stroke-dasharray': '7 5'});
    draw(root, 'text', {x: x0 + 1.5 * size, y: 58, 'text-anchor': 'middle', fill: '#087f90', 'font-size': 12}, 'SEALED OUTER WALLS · FREE SLIP');
    draw(root, 'text', {x: 46, y: y0 - 1.5 * size, transform: `rotate(-90 46 ${y0 - 1.5 * size})`, 'text-anchor': 'middle', fill: '#087f90', 'font-size': 11}, 'uₙ = 0 · ∂ₙuₜ = 0');
    for (let k = 0; k <= 3; k++) {
      draw(root, 'text', {x: x0 + k * size, y: y0 + 25, 'text-anchor': 'middle', 'font-size': 11, fill: '#52676d'}, compact(plane(0, k)));
      draw(root, 'text', {x: x0 - 13, y: y0 - k * size + 4, 'text-anchor': 'end', 'font-size': 11, fill: '#52676d'}, compact(plane(1, k)));
    }
    draw(root, 'text', {x: x0 + 3 * size + 17, y: y0 + 25, 'font-size': 12, fill: '#52676d'}, 'x (m)');
    draw(root, 'text', {x: x0 - 35, y: y0 - 3 * size - 7, 'font-size': 12, fill: '#52676d'}, 'y');
    function arrow(x, y, dx, dy, color, dashed = false) {
      if (Math.hypot(dx, dy) < 0.05) return;
      draw(root, 'line', {x1: x, y1: y, x2: x + dx, y2: y + dy, stroke: color, 'stroke-width': dashed ? 1.8 : 2.5, ...(dashed ? {'stroke-dasharray': '3 2'} : {})});
      const mag = Math.hypot(dx, dy), ux = dx / mag, uy = dy / mag;
      draw(root, 'path', {d: `M ${x + dx - 6 * ux + 3 * uy} ${y + dy - 6 * uy - 3 * ux} L ${x + dx} ${y + dy} L ${x + dx - 6 * ux - 3 * uy} ${y + dy - 6 * uy + 3 * ux}`, fill: 'none', stroke: color, 'stroke-width': 1.8});
    }
    const support = new Set(row.terms.map(term => term.active));
    const maxForce = Math.max(0, ...current.forces.map(Math.abs));
    for (const face of faces) {
      const inSlice = face.axis === 2 ? (face.coordinates[2] === state.slice || face.coordinates[2] === state.slice + 1) : face.coordinates[2] === state.slice;
      if (!inSlice && !support.has(face.id)) continue;
      let x = px(face.position[0]), y = py(face.position[1]);
      if (face.axis === 2) x += face.coordinates[2] === state.slice ? -11 : 11;
      const group = draw(root, 'g', {opacity: inSlice ? 1 : 0.45});
      if (support.has(face.id)) draw(group, 'circle', {cx: x, cy: y, r: 12, fill: '#fff8ea', stroke: '#aa5735', 'stroke-width': 2, 'stroke-dasharray': inSlice ? '0' : '3 2'});
      const target = draw(group, 'circle', {cx: x, cy: y, r: face.axis === 2 ? 5 : 3.5, fill: face.axis === 2 ? '#f3edff' : COLORS[face.axis], stroke: COLORS[face.axis], 'stroke-width': 1.5, class: 'strain-face', tabindex: 0, role: 'button', 'aria-label': `Active face ${face.id}, u${AXES[face.axis]}, speed ${compact(current.field[face.id])} meters per second`, 'data-active': face.id});
      const pick = () => { const candidate = rows.find(candidate => candidate.terms.some(term => term.active === face.id) && candidate.coordinates[2] === state.slice); if (candidate) { selectRow(candidate.id); render(); } };
      target.addEventListener('click', pick); target.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); pick(); } });
      const value = current.field[face.id];
      if (face.axis < 2) {
        arrow(x, y, face.axis === 0 ? 34 * value : 0, face.axis === 1 ? -34 * value : 0, COLORS[face.axis]);
        if (Math.abs(value) > 1e-10 || support.has(face.id)) draw(group, 'text', {x: x + (face.axis === 1 ? 10 : 0), y: y + (face.axis === 0 ? -10 : 4), 'text-anchor': face.axis === 0 ? 'middle' : 'start', 'font-size': 11, fill: COLORS[face.axis]}, compact(value));
        if (state.showForce && maxForce > 0) arrow(x, y, face.axis === 0 ? 40 * current.forces[face.id] / maxForce : 0, face.axis === 1 ? -40 * current.forces[face.id] / maxForce : 0, '#263b40', true);
      } else if (Math.abs(value) > 1e-10) {
        draw(group, 'text', {x, y: y + 3.5, 'text-anchor': 'middle', 'font-size': 10, fill: COLORS[2]}, value > 0 ? '·' : '×');
        draw(group, 'text', {x, y: y + 18, 'text-anchor': 'middle', 'font-size': 9, fill: COLORS[2]}, compact(value));
      }
      if (support.has(face.id)) draw(group, 'text', {x, y: y + (face.axis === 2 ? -17 : 23), 'text-anchor': 'middle', 'font-size': 10, fill: '#aa5735'}, `f${face.id}${inSlice ? '' : ' · other z'}`);
    }
    draw(root, 'text', {x: x0, y: 513, 'font-size': 12, fill: '#52676d'}, `Selected ${row.axes[0] === row.axes[1] ? 'normal cell' : 'shear edge'} row #${row.id} · strain ${compact(current.strains[row.id])} s⁻¹`);
    draw(root, 'text', {x: x0, y: 537, 'font-size': 11, fill: '#52676d'}, `${support.size} active coefficients · ${rows.filter(r => !r.terms.length).length} retained zero rows in specimen`);
    $('strain-scene-caption').textContent = `Velocity arrows: 34 px per m/s. Cell tint shows normal loss; warm patches show selected x–y edge sector losses. ${state.showForce ? `Dashed in-plane force arrows share a normalized scale; maximum over all active faces is ${compact(maxForce)} N. ` : ''}Purple markers show near/far z-normal faces. Wall traces are prescribed zeros.`;
    root.dataset.selectedRow = String(row.id); root.dataset.slice = String(state.slice); root.dataset.preset = state.preset;
  }
  function inspector() {
    const row = rows[state.rowSelection], strain = current.strains[row.id], boundary = row.boundary;
    $('strain-inspector').dataset.rowId = String(row.id); $('strain-inspector').dataset.strain = String(strain);
    const inspectedEdge = packet.cornerEdges.find(edge => edge.rowIds.includes(row.id));
    if (inspectedEdge) $('strain-inspector').dataset.cornerId = inspectedEdge.id; else delete $('strain-inspector').dataset.cornerId;
    const normal = row.axes[0] === row.axes[1];
    $('strain-row-location').textContent = `${normal ? 'Cell' : 'Internal edge'} (${row.coordinates.join(', ')}) · ${normal ? `normal ${AXES[row.axes[0]]}` : `engineering shear ${row.axes.map(a => AXES[a]).join('')} · quadrant ${row.quadrant}`} · native row ${row.id}`;
    $('strain-row-formula').textContent = `s${row.id} = ${row.terms.length ? row.terms.map((term, i) => `${i ? term.coefficient < 0 ? '− ' : '+ ' : term.coefficient < 0 ? '−' : ''}${compact(Math.abs(term.coefficient))} u${term.active}`).join(' ') : '0 (all represented traces eliminated)'} = ${compact(strain)} s⁻¹`;
    const values = $('strain-row-values'); values.replaceChildren();
    definition(values, normal ? 'Weight 2V' : 'Actual fluid-sector volume', `${compact(row.weight)} m³`);
    const strainMetric = definition(values, normal ? 'Normal strain ∂ₐuₐ' : 'Engineering shear γₐᵦ', `${compact(strain)} s⁻¹`);
    strainMetric.id = 'strain-row-strain'; strainMetric.dataset.value = String(strain);
    if (!normal) definition(values, 'Symmetric tensor entry εₐᵦ', `${compact(strain / 2)} s⁻¹`);
    const lossMetric = definition(values, 'Weighted loss μ w s²', `${compact(current.losses[row.id])} W`);
    lossMetric.dataset.metric = 'rowD'; lossMetric.dataset.value = String(current.losses[row.id]);
    definition(values, 'Weighted base strain w s', `${compact(row.weight * strain)} m³/s`);
    const table = $('strain-row-terms'); table.replaceChildren();
    for (const term of row.terms) {
      const face = faces[term.active], tr = document.createElement('tr');
      for (const value of [`f${face.id} · u${AXES[face.axis]} (${face.coordinates.join(',')})`, `${compact(term.coefficient)} m⁻¹`, `${compact(current.field[face.id])} m/s`, `${compact(term.coefficient * current.field[face.id])} s⁻¹`]) { const td = document.createElement('td'); td.textContent = value; tr.append(td); }
      tr.dataset.active = String(face.id); table.append(tr);
    }
    if (!row.terms.length) { const tr = document.createElement('tr'), td = document.createElement('td'); td.colSpan = 4; td.textContent = 'Zero coefficients. This fluid quadrature row remains retained.'; tr.append(td); table.append(tr); }
    const boundaryName = typeof boundary === 'string' ? boundary.toLowerCase() : ({0: 'interior', 1: 'obstacleflat', 2: 'obstaclecorner', 3: 'outerfreeslip', 4: 'normal'})[boundary];
    const explanations = {interior: 'Both centered differences use their represented adjacent-center distances. All four fluid quadrants carry this same engineering-shear row.', obstacleflat: 'Stationary no-slip traces are eliminated. The surviving tangential sample reaches zero over its represented center-to-wall distance.', obstaclecorner: 'The two hinge derivatives contribute only on their actual fluid quadrants. The shared sector couples the two represented velocity components.', normal: 'The cell-normal difference uses the represented cell width. Stationary obstacle and outer-wall normal traces are eliminated zeros.'};
    $('strain-row-boundary').textContent = `${explanations[boundaryName] ?? 'This row is copied from the native operator, including its eliminated stationary traces.'} Scatter each support coefficient as −μ Eᵣf wᵣ sᵣ.`;
    const edge = inspectedEdge ?? selectedCorner();
    document.querySelector('.strain-corner-panel').dataset.cornerId = edge.id;
    if (edge) {
      const patchLoss = edge.rowIds.reduce((sum, id) => sum + current.losses[id], 0);
      const [u, v] = edge.activeIds.map(id => current.field[id]);
      $('strain-corner-live').textContent = `Inspected ${edge.axes.map(a => AXES[a]).join('')} patch (${edge.coordinates.join(', ')}): physical-component block ${JSON.stringify(edge.unitPatchMatrix)}; U = ${compact(u)} m/s, V = ${compact(v)} m/s. Three-row loss = ${compact(patchLoss)} W. Reflection changes the cross-term sign in physical components. The complete specimen includes additional rows.`;
    }
  }
  function render() {
    current = evaluate(); controls(); scene(); inspector();
    metric('strain-normalD', current.normalD, 'W'); metric('strain-shearD', current.shearD, 'W'); metric('strain-D', current.D, 'W'); metric('strain-work', current.work, 'W');
    metric('strain-identity', current.identity, ''); metric('strain-energy', current.energy, ''); metric('strain-B', current.B, '');
    $('strain-status').textContent = `${faces.length} pressure-active faces · ${rows.length} native strain rows. Every change recomputes gather and transpose scatter.`;
  }
  function setControls(changes) {
    if (changes.preset !== undefined) {
      require(PRESET_NAMES.includes(changes.preset), 'Invalid preset'); state.preset = changes.preset;
      [state.amplitude, state.secondary] = defaults[state.preset]; selectRow(preferredRow(state.preset));
    }
    if (changes.cornerSelection !== undefined) {
      require(packet.cornerEdges.some(edge => edge.id === changes.cornerSelection), 'Invalid corner selection'); state.cornerSelection = changes.cornerSelection; selectRow(preferredRow('corner'));
    }
    for (const key of ['amplitude', 'secondary', 'mu']) if (changes[key] !== undefined) state[key] = bounded(changes[key], packet.controlLimits[key], key);
    if (changes.rowSelection !== undefined) selectRow(changes.rowSelection);
    if (changes.slice !== undefined) { require(Number.isInteger(changes.slice) && changes.slice >= 0 && changes.slice < meta.counts[2], 'Invalid slice'); state.slice = changes.slice; }
    if (changes.showForce !== undefined) { require(typeof changes.showForce === 'boolean', 'Invalid force overlay'); state.showForce = changes.showForce; }
    render();
  }
  for (const name of PRESET_NAMES) { const option = document.createElement('option'); option.value = name; option.textContent = packet.presets[name].label; $('strain-preset').append(option); }
  for (const edge of packet.cornerEdges) { const option = document.createElement('option'); option.value = edge.id; option.textContent = `${edge.axes.map(a => AXES[a]).join('')} · (${edge.coordinates.join(', ')})`; $('strain-corner').append(option); }
  for (let i = 0; i < meta.counts[2]; i++) { const option = document.createElement('option'); option.value = String(i); option.textContent = `${i} · z ${compact(meta.origin[2] + (i + 0.5) * meta.spacing[2])} m`; $('strain-slice').append(option); }
  for (const row of rows) { const option = document.createElement('option'); option.value = String(row.id); option.textContent = rowName(row); $('strain-row').append(option); }
  for (const key of ['amplitude', 'secondary', 'mu']) {
    const input = $(`strain-${key}`); [input.min, input.max] = packet.controlLimits[key].map(String);
    input.addEventListener('input', () => setControls({[key]: Number(input.value)}));
  }
  $('strain-preset').addEventListener('change', () => setControls({preset: $('strain-preset').value}));
  $('strain-corner').addEventListener('change', () => setControls({cornerSelection: $('strain-corner').value}));
  $('strain-row').addEventListener('change', () => setControls({rowSelection: Number($('strain-row').value)}));
  $('strain-slice').addEventListener('change', () => setControls({slice: Number($('strain-slice').value)}));
  $('strain-show-force').addEventListener('change', () => setControls({showForce: $('strain-show-force').checked}));
  $('strain-reset').addEventListener('click', () => setControls({preset: state.preset, mu: 1}));
  $('strain-focus-corner').addEventListener('click', () => setControls({preset: 'corner', cornerSelection: defaultCorner.id}));
  const source = packet.source;
  $('strain-provenance').textContent = `Native capture ${source.commit}. Binary SHA-256 ${source.binarySha256}; records SHA-256 ${source.recordsSha256}; independent Fraction summary SHA-256 ${source.oracleSummarySha256}. The book publisher validates these source bindings before rendering.`;
  selectRow(preferredRow(state.preset)); render();
  window.alignedStrainLab = {
    packet,
    field: () => field().slice(),
    snapshot: () => ({...current, field: current.field.slice(), strains: current.strains.slice(), action: current.action.slice(), forces: current.forces.slice(), losses: current.losses.slice(), selectedRow: state.rowSelection, selectedRowData: rows[state.rowSelection], controls: {...state}}),
    setControls
  };
}
try { start(); } catch (error) { $('strain-status').textContent = `Qualified specimen unavailable: ${error.message}`; console.error(error); }
