"""Vectorized independent transition oracle for the frozen 32³ lab scene.

Reconstruct every stage from the prior captured state and prescribed seed/time,
never from a teacher trajectory or current output diagnostics. All calculations
use NumPy float64 with explicit f32 storage rounding. Engineering bounds allow storage/libm roundoff in this frozen scene. They are
not a formal cross-platform error proof, tunable physical tolerance or fitted
result threshold.
"""
import math

import numpy as np

N = 32
DT = 1/60
OFFSETS = ((0.,.5,.5),(.5,0.,.5),(.5,.5,0.))
SHAPES = ((32,32,33),(32,33,32),(33,32,32))  # NumPy z,y,x
FACES = 33*32*32
EPS32 = np.finfo(np.float32).eps
EPS64 = np.finfo(float).eps
TINY32 = float(np.nextafter(np.float32(0),np.float32(1)))


def rounded(values):
    return values.astype(np.float32).astype(np.float64)


def coordinates(shape, offset, origin):
    z,y,x = np.indices(shape,dtype=np.float64)
    return tuple(origin+(a+b)/N for a,b in zip((x,y,z),offset))


def sample(field, offset, position, origin):
    # Convex eight-corner interpolation in the field's own MAC sample space.
    shape = field.shape[::-1]
    q = [np.clip((np.clip(p,origin,origin+1)-origin)*N-o,0,n-1)
         for p,o,n in zip(position,offset,shape)]
    low = [v.astype(np.intp) for v in q]
    high = [np.minimum(v+1,n-1) for v,n in zip(low,shape)]
    fraction = [v-i for v,i in zip(q,low)]
    result = np.zeros_like(q[0])
    minimum, maximum = np.full_like(result,np.inf), np.full_like(result,-np.inf)
    for z in range(2):
        for y in range(2):
            for x in range(2):
                bits = (x,y,z)
                index = [high[d] if bits[d] else low[d] for d in range(3)]
                weight = np.ones_like(result)
                for d in range(3):
                    weight *= fraction[d] if bits[d] else 1-fraction[d]
                value = field[index[2],index[1],index[0]]
                result += weight*value
                minimum = np.minimum(minimum,value)
                maximum = np.maximum(maximum,value)
    return np.clip(result,minimum,maximum)


def backtrace(velocity, position, origin):
    first = [sample(v,o,position,origin) for v,o in zip(velocity,OFFSETS)]
    middle = [np.clip(p-.5*DT*v,origin,origin+1) for p,v in zip(position,first)]
    second = [sample(v,o,middle,origin) for v,o in zip(velocity,OFFSETS)]
    return tuple(np.clip(p-DT*v,origin,origin+1) for p,v in zip(position,second))


def scene(seed, step):
    time = step*DT
    phase = (seed % 997)*.137
    center = (.43+.13*math.sin(phase+time),.28+.06*math.cos(phase),.45+.13*math.cos(phase-time))
    direction = (.75*math.cos(phase+2*time),1.,.75*math.sin(phase+2*time))
    return center,direction


def forcing(position, origin, center, seed, step, component, direction):
    distance = sum((p-origin-c)**2 for p,c in zip(position,center))
    # Same declared integer scene generator; wrapping operations are defined mod 2^64.
    bits = np.arange(FACES,dtype=np.uint64).reshape(SHAPES[component])
    bits = bits+np.uint64(seed*7919+step*104729+component*997)
    bits = (bits^(bits >> np.uint64(30)))*np.uint64(0xbf58476d1ce4e5b9)
    bits = (bits^(bits >> np.uint64(27)))*np.uint64(0x94d049bb133111eb)
    bits ^= bits >> np.uint64(31)
    noise = 2*((bits >> np.uint64(40)).astype(float)/16_777_216)-1
    return np.exp(-distance/.012)*direction+.05*noise


def norm_error(a, b):
    denominator = float(np.linalg.norm(b))
    numerator = float(np.linalg.norm(a-b))
    return numerator/denominator if denominator else numerator


def transition(previous, current, seed, frame, shifted):
    """Check frame 1 against rest, subsequent frames against captured prior state."""
    _, pressure, observed_velocity, observed_tracer = current
    origin = 1. if shifted else 0.
    if previous is None:
        old = [np.zeros(s) for s in SHAPES]
        tracer = np.zeros((N,N,N))
    else:
        old = [previous[2][d*FACES:(d+1)*FACES].reshape(s) for d,s in enumerate(SHAPES)]
        tracer = previous[3].reshape(N,N,N)
    center,direction = scene(seed,frame-1)
    candidate, bounds = [], []
    for d,(field,shape,offset) in enumerate(zip(old,SHAPES,OFFSETS)):
        position = coordinates(shape,offset,origin)
        departure = backtrace(old,position,origin)
        transported = rounded(sample(field,offset,departure,origin))
        acceleration = forcing(position,origin,center,seed,frame-1,d,direction[d])
        impulse = DT*acceleration
        candidate.append(rounded(transported+impulse))
        # Eight storage ULP scales conservatively cover independent f64 libm and
        # interpolation ordering, the stored advection and stored force addition.
        bounds.append(8*EPS32*(np.abs(transported)+np.abs(impulse))+8*TINY32)
        wall = [slice(None)]*3
        wall[2-d] = (0,-1)
        candidate[-1][tuple(wall)] = 0
        bounds[-1][tuple(wall)] = 0
    rhs = np.zeros((N,N,N))
    rhs_bound = np.zeros_like(rhs)
    for d,(field,bound) in enumerate(zip(candidate,bounds)):
        axis = 2-d
        low,high = [slice(None)]*3,[slice(None)]*3
        low[axis],high[axis] = slice(0,-1),slice(1,None)
        rhs += (field[tuple(low)]-field[tuple(high)])/N**2
        rhs_bound += (bound[tuple(low)]+bound[tuple(high)])/N**2
    rhs /= DT
    rhs_bound = rhs_bound/DT+64*EPS64*np.abs(rhs)
    if not np.all(np.abs(rhs.flatten()-current[0]) <= rhs_bound.flatten()):
        raise ValueError(f'transition RHS from prior state/forcing: seed {seed}, frame {frame}')
    p = pressure.reshape(N,N,N)
    corrected, correction_bounds = [], []
    for d,(field,bound) in enumerate(zip(candidate,bounds)):
        delta = DT*(N/1.25)*np.diff(p,axis=2-d)
        interior = [slice(None)]*3
        interior[2-d] = slice(1,-1)
        value = np.zeros_like(field)
        value[tuple(interior)] = field[tuple(interior)]-delta
        allowance = bound.copy()
        allowance[tuple(interior)] += 8*EPS32*(np.abs(field[tuple(interior)])+np.abs(delta))+8*TINY32
        corrected.append(rounded(value).flatten())
        correction_bounds.append(allowance.flatten())
    corrected = np.concatenate(corrected)
    if not np.all(np.abs(corrected-observed_velocity) <= np.concatenate(correction_bounds)):
        raise ValueError(f'transition pressure correction: seed {seed}, frame {frame}')
    position = coordinates((N,N,N),(.5,.5,.5),origin)
    distance = sum((p-origin-c)**2 for p,c in zip(position,center))
    emission = DT*8*np.exp(-distance/.006)
    source = rounded(np.minimum(tracer+emission,1))
    source_bound = 8*EPS32*(np.abs(tracer)+np.abs(emission))+8*TINY32
    # The native tracer stage uses the published f32 corrected velocity. Its
    # values were just bound to the pressure correction of this same candidate.
    velocity = [observed_velocity[d*FACES:(d+1)*FACES].reshape(s) for d,s in enumerate(SHAPES)]
    departure = backtrace(velocity,position,origin)
    expected = rounded(sample(source,(.5,.5,.5),departure,origin))
    tracer_bound = sample(source_bound,(.5,.5,.5),departure,origin)+8*EPS32*np.abs(expected)+8*TINY32
    # Sub-ULP differences in f64 departure coordinates can reach a donor with
    # tiny concentration. Bound that f64 term by the actual donor field scale.
    tracer_bound += 128*EPS64*float(np.max(np.abs(source)))
    if not np.all(np.abs(expected.flatten()-observed_tracer) <= tracer_bound.flatten()):
        raise ValueError(f'transition tracer from prior state: seed {seed}, frame {frame}')
    return {'rhs_relative_l2':norm_error(current[0],rhs.flatten()),
            'corrected_velocity_relative_l2':norm_error(observed_velocity,corrected),
            'tracer_relative_l2':norm_error(observed_tracer,expected.flatten())}
