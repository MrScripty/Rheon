import * as THREE from 'three';
import {buildLightDepth, buildInstantaneousStreamlines, FLOW_LIGHT_EXTINCTION,
  FLOW_LIGHT_AMBIENT, FLOW_STREAMLINE_SEEDS, FLOW_STREAMLINE_STEP,
  FLOW_STREAMLINE_MAX_LENGTH, FLOW_STREAMLINE_MIN_SPEED, FLOW_STREAMLINE_SPEED_MAX} from './flow_field.js';

// Display parameters, not solver parameters. Concentration is never normalized.
export const FLOW_EXTINCTION = 32;
export const FLOW_MAX_SAMPLES = 512;

function nonnegative(value, name) {
  if (!Number.isFinite(value) || value < 0) throw new Error(`${name} must be finite and nonnegative.`);
  return value;
}

export function beerLambertOpacity(concentration, distance, extinction = FLOW_EXTINCTION) {
  return -Math.expm1(-nonnegative(concentration, 'Concentration') *
    nonnegative(distance, 'Distance') * nonnegative(extinction, 'Extinction'));
}

function gridSpec(grid) {
  for (const key of ['counts', 'spacing', 'origin']) {
    if (!grid?.[key] || grid[key].length !== 3) throw new Error(`Grid ${key} must have three components.`);
  }
  const counts = Array.from(grid.counts), spacing = Array.from(grid.spacing), origin = Array.from(grid.origin);
  if (counts.some(n => !Number.isSafeInteger(n) || n < 1) ||
      spacing.some(h => !Number.isFinite(h) || h <= 0) || origin.some(x => !Number.isFinite(x))) {
    throw new Error('Invalid volume grid.');
  }
  const length = counts.reduce((a, b) => a * b, 1);
  const extent = counts.map((n, i) => n * spacing[i]);
  if (!Number.isSafeInteger(length) || extent.some((x, i) => !Number.isFinite(x) || !Number.isFinite(x + origin[i]))) {
    throw new Error('Volume grid is too large.');
  }
  return {counts, spacing, origin, length, extent};
}

function sourceSpec(source) {
  if (source == null) return null;
  if (source.lower?.length !== 3 || source.upper?.length !== 3) throw new Error('Source bounds must have three components.');
  const lower = Array.from(source.lower), upper = Array.from(source.upper);
  if (lower.some((x, i) => !Number.isFinite(x) || !Number.isFinite(upper[i]) || upper[i] <= x)) {
    throw new Error('Invalid source bounds.');
  }
  return {lower, upper};
}

// Independent callers can check the same cell-center/clamped-edge convention.
// Storage is x-fastest: i + nx * (j + ny * k). Position uses physical grid units.
export function sampleTracerTrilinear(state, position) {
  const {counts, spacing, origin, length} = gridSpec(state.grid);
  if (state.tracer?.length !== length || position?.length !== 3 || Array.from(position).some(x => !Number.isFinite(x))) {
    throw new Error('Invalid tracer sample.');
  }
  // Clamp physical positions before subtraction/division so even a very distant
  // finite query cannot overflow the dimensionless cell coordinate.
  const coordinate = Array.from(position, (x, d) =>
    (Math.max(origin[d], Math.min(origin[d] + counts[d] * spacing[d], x)) - origin[d]) / spacing[d] - .5);
  const lower = coordinate.map(Math.floor), fraction = coordinate.map((x, d) => x - lower[d]);
  let value = 0;
  for (let k = 0; k < 2; k++) for (let j = 0; j < 2; j++) for (let i = 0; i < 2; i++) {
    const corner = [i, j, k];
    const index = corner.map((offset, d) => Math.max(0, Math.min(counts[d] - 1, lower[d] + offset)));
    const weight = corner.reduce((w, offset, d) => w * (offset ? fraction[d] : 1 - fraction[d]), 1);
    value += nonnegative(state.tracer[index[0] + counts[0] * (index[1] + counts[1] * index[2])], 'Tracer') * weight;
  }
  return value;
}

const vertexShader = `
out vec3 localPosition;
void main() {
  localPosition = position;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}`;

const fragmentShader = `
precision highp sampler3D;
uniform sampler3D tracer;
uniform sampler3D lightDepth;
uniform ivec3 counts;
uniform vec3 extent;
uniform vec3 cameraLocal;
uniform vec3 tint;
uniform float extinction;
uniform float nominalStep;
uniform bool lit;
uniform float lightAmbient;
in vec3 localPosition;
out vec4 fragmentColor;

float fieldValue(sampler3D field, vec3 uv) {
  #ifdef MANUAL_TRILINEAR
    vec3 q = uv * vec3(counts) - 0.5;
    ivec3 lo = ivec3(floor(q));
    vec3 w = fract(q);
    ivec3 a = clamp(lo, ivec3(0), counts - 1);
    ivec3 b = clamp(lo + 1, ivec3(0), counts - 1);
    float c00 = mix(texelFetch(field, ivec3(a.x, a.y, a.z), 0).r,
                    texelFetch(field, ivec3(b.x, a.y, a.z), 0).r, w.x);
    float c10 = mix(texelFetch(field, ivec3(a.x, b.y, a.z), 0).r,
                    texelFetch(field, ivec3(b.x, b.y, a.z), 0).r, w.x);
    float c01 = mix(texelFetch(field, ivec3(a.x, a.y, b.z), 0).r,
                    texelFetch(field, ivec3(b.x, a.y, b.z), 0).r, w.x);
    float c11 = mix(texelFetch(field, ivec3(a.x, b.y, b.z), 0).r,
                    texelFetch(field, ivec3(b.x, b.y, b.z), 0).r, w.x);
    return mix(mix(c00, c10, w.y), mix(c01, c11, w.y), w.z);
  #else
    return texture(field, uv).r;
  #endif
}

bool clipAxis(float origin, float direction, inout float entry, inout float exit) {
  if (abs(direction) < 1e-7) return origin >= -0.5 && origin <= 0.5;
  float a = (-0.5 - origin) / direction;
  float b = (0.5 - origin) / direction;
  entry = max(entry, min(a, b));
  exit = min(exit, max(a, b));
  return exit > entry;
}

void main() {
  vec3 direction = normalize(localPosition - cameraLocal);
  float entry = 0.0; // Supports a perspective camera inside the domain too.
  float exit = 1e20;
  if (!clipAxis(cameraLocal.x, direction.x, entry, exit) ||
      !clipAxis(cameraLocal.y, direction.y, entry, exit) ||
      !clipAxis(cameraLocal.z, direction.z, entry, exit)) discard;
  float distance = (exit - entry) * length(direction * extent);
  int samples = int(clamp(ceil(distance / nominalStep), 1.0, float(MAX_SAMPLES)));
  float ds = distance / float(samples);
  vec3 increment = direction * ((exit - entry) / float(samples));
  vec3 p = cameraLocal + direction * entry + increment * 0.5;
  float integral = 0.0;
  vec3 radiance = vec3(0.0);
  float transmittance = 1.0;
  for (int i = 0; i < MAX_SAMPLES; i++) {
    if (i >= samples) break;
    float c = fieldValue(tracer, p + 0.5);
    if (lit && c > 0.0) {
      // Fixed +Y appearance light. Prefix depth is exact at cell centers;
      // trilinear interpolation between those centers is an approximation.
      float light = lightAmbient + (1.0 - lightAmbient) * exp(-fieldValue(lightDepth, p + 0.5));
      float transmission = exp(-extinction * c * ds);
      float weight = transmittance * (1.0 - transmission);
      radiance += weight * tint * light;
      transmittance *= transmission;
    }
    integral += c * ds;
    p += increment;
  }
  // Lighting changes RGB only. The original concentration integral and fixed
  // Beer–Lambert alpha are identical in lit and diagnostic unlit modes.
  float alpha = 1.0 - exp(-extinction * integral);
  vec3 color = lit && alpha > 1e-7 ? radiance / alpha : tint;
  fragmentColor = linearToOutputTexel(vec4(color, alpha));
}`;

function wireBox(color, opacity) {
  const solid = new THREE.BoxGeometry(1, 1, 1);
  const edges = new THREE.EdgesGeometry(solid);
  solid.dispose();
  return new THREE.LineSegments(edges, new THREE.LineBasicMaterial({color, transparent: true, opacity, depthWrite: false}));
}

function lightingMode(value) {
  if (value !== 'lit' && value !== 'unlit') throw new Error('Lighting must be lit or unlit.');
  return value;
}

function scalarTexture(data, counts, interpolation) {
  const texture = new THREE.Data3DTexture(data, ...counts);
  texture.format = THREE.RedFormat;
  texture.type = THREE.FloatType;
  texture.minFilter = texture.magFilter = interpolation === 'hardware-linear' ? THREE.LinearFilter : THREE.NearestFilter;
  texture.wrapS = texture.wrapT = texture.wrapR = THREE.ClampToEdgeWrapping;
  texture.unpackAlignment = 1;
  return texture;
}

// Fixed physical overlays. Lines and direction heads are stationary at a given
// accepted state; color records speed against one fixed 0–1 m/s display scale.
function streamlineGeometry(curves, spec) {
  const positions = [], colors = [];
  const slow = new THREE.Color(0xa96629), fast = new THREE.Color(0xf5ce80);
  const color = speed => slow.clone().lerp(fast, Math.max(0, Math.min(1, speed / FLOW_STREAMLINE_SPEED_MAX)));
  const segment = (a, b, speedA, speedB = speedA) => {
    positions.push(...a, ...b); colors.push(...color(speedA).toArray(), ...color(speedB).toArray());
  };
  const inside = point => point.every((x, d) => x >= spec.origin[d] && x <= spec.origin[d] + spec.extent[d]);
  for (const curve of curves) for (const [name, direction] of [['forward', 1], ['backward', -1]]) {
    const {points, speeds} = curve[name];
    for (let i = 1; i < points.length; i++) segment(points[i - 1], points[i], speeds[i - 1], speeds[i]);
    // Small chevrons at fixed arc-parameter intervals, always pointing with v.
    for (let arc = .125; arc <= curve[name].length; arc += .25) {
      const i = Math.min(points.length - 1, Math.round(arc / FLOW_STREAMLINE_STEP));
      if (i < 1) continue;
      const p = new THREE.Vector3(...points[i]);
      const tangent = new THREE.Vector3(...points[i]).sub(new THREE.Vector3(...points[i - 1])).normalize().multiplyScalar(direction);
      const across = new THREE.Vector3().crossVectors(tangent, Math.abs(tangent.y) < .9 ? new THREE.Vector3(0, 1, 0) : new THREE.Vector3(1, 0, 0)).normalize();
      const rear = p.clone().addScaledVector(tangent, -.018);
      const a = rear.clone().addScaledVector(across, .007).toArray();
      const b = rear.clone().addScaledVector(across, -.007).toArray();
      // A direction glyph is an annotation, but it must not suggest flow through
      // the sealed walls. Omit the whole glyph if either wing leaves the box.
      if (inside(a) && inside(b)) {
        segment(p.toArray(), a, speeds[i]); segment(p.toArray(), b, speeds[i]);
      }
    }
  }
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
  return geometry;
}

/**
 * Full-tensor perspective volume display using RecordedView's scene/lifecycle.
 * Grid/source coordinates are physical. The owned group translates the domain
 * center to zero for the existing lesson camera; it does not alter camera/controls.
 * Tint, extinction and +Y light describe appearance, not optical calibration.
 * Trilinear interpolation does not increase the simulation's spatial resolution.
 */
export class FlowVolume {
  constructor(view, {extinction = FLOW_EXTINCTION, source = null, boxVisible = true,
    sourceVisible = true, forceManualInterpolation = false, lighting = 'lit', streamlines = true} = {}) {
    if (!view?.group?.isObject3D || !view.renderer?.capabilities?.isWebGL2) {
      throw new Error('FlowVolume requires the existing WebGL2 view.');
    }
    this.view = view;
    const context = view.renderer.getContext();
    this.maxTextureSize = context.getParameter(context.MAX_3D_TEXTURE_SIZE);
    this.extinction = nonnegative(extinction, 'Extinction');
    this.source = sourceSpec(source);
    this.sourceVisible = !!sourceVisible;
    this.lighting = lightingMode(lighting);
    this.streamlinesVisible = !!streamlines;
    this.streamlineData = [];
    this.disposed = false;
    this.updates = 0;
    this.spec = null;
    this.texture = null;
    this.lightTexture = null;
    this.interpolation = !forceManualInterpolation && view.renderer.extensions.has('OES_texture_float_linear') ?
      'hardware-linear' : 'manual-trilinear';
    this.group = new THREE.Group();
    this.group.name = 'rheon-flow-volume';
    this.material = new THREE.ShaderMaterial({
      glslVersion: THREE.GLSL3,
      defines: {MAX_SAMPLES: FLOW_MAX_SAMPLES, ...(this.interpolation === 'manual-trilinear' ? {MANUAL_TRILINEAR: 1} : {})},
      uniforms: {
        tracer: {value: null}, lightDepth: {value: null}, counts: {value: [1, 1, 1]}, extent: {value: new THREE.Vector3(1, 1, 1)},
        cameraLocal: {value: new THREE.Vector3()}, tint: {value: new THREE.Color(0x167f88)},
        extinction: {value: this.extinction}, nominalStep: {value: .5},
        lit: {value: this.lighting === 'lit'}, lightAmbient: {value: FLOW_LIGHT_AMBIENT},
      },
      vertexShader, fragmentShader, side: THREE.BackSide, transparent: true, depthWrite: false, toneMapped: false,
    });
    this.mesh = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), this.material);
    this.mesh.name = 'accepted-tracer-volume';
    const inverse = new THREE.Matrix4();
    this.mesh.onBeforeRender = (_renderer, _scene, camera) => {
      this.material.uniforms.cameraLocal.value.setFromMatrixPosition(camera.matrixWorld)
        .applyMatrix4(inverse.copy(this.mesh.matrixWorld).invert());
    };
    this.box = wireBox(0x4f7479, .32);
    this.box.name = 'physical-domain-outline';
    this.box.visible = !!boxVisible;
    this.box.renderOrder = 1;
    this.sourceBox = wireBox(0xc18442, .72);
    this.sourceBox.name = 'physical-source-outline';
    this.sourceBox.renderOrder = 2;
    this.streamlineLines = new THREE.LineSegments(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({vertexColors: true, transparent: true, opacity: .86, depthWrite: false}));
    this.streamlineLines.name = 'instantaneous-velocity-streamlines';
    this.streamlineLines.renderOrder = 3;
    this.group.add(this.mesh, this.box, this.sourceBox, this.streamlineLines);
    this.group.visible = false;
    view.group.add(this.group);
  }

  update(state) {
    if (this.disposed) throw new Error('FlowVolume is disposed.');
    const spec = gridSpec(state?.grid);
    if (spec.counts.some(n => n > this.maxTextureSize)) {
      throw new Error('Volume exceeds the device 3D texture limit.');
    }
    if (state.tracer?.length !== spec.length) throw new Error('Tracer must contain the full grid tensor.');
    let minimum = Infinity, maximum = 0;
    const data = new Float32Array(spec.length);
    for (let i = 0; i < data.length; i++) {
      const value = nonnegative(state.tracer[i], 'Tracer');
      data[i] = value;
      if (!Number.isFinite(data[i])) throw new Error('Tracer exceeds float32 display range.');
      minimum = Math.min(minimum, value); maximum = Math.max(maximum, value);
    }
    const source = Object.hasOwn(state, 'source') ? sourceSpec(state.source) : this.source;
    // Validate and derive all display data before replacing the visible state.
    const lightData = buildLightDepth({...state, tracer: data});
    const hasMac = ['x', 'y', 'z'].some(key => state[key] != null);
    const curves = hasMac ? buildInstantaneousStreamlines(state) : [];
    const curveGeometry = streamlineGeometry(curves, spec);
    const resize = !this.spec || spec.counts.some((n, d) => n !== this.spec.counts[d]);
    if (resize) {
      this.texture?.dispose();
      this.lightTexture?.dispose();
      this.texture = scalarTexture(data, spec.counts, this.interpolation);
      this.lightTexture = scalarTexture(lightData, spec.counts, this.interpolation);
      this.material.uniforms.tracer.value = this.texture;
      this.material.uniforms.lightDepth.value = this.lightTexture;
    } else {
      this.texture.image.data.set(data);
      this.lightTexture.image.data.set(lightData);
    }
    this.texture.needsUpdate = true;
    this.lightTexture.needsUpdate = true;
    this.streamlineLines.geometry.dispose();
    this.streamlineLines.geometry = curveGeometry;
    this.streamlineData = curves;
    this.streamlineLines.visible = this.streamlinesVisible && curves.length > 0;
    this.spec = spec;
    this.sourceTime = Number.isFinite(state.time) ? state.time : null;
    this.sourceGeneration = Number.isSafeInteger(state.generation) ? state.generation : null;
    this.range = [minimum, maximum];
    this.source = source;
    this.material.uniforms.counts.value = spec.counts;
    this.material.uniforms.extent.value.fromArray(spec.extent);
    this.material.uniforms.nominalStep.value = Math.min(...spec.spacing) * .5;
    const center = spec.origin.map((x, d) => x + spec.extent[d] * .5);
    this.group.position.fromArray(center).negate();
    for (const object of [this.mesh, this.box]) {
      object.position.fromArray(center);
      object.scale.fromArray(spec.extent);
    }
    this.sourceBox.visible = this.sourceVisible && source !== null;
    if (source) {
      this.sourceBox.position.fromArray(source.lower.map((x, d) => (x + source.upper[d]) * .5));
      this.sourceBox.scale.fromArray(source.lower.map((x, d) => source.upper[d] - x));
    }
    this.group.visible = true;
    this.updates++;
    return this;
  }

  setDisplay({extinction = this.extinction, boxVisible = this.box.visible, sourceVisible = this.sourceVisible,
    lighting = this.lighting, streamlines = this.streamlinesVisible} = {}) {
    if (this.disposed) throw new Error('FlowVolume is disposed.');
    lightingMode(lighting);
    this.extinction = nonnegative(extinction, 'Extinction');
    this.material.uniforms.extinction.value = this.extinction;
    this.box.visible = !!boxVisible;
    this.sourceVisible = !!sourceVisible;
    this.sourceBox.visible = this.sourceVisible && this.source !== null;
    this.lighting = lightingMode(lighting);
    this.material.uniforms.lit.value = this.lighting === 'lit';
    this.streamlinesVisible = !!streamlines;
    this.streamlineLines.visible = this.streamlinesVisible && this.streamlineData.length > 0;
    return this;
  }

  snapshot({derived = false} = {}) {
    return {grid: this.spec ? {counts: [...this.spec.counts], spacing: [...this.spec.spacing], origin: [...this.spec.origin]} : null,
      tracerLength: this.spec?.length || 0, tracerRange: this.range ? [...this.range] : null,
      extinction: this.extinction, nominalStep: this.material.uniforms.nominalStep.value,
      maxSamples: FLOW_MAX_SAMPLES, interpolation: this.interpolation,
      lighting: this.lighting, lightExtinction: FLOW_LIGHT_EXTINCTION, lightAmbient: FLOW_LIGHT_AMBIENT,
      lightInterpolation: 'cell-center-prefix-trilinear', streamlinesVisible: this.streamlineLines.visible,
      sourceTime: this.sourceTime ?? null, sourceGeneration: this.sourceGeneration ?? null,
      streamlineGeometry: 'frozen-instantaneous-arclength-rk4', streamlinePositionUnits: 'm', streamlineSpeedUnits: 'm/s',
      streamlineDirection: 'heads follow velocity; branches integrate both directions',
      streamlineOverlay: true,
      streamlineSeeds: FLOW_STREAMLINE_SEEDS.map(seed => [...seed]), streamlineStep: FLOW_STREAMLINE_STEP,
      streamlineMaxLength: FLOW_STREAMLINE_MAX_LENGTH, streamlineMinSpeed: FLOW_STREAMLINE_MIN_SPEED,
      streamlineSpeedMax: FLOW_STREAMLINE_SPEED_MAX, streamlineCurveCount: this.streamlineData.length,
      streamlineVertexCount: this.streamlineLines.geometry.getAttribute('position')?.count || 0,
      source: this.source ? {lower: [...this.source.lower], upper: [...this.source.upper]} : null,
      centered: true, boxVisible: this.box.visible, sourceVisible: this.sourceBox.visible,
      ...(derived ? {streamlineData: this.streamlineData.map(curve => ({seed: [...curve.seed],
        ...Object.fromEntries(['forward', 'backward'].map(name => [name, {...curve[name],
          points: curve[name].points.map(point => [...point]), speeds: [...curve[name].speeds]}]))}))} : {}),
      updates: this.updates, disposed: this.disposed};
  }

  dispose() {
    if (this.disposed) return;
    this.group.removeFromParent();
    for (const object of [this.mesh, this.box, this.sourceBox, this.streamlineLines]) {
      object.geometry.dispose(); object.material.dispose();
    }
    this.texture?.dispose();
    this.lightTexture?.dispose();
    this.disposed = true;
  }
}
