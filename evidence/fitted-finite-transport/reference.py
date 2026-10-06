"""Actual finite bottom-fixed, cap-driven Powell–Sabin ALE transport research.

No production advancing API is enabled here. Spatial and instantaneous Dual
arithmetic reuse the frozen corrected rational reference. Symbolic path fluxes
are integrated before constrained donor/viscosity solves; no mass-fitted flux.
"""
import importlib.util,inspect,sys
from pathlib import Path
from functools import lru_cache
import numpy as np
import sympy as sp
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('corrected_ps',ROOT/'evidence/fitted-height-periodic-repair/reference.py')
ref=importlib.util.module_from_spec(spec);sys.modules[spec.name]=ref;spec.loader.exec_module(ref)
require=ref.require
code=inspect.getsource(ref.mesh).replace('def mesh(speed=Q(1, 4)):','def offset_mesh(offset=Q(0), speed=Q(1, 4)):').replace('Dual(Q(i, 4), speed)','Dual(Q(i, 4)+offset, speed)')
exec(code,ref.__dict__)
q=sp.Symbol('q',real=True)
def cross(a,b):return a[0]*b[1]-a[1]*b[0]
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def add(a,b):return tuple(x+y for x,y in zip(a,b))
def scale(a,b):return tuple(x*b for x in a)

@lru_cache(maxsize=1)
def symbolic_path():
    h=[sp.Rational(1),sp.Rational(5,4),sp.Rational(1),sp.Rational(3,2),sp.Rational(1)]
    points=[(sp.Rational(i,4),sp.Rational(0))for i in range(5)]+[(sp.Rational(i,4)+q,y)for i,y in enumerate(h)]
    macro=[]
    for i in range(4):macro.extend([(i,i+1,i+6),(i,i+6,i+5)])
    centers=[]
    for tri in macro:centers.append(len(points));points.append(scale(add(add(points[tri[0]],points[tri[1]]),points[tri[2]]),sp.Rational(1,3)))
    adjacent={}
    for t,tri in enumerate(macro):
        for k in range(3):adjacent.setdefault(tuple(sorted((tri[k],tri[(k+1)%3]))),[]).append(t)
    left,right=(0,5),(4,9);adjacent[left].append(adjacent[right][0]);adjacent[right].append(adjacent[left][0]);edge_nodes={};constraints={}
    for edge,owners in sorted(adjacent.items()):
        a,b=(points[i]for i in edge)
        if len(owners)==2:
            p,r=(points[centers[i]]for i in owners)
            if edge==left:r=(r[0]-1,r[1])
            elif edge==right:r=(r[0]+1,r[1])
            ab,pr=sub(b,a),sub(r,p);s=sp.cancel(cross(sub(p,a),pr)/cross(ab,pr));pos=add(a,scale(ab,s))
        else:pos=scale(add(a,b),sp.Rational(1,2))
        node=len(points);points.append(tuple(sp.cancel(x)for x in pos));edge_nodes[edge]=node
        if len(owners)==1 and (all(i<5 for i in edge)or all(5<=i<10 for i in edge)):constraints[node]=edge
    micro=[]
    for t,tri in enumerate(macro):
        for k in range(3):
            a,b=tri[k],tri[(k+1)%3];e,c=edge_nodes[tuple(sorted((a,b)))],centers[t];micro.extend([(a,e,c),(e,b,c)])
    representative=list(range(len(points)));representative[4]=0;representative[9]=5;representative[edge_nodes[right]]=edge_nodes[left];unique=sorted(set(representative));compact={i:j for j,i in enumerate(unique)};ids=[compact[i]for i in representative]
    constraints={ids[i]:tuple(ids[j]for j in ends)for i,ends in constraints.items()};n=max(ids)+1
    mass=[sp.Rational(0)]*n;flux={};areas=[];velocity=[tuple(sp.diff(x,q)for x in p)for p in points]
    for tri in micro:
        p=[points[i]for i in tri];area=sp.cancel(cross(sub(p[1],p[0]),sub(p[2],p[0]))/2)
        areas.append(area)
        for node in tri:mass[ids[node]]+=area # rho=3,b=1, nodal area/3
        center=scale(add(add(p[0],p[1]),p[2]),sp.Rational(1,3))
        for i in range(3):
            j,k=(i+1)%3,(i+2)%3;a,b=ids[tri[i]],ids[tri[j]]
            if a==b:continue
            segment=sub(center,scale(add(p[i],p[j]),sp.Rational(1,2)));wm=add(add(scale(velocity[tri[i]],sp.Rational(5,12)),scale(velocity[tri[j]],sp.Rational(5,12))),scale(velocity[tri[k]],sp.Rational(1,6)))
            # Divide physical mass flux by a=dq/dt=1/4 before integration.
            f=3*((1-wm[0])*segment[1]+wm[1]*segment[0]);pair=(min(a,b),max(a,b));flux[pair]=flux.get(pair,0)+(f if a<b else -f)
    mass=[sp.cancel(x)for x in mass];flux={ij:sp.cancel(f)for ij,f in flux.items()};flux={ij:f for ij,f in flux.items()if f!=0};balance=[sp.diff(m,q)for m in mass]
    for (i,j),f in flux.items():balance[i]+=f;balance[j]-=f
    require(all(sp.cancel(x)==0 for x in balance),'exact symbolic instantaneous GCL along whole path')
    require(sp.cancel(sum(mass)-sp.Rational(57,16))==0,'finite whole-path liquid mass')
    # Certify no geometry poles, nonpositive microareas or flux direction
    # reversals in this explicitly bounded interval. No sampled rank claim.
    lower,upper=sp.Rational(0),sp.Rational(1,8)
    def roots_in_interval(expr):
        roots=sp.polys.polytools.intervals(sp.Poly(expr,q),eps=sp.Rational(1,10**20))
        return [interval for interval,multiplicity in roots if interval[1]>=lower and interval[0]<=upper]
    for m in [*mass,*areas]:
        num,den=sp.fraction(m);require(not roots_in_interval(num) and not roots_in_interval(den) and m.subs(q,0)>0,'positive whole-path mass and microarea')
    signs={}
    for pair,f in flux.items():
        num,den=sp.fraction(f);require(not roots_in_interval(num) and not roots_in_interval(den),'no flux reversal or pole on supported interval');signs[pair]=1 if f.subs(q,0)>0 else -1
    def real_log(term):
        arg=term.args[0]
        sign=arg.subs(q,0)
        require(sign!=0 and sign.is_real,'real logarithm branch on bounded path')
        return sp.log(arg if sign>0 else -arg)
    primitive={pair:sp.integrate(f,q).replace(lambda e:e.func==sp.log,real_log)for pair,f in flux.items()}
    require(all(sp.simplify(sp.diff(primitive[pair],q)-f)==0 for pair,f in flux.items()),'actual flux antiderivatives')
    finite_balance=list(mass)
    for (i,j),F in primitive.items():finite_balance[i]+=F;finite_balance[j]-=F
    require(all(sp.simplify(sp.diff(v,q))==0 for v in finite_balance),'actual antiderivatives integrate finite GCL')
    R=np.array(ref.scalar_embedding(n,constraints),float)
    return {'points':points,'micro':micro,'ids':ids,'constraints':constraints,'mass':mass,'flux':flux,'primitive':primitive,'signs':signs,'R':R}

@lru_cache(maxsize=128)
def spatial(offset):
    points,micro,ids,constraints,_=ref.offset_mesh(ref.Q(offset));mass,rate,k,_,_=ref.operators(points,micro,mu=ref.Q(1,20));mass=ref.merge(mass,ids);rate=ref.merge(rate,ids);n=len(mass);R=np.array(ref.scalar_embedding(n,constraints),float);km=ref.zeros(n,n)
    for i in range(len(points)):
        for j in range(len(points)):km[ids[i]][ids[j]]+=k[3*i+2][3*j+2]
    return np.array(mass,float),np.array(rate,float),np.array(km,float),R,points

def integrated_flux(left,right,digits=50):
    s=symbolic_path();a=sp.Rational(left);b=sp.Rational(right)
    require(0<=a<b<=sp.Rational(1,8),'supported finite path interval')
    return {ij:float(sp.N(F.subs(q,b)-F.subs(q,a),digits))for ij,F in s['primitive'].items()}

if __name__=='__main__':
    import json,time
    start=time.time();s=symbolic_path();F=integrated_flux(ref.Q(0),ref.Q(1,8));m0=sp.Matrix([m.subs(q,0)for m in s['mass']]);m1=sp.Matrix([m.subs(q,sp.Rational(1,8))for m in s['mass']]);g=np.array(m1-m0,float).reshape(-1)
    for (i,j),f in F.items():g[i]+=f;g[j]-=f
    print(json.dumps({'scope':'actual finite bottom-fixed/cap-driven ALE flux; no native advancing API','shared_nonzero_pairs':len(F),'nonzero_mass_changes':sum(x!=0 for x in m1-m0),'finite_liquid_mass':str(sum(m1)),'max_float_integrated_GCL':float(np.max(np.abs(g))),'logarithmic_primitives':sum(F.has(sp.log)for F in s['primitive'].values()),'whole_interval_symbolic_GCL':True,'positive_masses_microareas_and_no_flux_reversal_interval':'[0,1/8]','new_native_API':False},indent=2),flush=True)
