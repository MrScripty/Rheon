"""Deterministic algebra/formula checks, not a fluid solver or Lean proof."""
from pathlib import Path
import json
import math
import platform
import numpy as np
import sympy as sp
from force_balance import pressure_fixture, check_pressure_force

ROOT = Path(__file__).resolve().parent
rng = np.random.default_rng(43177)
results = []


def record(name, residual, tolerance=1e-10, **details):
    value = float(abs(residual))
    if not math.isfinite(value) or value > tolerance:
        raise ValueError((name, value, tolerance))
    results.append(dict(name=name, residual=value, tolerance=tolerance,
                        passed=True, **details))


def energy(v, m):
    return float(v @ m @ v / 2)


worst_t1 = worst_t2 = worst_t3 = 0.
for _ in range(100):
    n, nb, ns = 7, 3, 9
    z = rng.normal(size=(n, n))
    M = z.T @ z + np.eye(n)
    B = rng.normal(size=(ns, n))
    W = np.diag(rng.uniform(0, 2, ns))
    C = rng.normal(size=(nb, n))
    L = np.diag(rng.uniform(0, 2, nb))
    K = B.T @ W @ B
    Q = K + C.T @ L @ C
    dt = rng.uniform(.001, .5)
    u = rng.normal(size=n)
    v = np.linalg.solve(M + dt * Q, M @ u)
    delta = v-u
    residual = energy(v, M)-energy(u, M)+energy(delta, M)+dt*(v@Q@v)
    worst_t1 = max(worst_t1, abs(residual))
    assert energy(v, M) <= energy(u, M)+1e-10
    w, f, e = rng.normal(size=nb), rng.normal(size=n), rng.normal(size=n)*1e-6
    v = np.linalg.solve(M+dt*Q, M@u + dt*(f+C.T@L@w)+e)
    r = C@v-w
    left = energy(v,M)-energy(u,M)+energy(v-u,M)+dt*(v@K@v+r@L@r)
    right = dt*(v@f-w@L@r)+v@e
    worst_t2 = max(worst_t2, abs(left-right))
    assert abs(v@e) <= math.sqrt(v@M@v)*math.sqrt(e@np.linalg.solve(M,e))+1e-12
    A = rng.normal(size=(3,n))
    b = rng.normal(size=3)
    MinvAt = np.linalg.solve(M,A.T)
    lam = np.linalg.solve(A@MinvAt,A@u-b)
    v = u-MinvAt@lam
    residual = energy(v,M)-energy(u,M)+energy(v-u,M)+lam@b
    worst_t3 = max(worst_t3, abs(residual), np.max(np.abs(A@v-b)))
record('T1 stationary implicit energy identity in 100 SPD systems',worst_t1)
record('T2 moving wall work and residual identity in 100 SPD systems',worst_t2)
record('T3 affine projection identity in 100 feasible systems',worst_t3)

M = np.diag([2., 3.])
u = np.array([4., -1.])
J = np.array([[1., -1.]])
MinvJt = np.linalg.solve(M,J.T)
lam = np.linalg.solve(J@MinvJt,J@u)
v = u-MinvJt@lam
record('T4 combined linear momentum',np.ones(2)@M@(v-u))
record('T4 combined energy budget',energy(v,M)-energy(u,M)+energy(v-u,M))
record('T4 relative velocity constraint',float((J@v)[0]))

omega = np.array([.7,-1.2,.3])
cross = np.array([[0,-omega[2],omega[1]],[omega[2],0,-omega[0]],[-omega[1],omega[0],0]])
record('T5 rigid rotation symmetric gradient',np.linalg.norm((cross+cross.T)/2))
x = np.array([[1.,0.,0.],[-1.,0.,0.]])
vr = np.cross(omega,x)
graph_dissipation = float(np.sum((vr[1]-vr[0])**2))
assert graph_dissipation > 0
results.append(dict(name='T5 graph smoothing rotation counterexample',passed=True,
                    graph_dissipation=graph_dissipation,
                    interpretation='Positive graph dissipation despite zero continuum strain'))

incidence=np.array([[1,0,-1],[-1,1,0],[0,-1,1]],dtype=float)
flux=rng.normal(size=3)
record('T6 internal face flux cancellation',np.sum(incidence@flux))
R=rng.uniform(0,1,(4,3)); R/=R.sum(axis=0)
delta=rng.normal(size=3)
record('T6 redistribution conservation',(R@delta).sum()-delta.sum())
vol=np.array([1.,2.,3.]); c0=1.7; dt=.05; G=np.array([.1,.3,-.2])
newvol=vol+dt*(incidence@G)
newmass=c0*vol+dt*c0*(incidence@G)
record('T6 uniform moving capacity field',np.max(np.abs(newmass-c0*newvol)))

G, p, expected_force = pressure_fixture()
record('T7 fixed finite pressure-gradient force balance',check_pressure_force(G@p),
       pressure=p.tolist(),expected_force=expected_force.tolist(),
       interpretation='Independent oriented face pressure jumps; algebraic balance only, no capillary assembly')

# Implicit midpoint for a harmonic energy: an explicit discrete-gradient witness.
m,k,dt,x0,v0=2.,7.,.13,.6,-.2
mat=np.array([[1.,-dt/2],[dt*k/2,m]])
x1,v1=np.linalg.solve(mat,np.array([x0+dt*v0/2,m*v0-dt*k*x0/2]))
record('T8 discrete gradient work identity',k*(x1+x0)/2*(x1-x0)-.5*k*(x1*x1-x0*x0))
record('T8 total energy conservation',.5*m*(v1*v1-v0*v0)+.5*k*(x1*x1-x0*x0))
zeta,k,dt,a0=3.,2.,.2,1.3
a1=(zeta-dt*k/2)/(zeta+dt*k/2)*a0
record('T9 contact coordinate dissipation',.5*k*(a1*a1-a0*a0)+zeta/dt*(a1-a0)**2)

mu,U,H,ell=sp.symbols('mu U H ell',positive=True)
slope=U/(H+2*ell)
power=mu*slope*U
bulk=mu*slope**2*H
wall=2*mu/ell*(slope*ell)**2
assert sp.simplify(power-bulk-wall)==0
results.append(dict(name='F2 symbolic Couette work partition',passed=True))
y,G=sp.symbols('y G',positive=True)
profile=G/(2*mu)*(y*(H-y)+ell*H)
flux=sp.integrate(profile,(y,0,H))
assert sp.simplify(flux-G*H**3/(12*mu)*(1+6*ell/H))==0
results.append(dict(name='F3 symbolic Poiseuille slip flux',passed=True))

R,c,z=sp.symbols('R c z',real=True)
cap_volume=sp.integrate(sp.pi*(R**2-(z+R*c)**2),(z,0,R*(1-c)))
assert sp.simplify(cap_volume-sp.pi*R**3*(2-3*c+c**3)/3)==0
results.append(dict(name='F6 symbolic spherical cap volume',passed=True))

for theta in np.deg2rad([30,60,90,120,150]):
    V=1.
    radius=(3*V/(math.pi*(2-3*math.cos(theta)+math.cos(theta)**3)))**(1/3)
    height=radius*(1-math.cos(theta))
    cap=math.pi*height**2*(radius-height/3)
    record(f'F6 equal volume cap at {round(math.degrees(theta))} degrees',cap-V)

mu,rho,H,U,G=.1,1000.,.01,.1,120.
record('F1 synthetic shear stress',mu*U/H-1.)
record('F1 synthetic wall power per area',mu*U*U/H-.1)
record('F3 synthetic mean speed',G*H*H/(12*mu)-.01)
record('F5 2D pressure jump',.072/.001-72.)
record('F5 3D pressure jump',2*.072/.001-144.)

out=dict(description='Mathematical identities and analytical geometry only; no Rheon simulation or Lean compilation',
         seed=43177,python=platform.python_version(),numpy=np.__version__,sympy=sp.__version__,
         passed=sum(r['passed'] for r in results),checks=results)
(ROOT/'sanity_results.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:out[k] for k in ['description','passed','seed']},indent=2))
