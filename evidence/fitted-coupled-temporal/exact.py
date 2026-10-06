"""Exact instantaneous compatibility, full momentum/work and cap-rank chart."""
from functools import lru_cache
from fractions import Fraction as Q
import json
import sympy as sp
import model as m
b=m.base


def apply_chart(cap, free_velocity):
    C=[m.INITIAL['R'][3*3],m.INITIAL['R'][3*4],m.INITIAL['R'][3*3+1]]
    D=m.INITIAL['D']
    rows=b.independent_columns(m.old.transpose(D))
    L=[D[i]for i in rows]+C
    pivots=b.independent_columns(L)
    free=[i for i in range(22)if i not in pivots]
    m.require(len(pivots)==19 and free==[19,20,21],'exact div/cap chart rank')
    full=L+[[Q(int(j==i))for j in range(22)]for i in m.SELECT]
    z=b.solve(full,[Q(0)]*16+list(cap)+list(free_velocity))
    m.require(b.mv(D,z)==[Q(0)]*24,'all strong div rows on cap chart')
    m.require(b.mv(C,z)==list(cap),'independent material cap velocities')
    m.require(b.dot(m.INITIAL['R'][3*4+1],z)==-cap[2],'fourth cap velocity from volume')
    return z


@lru_cache(maxsize=4)
def instantaneous(kind):
    z=m.old.force_solve()['old' if kind=='initial' else 'z']
    raw=b.mv(m.INITIAL['R'],z)
    cap=tuple(tuple(raw[3*i+d]for d in range(2))for i in [3,4,5])
    points,micro,ids,_,_=m.geometry(m.INITIAL_CAP,cap)
    _,mdot_raw,_,_,areas=b.operators(points,micro,mu=Q(1,20))
    mdot=b.merge(mdot_raw,ids)
    dprime=b.zeros(24,22);f={}
    for t,tri in enumerate(micro):
        p=[points[i]for i in tri]
        twice=b.cross(b.sub(p[1],p[0]),b.sub(p[2],p[0]))
        center=b.scale(b.add(b.add(p[0],p[1]),p[2]),Q(1,3))
        for i in range(3):
            j,k=(i+1)%3,(i+2)%3
            gx=(p[j][1]-p[k][1])/twice;gy=(p[k][0]-p[j][0])/twice
            for col in range(22):
                dprime[t][col]+=gx.d*m.INITIAL['R'][3*tri[i]][col]+gy.d*m.INITIAL['R'][3*tri[i]+1][col]
            segment=b.sub(center,b.scale(b.add(p[i],p[j]),Q(1,2)))
            relative=[sum(weight*(raw[3*tri[l]+d]-p[l][d].d)for l,weight in zip((i,j,k),(Q(5,12),Q(5,12),Q(1,6))))for d in range(2)]
            value=3*(relative[0]*segment[1].v-relative[1]*segment[0].v)
            u,v=ids[tri[i]],ids[tri[j]]
            pair=min(u,v),max(u,v)
            f[pair]=f.get(pair,Q(0))+(value if u<v else -value)
    u=[None]*16
    for raw_index,node in enumerate(ids):u[node]=raw[3*raw_index:3*raw_index+3]
    convection=b.zeros(16,3);adv=Q(0)
    for (i,j),value in f.items():
        donor=u[i]if value>=0 else u[j]
        for d in range(3):
            convection[i][d]+=value*donor[d];convection[j][d]-=value*donor[d]
            adv+=abs(value)*(u[i][d]-u[j][d])**2/2
    g=b.mv(m.INITIAL['K'],z)
    for i in range(len(points)):
        for d in range(3):
            for col in range(22):g[col]+=m.INITIAL['R'][3*i+d][col]*mdot_raw[i]*raw[3*i+d]
    # Merge convection once; never double count periodic raw representatives.
    Rn=b.zeros(48,22)
    for raw_index,node in enumerate(ids):
        for d in range(3):Rn[3*node+d]=m.INITIAL['R'][3*raw_index+d]
    for i in range(16):
        for d in range(3):
            for col in range(22):g[col]+=Rn[3*i+d][col]*convection[i][d]
    Bt=m.old.transpose(m.INITIAL['B'])
    moving=b.mv(dprime,z)
    target=b.mv(m.old.transpose(m.INITIAL['Q']),[a*v for a,v in zip(areas,moving)])
    saddle=[M+BT for M,BT in zip(m.INITIAL['M'],Bt)]+[row+[Q(0)]*16 for row in m.INITIAL['B']]
    rhs=[-value for value in g]+target
    solution=b.solve(saddle,rhs);acc,p=solution[:22],solution[22:]
    strong=[a+d for a,d in zip(b.mv(m.INITIAL['D'],acc),moving)]
    m.require(strong==[Q(0)]*24,'exact full differentiated strong constraint')
    m.require(b.mv(saddle,solution)==rhs,'exact coupled momentum/pressure equations')
    net=b.flux_divergence(f,16)
    m.require([x+y for x,y in zip(mdot,net)]==[Q(0)]*16,'exact instantaneous local physical GCL')
    pressure_work=b.dot(z,b.mv(Bt,p))
    strain=b.dot(z,b.mv(m.INITIAL['K'],z))
    energy_rate=b.dot(z,b.mv(m.INITIAL['M'],acc))+sum(mdot[i]*b.dot(u[i],u[i])/2 for i in range(16))
    m.require(energy_rate+adv+strain==0 and pressure_work==0,'exact coupled donor/strain/pressure work')
    constant_x=[Q(1)]*12+[Q(0)]*10
    xrate=b.dot(constant_x,b.mv(m.INITIAL['M'],acc))+sum(mdot[i]*u[i][0]for i in range(16))
    m.require(xrate==0,'exact horizontal momentum rate')
    return dict(z=z,acceleration=acc,pressure=p,Ddot=dprime,moving_term=moving,physical_flux=f,
                nodal_mdot=mdot,energy_rate=energy_rate,advection_loss=adv,strain_power=strain,
                pressure_work=pressure_work,horizontal_momentum_rate=xrate)


def incompatible_component():
    weighted_transpose=sp.Matrix([[Q(t)*Q(a)for t,a in zip(row,m.INITIAL['areas'])]for row in m.old.transpose(m.INITIAL['Q'])])
    witness=list(weighted_transpose.nullspace()[0]);w=[Q(x)for x in witness]
    m.require(any(w)and b.mv(weighted_transpose.tolist(),w)==[Q(0)]*16,'exact invisible incompatible full component')
    return w


def summary():
    rows=[]
    for kind in ['initial','pressure_state']:
        s=instantaneous(kind)
        rows.append(dict(kind=kind,acceleration=[str(x)for x in s['acceleration']],pressure_coefficients=[str(x)for x in s['pressure']],
                    reconstructed_pressure_max=float(max(map(abs,b.mv(m.INITIAL['Q'],s['pressure'])))),
                    moving_constraint_term_max=float(max(map(abs,s['moving_term']))),
                    full_momentum_residual='0',full_strong_acceleration_residual='0',local_GCL_residual='0',
                    energy_rate=str(s['energy_rate']),advection_loss=str(s['advection_loss']),strain_power=str(s['strain_power']),
                    pressure_work='0',horizontal_momentum_rate='0'))
    chart=apply_chart((Q(1,7),Q(2,9),Q(1,13)),(Q(2,11),Q(3,17),Q(-1,19)))
    return dict(scope='exact tangent compatibility and six-coordinate div/material-cap chart, not finite advancement',
                rows=rows,divergence_rank=16,divergence_plus_independent_cap_rank=19,interior_coordinate_selectors=m.SELECT.tolist(),
                exact_nontrivial_cap_chart_example=[str(x)for x in chart],
                exact_image_invisible_incompatible_component=[str(x)for x in incompatible_component()],
                public_general_step_enabled=False,new_Lean_claims=0)


if __name__=='__main__':print(json.dumps(summary(),indent=2))
