import {readRecording, mountKenoma, KENOMA_COMMIT} from './adapters.js';
import {RecordedView} from './view.js';
import {readRecordingFolder} from './catalog.js';
const fmt = value => Array.isArray(value) ? value.map(fmt).join(', ') : Number(value).toPrecision(6);
const node = (tag, text) => { const el = document.createElement(tag); el.textContent = text; return el; };

export class RheonViewer extends HTMLElement {
  constructor() {
    super(); this.attachShadow({mode: 'open'}); this.mode = 'records'; this.index = 0; this.caseIndex = 0;
    this.shadowRoot.innerHTML = `<link rel="stylesheet" href="${new URL('./shell.css', import.meta.url).href}">
      <section class="workspace" aria-label="Rheon viewer workspace">
      <div class="toolbar"><div class="mode" aria-label="Workspace mode"><button id="records" aria-pressed="true">Recorded runs</button><button id="pose" aria-pressed="false">Human pose</button></div>
      <label id="import-label" class="file-button">Open native JSON<input id="file" type="file" accept=".json,application/json"></label>
      <label id="folder-label" class="file-button">Open output folder<input id="folder" type="file" webkitdirectory multiple aria-label="Open output folder"></label>
      <label id="runs-label" hidden>Recording <select id="runs" aria-label="Folder recording"></select></label>
      <label id="cases-label" hidden>Case <select id="cases" aria-label="Recorded case"></select></label><button id="fit">Frame view</button></div>
      <p id="scope" class="scope"></p><details id="catalog-panel" hidden><summary id="catalog-summary">Folder files</summary><ul id="catalog-items"></ul></details><div class="layout"><div><div id="record-host" class="viewport"><div id="empty" class="empty"><strong>Your data, in view.</strong><span>Open a saved Rheon JSON output or output folder.<br>Or switch to Human pose to arrange simple characters.</span></div></div>
      <div id="pose-host" class="viewport" hidden></div></div><aside class="sidebar"><div><h2 id="title">Recorded mechanics</h2><dl id="metrics"></dl></div><div id="profile-panel" hidden><svg id="profile" viewBox="0 0 230 210" role="img" aria-label="Recorded tangential speed in metres per second against height in metres"></svg></div><p id="provenance"></p></aside></div>
      <div id="transport" class="transport"><button id="previous" aria-label="Previous recorded frame" disabled>←</button><button id="play" disabled>Play records</button><button id="next" aria-label="Next recorded frame" disabled>→</button>
      <input id="timeline" type="range" min="0" max="0" value="0" aria-label="Recorded frame" disabled><output id="time">No recording loaded</output></div>
      <div id="status" class="status" role="status" aria-live="polite">Ready to open a recording.</div>
      <details><summary>Embedding & computation boundaries</summary><p>Playback selects stored frames without interpolation or stepping a solver. Posing edits Kenoma's independent kinematic graph. There is no muscle, contact or fluid coupling between the two views.</p><p>Embed this page in an iframe, or import <code>shell.js</code> and insert <code>&lt;rheon-viewer&gt;</code>. Serve the complete packaged folder over HTTP(S), including its local assets.</p></details></section>`;
  }
  $(id) { return this.shadowRoot.getElementById(id); }
  connectedCallback() {
    if (this.active) return; this.active = true;
    this.$('records').onclick = () => this.selectMode('records'); this.$('pose').onclick = () => this.selectMode('pose');
    this.$('file').onchange = async e => {
      const input = e.target, file = input.files[0]; if (!file) return;
      const request = this.loadRequest = (this.loadRequest || 0) + 1; this.pause();
      this.status('Opening ' + file.name);
      try { const data = await readRecording(file); if (request === this.loadRequest && this.active) { this.setCatalog(null); this.load(data); } }
      catch (error) { if (request === this.loadRequest && this.active) this.status(error.message, true); }
      input.value = '';
    };
    this.$('folder').onchange = async e => {
      const input=e.target; if (!input.files.length) return;
      const request=this.loadRequest=(this.loadRequest || 0)+1; this.pause();
      const isCurrent=()=>request===this.loadRequest && this.active;
      this.status('Reading output folder…');
      try {
        const catalog=await readRecordingFolder(input.files,{isCurrent});
        if (isCurrent()) {
          this.setCatalog(catalog);
          const first=catalog.entries.findIndex(entry=>entry.status==='ready');
          if (first>=0) { this.$('runs').value=String(first); this.load(catalog.entries[first].data); }
          else this.status('No supported recordings in this folder. The current recording is retained; expand Folder files for details.',true);
        }
      } catch(error) { if (isCurrent()) this.status(error.message,true); }
      input.value='';
    };
    this.$('runs').onchange = e => {
      const entry=this.catalog?.entries[Number(e.target.value)];
      if (entry?.status==='ready') this.load(entry.data);
    };
    this.$('cases').onchange = e => { this.pause(); this.caseIndex = Number(e.target.value); this.index = 0; this.draw(true); };
    this.$('timeline').oninput = e => { this.pause(); this.index = Number(e.target.value); this.draw(); };
    this.$('previous').onclick = () => this.seek(this.index - 1); this.$('next').onclick = () => this.seek(this.index + 1);
    this.$('play').onclick = () => this.timer ? this.pause() : this.play(); this.$('fit').onclick = () => this.view?.fit();
    this.selectMode('records');
  }
  disconnectedCallback() { this.active = false; this.pause(); this.loadRequest = (this.loadRequest || 0) + 1; this.poseRequest = (this.poseRequest || 0) + 1; this.view?.dispose(); this.view = null; this.unmountPose?.(); this.unmountPose = null; }
  status(message, error = false) { this.$('status').textContent = message; this.$('status').classList.toggle('error', error); }
  setCatalog(catalog) {
    this.catalog=catalog;
    this.$('runs').replaceChildren(); this.$('catalog-items').replaceChildren();
    if (catalog) {
      const ready=catalog.entries.filter(entry=>entry.status==='ready').length;
      this.$('catalog-summary').textContent=`Folder files · ${ready} recordings / ${catalog.entries.length} files`;
      catalog.entries.forEach((entry,i)=>{
        const label=`${entry.path} · ${entry.status==='ready'?'Ready':entry.status==='unsupported'?'Unsupported':'Refused'} · ${entry.message}`;
        this.$('catalog-items').append(node('li',label));
        if (entry.status==='ready') { const option=node('option',entry.path); option.value=String(i); this.$('runs').append(option); }
      });
    }
    this.$('runs-label').hidden=this.mode==='pose' || !catalog?.entries.some(entry=>entry.status==='ready');
    this.$('catalog-panel').hidden=this.mode==='pose' || !catalog;
  }
  load(data) {
    this.pause(); this.data = data; this.index = 0; this.caseIndex = 0;
    this.$('cases').replaceChildren(...data.cases.map((c, i) => { const option = node('option', c.label); option.value = i; return option; }));
    this.status(`${data.provenance.name} · SHA256 ${data.provenance.sha256} · Imported for visualization, not solver qualification.`);
    this.selectMode('records'); this.draw(true);
  }
  frames() { return this.data?.cases[this.caseIndex].frames; }
  seek(index) { this.pause(); if (!this.frames()) return; this.index = Math.max(0, Math.min(index, this.frames().length - 1)); this.draw(); }
  pause() { clearInterval(this.timer); this.timer = null; this.$('play').textContent = 'Play records'; }
  play() {
    if (!this.frames() || this.mode !== 'records') return;
    if (this.index === this.frames().length - 1) this.index = 0;
    this.$('play').textContent = 'Pause'; this.draw();
    // UI cadence only. Stored physical time is displayed separately.
    this.timer = setInterval(() => { if (this.index < this.frames().length - 1) { this.index++; this.draw(); } else this.pause(); }, 500);
  }
  selectMode(mode) {
    this.pause(); this.mode = mode; const pose = mode === 'pose';
    if (pose) this.loadRequest=(this.loadRequest || 0)+1;
    this.$('pose').setAttribute('aria-pressed', String(pose)); this.$('records').setAttribute('aria-pressed', String(!pose));
    this.$('record-host').hidden = pose; this.$('pose-host').hidden = !pose;
    for (const id of ['import-label', 'folder-label', 'transport', 'fit']) this.$(id).hidden = pose;
    this.$('runs-label').hidden=pose || !this.catalog?.entries.some(entry=>entry.status==='ready');
    this.$('catalog-panel').hidden=pose || !this.catalog;
    this.$('cases-label').hidden = pose || !this.data;
    this.$('scope').textContent = pose ? 'Kinematic pose editor · Arrange characters and IK targets. No biomechanical dynamics, forces or simulation run here.' : 'Recorded playback · Select saved native frames. The viewer does not advance a simulation or regenerate results.';
    if (pose) {
      this.$('title').textContent = 'Simple-human component'; this.$('metrics').replaceChildren(); this.$('profile-panel').hidden = true;
      this.$('provenance').textContent = `Kenoma ${KENOMA_COMMIT} · rig version 1. Worker-backed posing preserves bound vertex IDs and triangle indices. Contact may interpenetrate; extreme bends can crease. The latest mesh may briefly lag the handles. No collision physics, anatomical or muscle model. Save/Open local .human.sqlite files to keep source scenes across reloads. Imports replace the scene in one undoable edit; there is no autosave.`;
      if (!this.unmountPose) {
        this.status('Loading the pinned Kenoma component…');
        const request = this.poseRequest = (this.poseRequest || 0) + 1;
        fetch(new URL('./component.json', import.meta.url)).then(r => { if (!r.ok) throw Error('Kenoma component is not packaged'); return r.json(); }).then(config => {
          if (!this.active || request !== this.poseRequest) return;
          this.unmountPose = mountKenoma(this.$('pose-host'), config);
          if (this.mode === 'pose') this.status('Kenoma pose editor · Versioned kinematic component, independent from recorded physics.');
        }).catch(error => { if (this.active && request === this.poseRequest && this.mode === 'pose') this.status(error.message, true); });
      } else this.status('Kenoma pose editor · In-memory scene retained.');
    } else if (this.data) { this.draw(); this.view?.resize(); }
    else { this.$('title').textContent = 'Recorded mechanics'; this.$('provenance').textContent = 'Supported: rigid_motion JSON, obstacle-flow shear records and native fixed-slab results.json. Data stays in your browser.'; this.status('Ready to open a recording.'); }
  }
  draw(fit = false) {
    const frames = this.frames(); if (!frames) return;
    const frame = frames[this.index];
    this.$('timeline').max = frames.length - 1; this.$('timeline').value = this.index;
    this.$('timeline').disabled = false; this.$('play').disabled = frames.length < 2;
    this.$('previous').disabled = this.index === 0; this.$('next').disabled = this.index === frames.length - 1;
    this.$('time').textContent = `${fmt(frame.time)} s · ${this.index + 1}/${frames.length}`;
    this.$('title').textContent = this.data.label; this.$('empty').hidden = true;
    this.$('metrics').replaceChildren(...frame.metrics.flatMap(([key, value]) => [node('dt', key), node('dd', fmt(value))]));
    this.$('provenance').textContent = this.data.kind === 'rigid' ? 'World-space recorded vertices (m). Orange arrow: preceding update force at its recorded starting center; normalized display length, not force magnitude. Initial loads are not reported.' : 'Recorded tangential samples at supplied centers. Arrows normalize within the selected frame; the plot retains numerical speeds. Shear elapsed time is recorded step × reported dt.';
    this.$('profile-panel').hidden = !frame.profile; if (frame.profile) this.profile(frame.profile);
    try { const first = !this.view; this.view ||= new RecordedView(this.$('record-host')); this.view.show(frame, fit || first); }
    catch (error) { this.status('3D view unavailable: ' + error.message + '. Recorded values remain available.', true); }
    this.dispatchEvent(new CustomEvent('rheon-frame', {detail: {caseIndex: this.caseIndex, index: this.index, time: frame.time}, bubbles: true, composed: true}));
  }
  profile(samples) {
    const low = Math.min(...samples.map(p => p[0])), high = Math.max(...samples.map(p => p[0]));
    const max = Math.max(1e-12, ...samples.map(p => Math.abs(p[1]))), span = Math.max(high - low, 1e-12);
    const svg = this.$('profile'); svg.replaceChildren();
    const add = (tag, attributes, text) => { const el = document.createElementNS('http://www.w3.org/2000/svg', tag); for (const [k, v] of Object.entries(attributes)) el.setAttribute(k, v); if (text) el.textContent = text; svg.append(el); };
    add('path', {d: 'M25 15 V175 H220', fill: 'none', stroke: '#69818b'});
    add('polyline', {points: samples.map(([y, v]) => `${122 + 90 * v / max},${175 - 155 * (y - low) / span}`).join(' '), fill: 'none', stroke: '#1595a5', 'stroke-width': 3});
    add('text', {x: 25, y: 204, 'font-size': 10}, `speed ±${fmt(max)} m/s`); add('text', {x: 25, y: 12, 'font-size': 10}, `height ${fmt(low)}–${fmt(high)} m`);
  }
}
customElements.define('rheon-viewer', RheonViewer);
