// Execute the delivered inline JavaScript with select-option semantics and
// recorded canvas calls. This checks controls/data, not a browser screenshot.
const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const htmlPath = process.argv[2], nativePath = process.argv[3];
const html = fs.readFileSync(htmlPath, 'utf8');
const native = fs.readFileSync(nativePath, 'utf8').trim().split('\n').map(JSON.parse);
const script = html.match(/<script>([\s\S]*)<\/script>/)[1];
let polygons = [], polygon, tick;
const ctx = {
  clearRect() { polygons = []; },
  beginPath() { polygon = { points: [] }; },
  moveTo(...p) { polygon.points.push(p); }, lineTo(...p) { polygon.points.push(p); },
  closePath() {},
  fill() { polygon.color = this.fillStyle; },
  stroke() { polygons.push(polygon); }, fillText() {},
};
class Select {
  constructor() { this.options = []; this.selected = ''; }
  add(option) { this.options.push(option); if (this.options.length === 1) this.selected = option.value; }
  set value(value) { this.selected = this.options.some(o => o.value === String(value)) ? String(value) : ''; }
  get value() { return this.selected; }
}
const elements = { case: new Select(), h: new Select(), frame: { value: 0 },
  info: {}, play: { textContent: 'Play' }, view: { getContext: () => ctx } };
const sandbox = {
  document: { getElementById: id => elements[id] },
  Option: function(text, value) { this.text = String(text); this.value = String(value); },
  setInterval(callback) { tick = callback; return 1; },
  clearInterval() { tick = undefined; },
};
vm.createContext(sandbox);
vm.runInContext(script, sandbox, { timeout: 10000 });
assert.equal(elements.h.value, '0.003125');
assert.equal(elements.frame.max, 32);
assert.deepEqual(JSON.parse(vm.runInContext('JSON.stringify(rows)', sandbox)), native);
function expected(row) {
  const energy = row.mass.reduce((a, m, i) => a + .5 * m * row.velocity[i].reduce((b,u) => b+u*u,0),0);
  const momentum = row.mass.reduce((a,m,i) => a + m * row.velocity[i][2],0);
  assert.equal(elements.info.textContent, `Time ${row.time.toFixed(5)} · actual stamp ${row.stamp.id}:${row.stamp.version} · published total energy ${energy.toPrecision(8)} · third momentum ${momentum.toPrecision(8)} · ${row.load} body force · signed force work ${row.forcing?row.forcing.total_work.toPrecision(8):'initial state'} · work ledger ${row.report?row.report.total.energy_ledger_error.toExponential(3):'initial state'}`);
  assert.equal(polygons.length, 24);
  row.triangles.forEach((tri, i) => {
    assert.deepEqual(polygons[i].points, tri.map(n => [70+720*row.positions[n][0],485-340*row.positions[n][1]]));
    const w = tri.reduce((s,n) => s + row.velocity[row.periodic_indices[n]][2]/3,0);
    const t = Math.max(0,Math.min(1,(w+.25)/.875));
    assert.equal(polygons[i].color, `rgb(${Math.round(42+195*t)},${Math.round(112-34*t)},${Math.round(195-130*t)})`);
  });
}
let publications = 0, cases = 0;
for (const option of elements.case.options) {
  elements.case.value = option.value;
  for (const interval of elements.h.options) {
    elements.h.value = interval.value; elements.h.onchange();
    const [kind, field, load] = option.value.split(' / ');
    const rows = native.filter(r => r.kind===kind && r.field===field && r.load===load && r.h===Number(interval.value));
    assert.equal(elements.frame.max, rows.length-1);
    rows.forEach((row,i) => { elements.frame.value=i; elements.frame.oninput(); expected(row); publications++; });
    cases++;
  }
}
elements.frame.value = elements.frame.max;
elements.play.onclick(); assert.equal(elements.play.textContent, 'Pause');
assert.equal(typeof tick, 'function'); tick(); assert.equal(Number(elements.frame.value), 0);
elements.play.onclick(); assert.equal(elements.play.textContent, 'Play'); assert.equal(tick, undefined);
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
console.log(JSON.stringify({ status:'PASS', scope:'Inline JavaScript controls and exact canvas/data commands in Node VM; browser rasterization not tested', publications, cases, default_interval:'0.003125', play_wrap_pause:true, html_sha256:sha(fs.readFileSync(htmlPath)), native_sha256:sha(fs.readFileSync(nativePath)), checker_sha256:sha(fs.readFileSync(__filename)) },null,2));
