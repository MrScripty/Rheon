import * as THREE from 'three';
import {OrbitControls} from 'three/addons/controls/OrbitControls.js';

const $ = id => document.getElementById('compressible-' + id);
const response = await fetch('compressible-records.json');
if (!response.ok) throw new Error(`Compressible records request failed: ${response.status}`);
const packet = await response.json();
if (packet.schema_version !== 1 || packet.cases.length !== 2) throw new Error('Invalid compressible fixture schema');
const host = $('scene');
const number = value => Number(value).toPrecision(6);
let renderer, controls, mesh, geometry, material;
let renderMode = 'webgl', playing = null, current, fields = [], indices = [], positions = [];
let suspended = false, disposed = false, renderedFrames = 0;
const scene = new THREE.Scene();
scene.background = new THREE.Color(0xedf3f2);
const camera = new THREE.PerspectiveCamera(42, 1, .01, 100);
const group = new THREE.Group();
scene.add(group);
try {
  renderer = new THREE.WebGLRenderer({antialias:true});
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.domElement.setAttribute('aria-hidden', 'true');
  host.append(renderer.domElement);
  controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
} catch {
  renderMode = 'slice-fallback';
  host.innerHTML = '<svg id="compressible-fallback" viewBox="0 0 400 400" role="img" aria-label="Saved scalar field on the selected cell-center slice"></svg>';
  $('mode').value = 'slice';
  $('mode').disabled = true;
  $('reset').disabled = true;
}
const colorStops = ['#2348a1', '#24abb7', '#edc459', '#d44435'].map(c => new THREE.Color(c));
function color(value, range) {
  const fraction = range[1] === range[0] ? .5 : Math.max(0, Math.min(1, (value - range[0]) / (range[1] - range[0])));
  const scaled = fraction * (colorStops.length - 1), low = Math.min(colorStops.length - 2, Math.floor(scaled));
  return colorStops[low].clone().lerp(colorStops[low + 1], scaled - low);
}
function coordinates(index) {
  const [, ny, nz] = current.grid.shape;
  return [Math.floor(index / (ny * nz)), Math.floor(index / nz) % ny, index % nz];
}
function center(index) {
  return coordinates(index).map((q, d) => {
    const [lo, hi] = current.grid.extent[d];
    return lo + (q + .5) * (hi - lo) / current.grid.shape[d];
  });
}
function selectedFrame() { return current.frames[Number($('frame').value)]; }
function clearGroup() {
  for (const child of [...group.children]) {
    group.remove(child);
    child.geometry?.dispose();
    child.material?.dispose();
  }
  mesh = geometry = material = null;
}
function resetCamera() {
  if (controls) {controls.enableDamping = false; controls.update(); controls.enableDamping = true;}
  const mid = current.grid.extent.map(e => (e[0] + e[1]) / 2);
  const span = Math.max(...current.grid.extent.map(e => e[1] - e[0]));
  camera.position.set(...mid.map((v, d) => v + [1.5, 1.1, 1.7][d] * span / Math.min(1, camera.aspect)));
  camera.near = span / 1000; camera.far = span * 100;
  camera.updateProjectionMatrix();
  controls?.target.set(...mid); controls?.update();
}
function drawFallback(range) {
  const svg = $('fallback'), axis = Number($('axis').value), tangents = [0,1,2].filter(d => d !== axis);
  svg.replaceChildren();
  for (let i = 0; i < indices.length; i++) {
    const q = coordinates(indices[i]), rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    const width = 360 / current.grid.shape[tangents[0]], height = 360 / current.grid.shape[tangents[1]];
    rect.setAttribute('x', 20 + q[tangents[0]] * width);
    rect.setAttribute('y', 380 - (q[tangents[1]] + 1) * height);
    rect.setAttribute('width', width); rect.setAttribute('height', height);
    rect.setAttribute('fill', '#' + color(fields[i], range).getHexString());
    rect.setAttribute('data-cell', indices[i]); rect.setAttribute('data-value', fields[i]);
    const title = document.createElementNS('http://www.w3.org/2000/svg', 'title');
    title.textContent = `Cell ${q.join(', ')}: ${number(fields[i])}`; rect.append(title); svg.append(rect);
  }
}
function drawVoxels(range) {
  clearGroup();
  const lengths = current.grid.extent.map((e, d) => (e[1] - e[0]) / current.grid.shape[d]);
  geometry = new THREE.BoxGeometry(...lengths.map(v => v * .94));
  const volume = $('mode').value === 'volume';
  material = new THREE.MeshBasicMaterial({transparent:volume, opacity:volume ? .32 : 1, depthWrite:!volume});
  mesh = new THREE.InstancedMesh(geometry, material, indices.length);
  const transform = new THREE.Matrix4();
  indices.forEach((index, i) => {
    transform.makeTranslation(...positions[i]); mesh.setMatrixAt(i, transform); mesh.setColorAt(i, color(fields[i], range));
  });
  mesh.instanceMatrix.needsUpdate = true; mesh.instanceColor.needsUpdate = true;
  group.add(mesh);
  const outlineGeometry = new THREE.BoxGeometry(...current.grid.extent.map(e => e[1] - e[0]));
  const edges = new THREE.EdgesGeometry(outlineGeometry); outlineGeometry.dispose();
  const outline = new THREE.LineSegments(edges, new THREE.LineBasicMaterial({color:0x647d84}));
  outline.position.set(...current.grid.extent.map(e => (e[0] + e[1]) / 2)); group.add(outline);
  const axes = new THREE.AxesHelper(.28); axes.position.set(...current.grid.extent.map(e => e[0])); group.add(axes);
  const probeGeometry = new THREE.BoxGeometry(...lengths);
  const probeEdges = new THREE.EdgesGeometry(probeGeometry); probeGeometry.dispose();
  const probe = new THREE.LineSegments(probeEdges, new THREE.LineBasicMaterial({color:0x192d32}));
  probe.position.set(...center(Number($('cell').value))); group.add(probe);
}
function metric(id, value) {
  $(id).textContent = number(value); $(id).dataset.value = String(value);
}
function refresh() {
  const frame = selectedFrame(), scalar = $('scalar').value, axis = Number($('axis').value), slice = Number($('slice').value);
  const source = scalar === 'speed' ? frame.velocity.map(v => Math.hypot(...v)) : frame[scalar];
  indices = source.map((_, i) => i).filter(i => $('mode').value === 'volume' || coordinates(i)[axis] === slice);
  fields = indices.map(i => source[i]); positions = indices.map(center);
  const range = current.field_ranges[scalar];
  if (renderer) drawVoxels(range); else drawFallback(range);
  $('scene').dataset.case = current.id; $('scene').dataset.frame = $('frame').value;
  $('scene').dataset.scalar = scalar; $('scene').dataset.axis = axis; $('scene').dataset.slice = slice;
  $('scene').dataset.cells = indices.length;
  $('time').textContent = number(frame.time); $('slice-value').textContent = `${slice} / ${current.grid.shape[axis] - 1}`;
  $('low').textContent = number(range[0]); $('high').textContent = number(range[1]);
  $('title').textContent = current.title;
  $('grid').textContent = `${current.grid.shape.join(' × ')} cell centers · periodic cube · ${frame.diagnostics.steps} saved update steps`;
  $('caption').textContent = `${scalar === 'speed' ? 'Speed' : scalar[0].toUpperCase() + scalar.slice(1)} · case-wide color range · ${indices.length} displayed cells. ${$('mode').value === 'volume' ? 'Transparent boxes show the full field; use a slice for unoccluded values.' : `Slice normal ${'xyz'[axis]} at cell ${slice}.`} Axes: red x, green y, blue z.`;
  const diagnostics = frame.diagnostics;
  for (const key of ['mass', 'energy', 'min_density', 'min_pressure', 'relative_conservation_drift']) metric(key, diagnostics[key]);
  diagnostics.momentum.forEach((value, d) => metric('momentum-' + 'xyz'[d], value));
  const cell = Number($('cell').value), q = coordinates(cell), rho = frame.density[cell], velocity = frame.velocity[cell];
  $('cell-value').textContent = q.join(', ');
  const conservative = frame.conserved[cell];
  const probe = [['Cell (i,j,k)', q.join(', ')], ['Center (x,y,z)', center(cell).map(number).join(', ')], ['Density ρ', number(rho)], ['Pressure p', number(frame.pressure[cell])], ['Velocity (uₓ,uᵧ,u_z)', velocity.map(number).join(', ')], ['Conserved (ρ,mₓ,mᵧ,m_z,E)', conservative.map(number).join(', ')]];
  $('probe').replaceChildren();
  for (const [name, value] of probe) { const dt = document.createElement('dt'), dd = document.createElement('dd'); dt.textContent = name; dd.textContent = value; $('probe').append(dt, dd); }
  $('probe').dataset.cell = cell;
  $('error').textContent = `Density L¹ reference difference: ${number(diagnostics.density_l1_reference_error)}${current.id === 'planar_sod_periodic' ? ' · central shock-tube window' : ' · linear acoustic reference'}.`;
  $('error').dataset.value = diagnostics.density_l1_reference_error;
  $('qualification').textContent = `Numerical checks: ${packet.qualification.status}. Compare the finite-grid shock profile and acoustic phase, damping and refinement in the downloadable qualification. The acoustic linear comparison includes small nonlinear effects.`;
  updateStatus();
}
function updateStatus() {
  $('status').textContent = `${renderMode === 'webgl' ? '3D saved field' : 'WebGL unavailable: displaying the authoritative 2D saved slice'}. Frame ${Number($('frame').value) + 1} of ${current.frames.length}, t = ${number(selectedFrame().time)}. ${playing ? 'Playing recorded frames.' : 'No fluid computation runs in the browser.'}`;
}
function stop() {
  if (playing !== null) clearInterval(playing);
  playing = null; $('play').textContent = 'Play saved frames'; $('play').setAttribute('aria-pressed', 'false');
}
function choose() {
  stop(); current = packet.cases.find(c => c.id === $('case').value);
  $('frame').max = current.frames.length - 1; $('frame').value = 0;
  $('slice').max = current.grid.shape[Number($('axis').value)] - 1; $('slice').value = Math.floor(Number($('slice').max) / 2);
  $('cell').max = current.grid.shape.reduce((a,b) => a*b, 1) - 1;
  $('cell').value = Math.floor(Number($('cell').max) / 2);
  refresh(); if (renderer) resetCamera();
}
for (const c of packet.cases) $('case').add(new Option(c.title, c.id));
$('case').addEventListener('change', choose);
$('frame').addEventListener('input', () => {stop(); refresh();});
for (const id of ['scalar', 'mode']) $(id).addEventListener('change', refresh);
$('axis').addEventListener('change', () => { $('slice').max = current.grid.shape[Number($('axis').value)] - 1; $('slice').value = Math.min(Number($('slice').value), Number($('slice').max)); refresh(); });
for (const id of ['slice', 'cell']) $(id).addEventListener('input', refresh);
$('reset').addEventListener('click', resetCamera);
$('play').addEventListener('click', () => {
  if (playing !== null) {stop(); refresh(); return;}
  if (Number($('frame').value) === current.frames.length - 1) $('frame').value = 0;
  playing = setInterval(() => {
    $('frame').value = Number($('frame').value) + 1;
    if (Number($('frame').value) === current.frames.length - 1) stop();
    refresh();
  }, 600);
  $('play').textContent = 'Pause playback'; $('play').setAttribute('aria-pressed', 'true'); refresh();
});
host.addEventListener('keydown', event => {
  if (!controls || !['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', '+', '=', '-', 'Escape'].includes(event.key)) return;
  event.preventDefault();
  if (event.key === 'Escape') {host.blur(); return;}
  const offset = camera.position.clone().sub(controls.target), spherical = new THREE.Spherical().setFromVector3(offset);
  if (event.key === 'ArrowLeft') spherical.theta -= .12;
  if (event.key === 'ArrowRight') spherical.theta += .12;
  if (event.key === 'ArrowUp') spherical.phi = Math.max(.05, spherical.phi - .12);
  if (event.key === 'ArrowDown') spherical.phi = Math.min(Math.PI - .05, spherical.phi + .12);
  if (event.key === '+' || event.key === '=') spherical.radius *= .9;
  if (event.key === '-') spherical.radius *= 1.1;
  camera.position.copy(controls.target).add(offset.setFromSpherical(spherical)); controls.update();
});
function resizeScene() {
  if (!renderer || suspended || disposed) return;
  const width = host.clientWidth, height = host.clientHeight;
  if (!width || !height) return;
  renderer.setSize(width, height); camera.aspect = width / height; camera.updateProjectionMatrix();
}
const observer = new ResizeObserver(resizeScene);
observer.observe(host);
$('provenance').textContent = `Generator source SHA-256: ${packet.provenance.source_sha256['generate.py']}. Stored frame fields are flattened in C order, z fastest.`;
choose();
function renderFrame() {
  if (suspended || disposed) return;
  controls.update(); renderer.render(scene, camera); renderedFrames += 1;
}
if (renderer) renderer.setAnimationLoop(renderFrame);
window.__compressible = {
  snapshot() {
    const renderedColors = renderer ? indices.map((_, i) => {const c = new THREE.Color(); mesh.getColorAt(i, c); return c.toArray();}) : [];
    const renderedPositions = renderer ? indices.map((_, i) => {const m = new THREE.Matrix4(); mesh.getMatrixAt(i, m); return [m.elements[12], m.elements[13], m.elements[14]];}) : positions;
    return {caseId:current.id, frame:Number($('frame').value), scalar:$('scalar').value, mode:$('mode').value, axis:Number($('axis').value), slice:Number($('slice').value), indices:indices.slice(), values:fields.slice(), renderedPositions, renderedColors, renderMode, camera:camera.position.toArray(), cameraAspect:camera.aspect, viewport:[host.clientWidth,host.clientHeight], playing:playing !== null, cell:Number($('cell').value), renderedFrames, suspended};
  }
};
window.addEventListener('pagehide', event => {
  stop(); suspended = true; renderer?.setAnimationLoop(null);
  // A persisted page keeps its resources for the browser's history cache.
  if (event.persisted) return;
  disposed = true; observer.disconnect(); controls?.dispose(); clearGroup(); renderer?.dispose();
});
window.addEventListener('pageshow', event => {
  if (!event.persisted || disposed) return;
  suspended = false; resizeScene(); updateStatus(); renderer?.setAnimationLoop(renderFrame);
});
