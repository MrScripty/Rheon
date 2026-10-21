// Pure display calculations on accepted fields. These do not advance the solver.
export const FLOW_LIGHT_EXTINCTION = 12;
export const FLOW_LIGHT_AMBIENT = .28;
export const FLOW_STREAMLINE_STEP = .01;
export const FLOW_STREAMLINE_MAX_LENGTH = 1;
export const FLOW_STREAMLINE_MIN_SPEED = 1e-4;
export const FLOW_STREAMLINE_SPEED_MAX = 1;
// Fixed physical coordinates in the lesson's unit cube, never fitted to a frame.
export const FLOW_STREAMLINE_SEEDS = Object.freeze([
  [.34, .14, .42], [.46, .14, .42], [.34, .24, .54],
  [.46, .24, .54], [.40, .19, .38], [.40, .19, .58],
].map(Object.freeze));

function gridSpec(grid) {
  if (['counts', 'spacing', 'origin'].some(key => grid?.[key]?.length !== 3)) throw new Error('Invalid field grid.');
  const counts = Array.from(grid.counts), spacing = Array.from(grid.spacing), origin = Array.from(grid.origin);
  if (counts.some(n => !Number.isSafeInteger(n) || n < 1) ||
      spacing.some(h => !Number.isFinite(h) || h <= 0) || origin.some(x => !Number.isFinite(x))) {
    throw new Error('Invalid field grid.');
  }
  const length = counts.reduce((a, b) => a * b, 1);
  const upper = origin.map((x, d) => x + counts[d] * spacing[d]);
  if (!Number.isSafeInteger(length) || upper.some(x => !Number.isFinite(x))) throw new Error('Field grid is too large.');
  return {counts, spacing, origin, upper, length};
}

function position3(position) {
  if (position?.length !== 3 || Array.from(position).some(x => !Number.isFinite(x))) throw new Error('Invalid field position.');
  return Array.from(position);
}

/** Optical depth toward +Y, at cell centers. This is the exact column integral
 * of the clamped trilinear tracer at those knots, including the upper half-cell.
 * Interpolating these depth values BETWEEN knots is a display approximation.
 * Extinction is an appearance parameter, not a measured material property. */
export function buildLightDepth(state, {extinction = FLOW_LIGHT_EXTINCTION} = {}) {
  const {counts: [nx, ny, nz], spacing, length} = gridSpec(state?.grid);
  if (!Number.isFinite(extinction) || extinction < 0 || state.tracer?.length !== length) throw new Error('Invalid light field.');
  const depth = new Float32Array(length);
  for (let k = 0; k < nz; k++) for (let i = 0; i < nx; i++) {
    let above = 0;
    for (let j = ny - 1; j >= 0; j--) {
      const index = i + nx * (j + ny * k), c = state.tracer[index];
      if (!Number.isFinite(c) || c < 0) throw new Error('Tracer must be finite and nonnegative.');
      depth[index] = extinction * spacing[1] * (.5 * c + above);
      if (!Number.isFinite(depth[index])) throw new Error('Light depth exceeds float32 range.');
      above += c;
    }
  }
  return depth;
}

/** Trilinear MAC sampling with the actual face offsets: the component's own
 * axis lies at integer grid coordinates; the other two axes at half-integers.
 * Component arrays and their x-fastest dimensions are validated once. */
export function createMacSampler(state) {
  const {counts, spacing, origin, upper} = gridSpec(state?.grid);
  const fields = ['x', 'y', 'z'].map((key, axis) => {
    const shape = counts.map((n, d) => n + Number(d === axis));
    const data = state[key];
    if (data?.length !== shape.reduce((a, b) => a * b, 1)) throw new Error(`Invalid MAC ${key} field.`);
    for (const value of data) if (!Number.isFinite(value)) throw new Error(`Nonfinite MAC ${key} value.`);
    return {shape, data, axis};
  });
  return position => {
    const p = position3(position);
    return fields.map(({shape, data, axis}) => {
      const q = p.map((x, d) => (Math.max(origin[d], Math.min(upper[d], x)) - origin[d]) / spacing[d] - (d === axis ? 0 : .5));
      const lo = q.map(Math.floor), fraction = q.map((x, d) => x - lo[d]);
      let result = 0;
      for (let z = 0; z < 2; z++) for (let y = 0; y < 2; y++) for (let x = 0; x < 2; x++) {
        const offset = [x, y, z];
        const index = offset.map((v, d) => Math.max(0, Math.min(shape[d] - 1, lo[d] + v)));
        const weight = offset.reduce((w, v, d) => w * (v ? fraction[d] : 1 - fraction[d]), 1);
        result += data[index[0] + shape[0] * (index[1] + shape[1] * index[2])] * weight;
      }
      return result;
    });
  };
}

export function sampleMacVelocity(state, position) { return createMacSampler(state)(position); }

function traceWithSampler(grid, sample, seed, {step = FLOW_STREAMLINE_STEP,
  maxLength = FLOW_STREAMLINE_MAX_LENGTH, minSpeed = FLOW_STREAMLINE_MIN_SPEED, maxSteps = 100} = {}) {
  const {origin, upper} = grid;
  const start = position3(seed);
  const inside = p => p.every((x, d) => x >= origin[d] && x <= upper[d]);
  if (!inside(start)) throw new Error('Streamline seed is outside the domain.');
  if (![step, maxLength, minSpeed].every(x => Number.isFinite(x) && x > 0) ||
      !Number.isSafeInteger(maxSteps) || maxSteps < 1 || maxSteps > 10000) throw new Error('Invalid streamline limits.');
  const branch = direction => {
    const points = [start.slice()], speeds = [Math.hypot(...sample(start))];
    let length = 0, termination = 'steps';
    const tangent = p => {
      const velocity = sample(p), speed = Math.hypot(...velocity);
      return speed < minSpeed ? null : velocity.map(v => direction * v / speed);
    };
    const offset = (p, v, h) => p.map((x, d) => x + v[d] * h);
    for (let iteration = 0; iteration < maxSteps; iteration++) {
      const p = points[points.length - 1], h = Math.min(step, maxLength - length);
      if (h <= 1e-12) { termination = 'length'; break; }
      const a = tangent(p);
      const b = a && tangent(offset(p, a, h / 2));
      const c = b && tangent(offset(p, b, h / 2));
      const d = c && tangent(offset(p, c, h));
      if (!d) { termination = 'stagnation'; break; }
      const next = p.map((x, axis) => x + h * (a[axis] + 2 * b[axis] + 2 * c[axis] + d[axis]) / 6);
      if (!inside(next)) {
        // Clip the final small chord to the physical box; never draw outside it.
        let fraction = 1;
        for (let axis = 0; axis < 3; axis++) {
          if (next[axis] < origin[axis]) fraction = Math.min(fraction, (origin[axis] - p[axis]) / (next[axis] - p[axis]));
          if (next[axis] > upper[axis]) fraction = Math.min(fraction, (upper[axis] - p[axis]) / (next[axis] - p[axis]));
        }
        if (fraction > 1e-12) {
          const end = p.map((x, axis) => Math.max(origin[axis], Math.min(upper[axis], x + fraction * (next[axis] - x))));
          points.push(end); speeds.push(Math.hypot(...sample(end))); length += h * fraction;
        }
        termination = 'boundary'; break;
      }
      points.push(next); speeds.push(Math.hypot(...sample(next))); length += h;
      if (maxLength - length <= 1e-12) { termination = 'length'; break; }
    }
    return {points, speeds, length, termination};
  };
  return {seed: start, forward: branch(1), backward: branch(-1)};
}

/** Spatial curves in one frozen velocity field, NOT particle paths over time.
 * RK4 integrates dx/ds = ±v/|v| with a fixed physical arc step. Component values
 * are clamped at their outer sample locations, including intermediate stages. */
export function traceStreamline(state, seed, options = {}) {
  return traceWithSampler(gridSpec(state?.grid), createMacSampler(state), seed, options);
}

export function buildInstantaneousStreamlines(state, {seeds = FLOW_STREAMLINE_SEEDS, ...options} = {}) {
  const grid = gridSpec(state?.grid), sample = createMacSampler(state);
  return seeds.filter(seed => position3(seed).every((x, d) => x >= grid.origin[d] && x <= grid.upper[d]))
    .map(seed => traceWithSampler(grid, sample, seed, options));
}
