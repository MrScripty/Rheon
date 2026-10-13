// Unit cubes; positive x sweep; initially empty receiver. No native solver.
export function sweptVolume(f, c, side = 'left') {
  if (![f,c].every(x=>Number.isFinite(x)&&x>=0&&x<=1) || !['left','right'].includes(side))
    throw new RangeError('Require fractions and swept width in [0,1], and left/right liquid.');
  const lo=side==='left'?0:1-f, hi=side==='left'?f:1;
  const overlapLo=Math.max(lo,1-c), overlapHi=hi;
  const q=Math.max(0,overlapHi-overlapLo);
  const raw=[f-c,c], clamped=raw.map(x=>Math.max(0,Math.min(1,x)));
  return {f,c,side,lo,hi,overlapLo,overlapHi,q,donor:f-q,receiver:q,
    mixed:c*f,raw,clamped,clampDrift:clamped[0]+clamped[1]-f};
}
