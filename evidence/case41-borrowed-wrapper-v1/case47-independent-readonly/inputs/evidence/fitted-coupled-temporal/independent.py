"""Independent rational chart/physical operators and high-precision work sums.

Only the published binary cell coefficients and standard quadrature nodes are
inputs. No NumPy momentum/strain/flux or nonlinear solver results are reused.
"""
from fractions import Fraction as Q
import json
import sys
import mpmath as mp
import numpy as np
import model as m
from certificate import rational_geometry

b=m.base
mp.mp.dps=80
Rn=b.zeros(48,22)
for raw,node in enumerate(m.INITIAL['ids']):
    for d in range(3):Rn[3*node+d]=m.INITIAL['R'][3*raw+d]
C=[[Q(x)for x in row]for row in m.C_CAP]
selectors=[[Q(int(j==i))for j in range(22)]for i in m.SELECT]


def point(q0,eta0,alpha,pressure,time):
    D,areas,_,points,micro=rational_geometry(q0,eta0,alpha,time)
    L=[D[i]for i in m.ROWS]+C+selectors
    eta=[eta0[i]+time*alpha[i]for i in range(6)]
    z=b.solve(L,[Q(0)]*16+eta)
    b.require(b.mv(D,z)==[Q(0)]*24,'independent all full chart divergence rows')
    raw=b.mv(m.INITIAL['R'],z);u=[raw[3*i:3*i+3]for i in range(19)]
    un=[[Q(0)]*3 for _ in range(16)]
    for i,node in enumerate(m.INITIAL['ids']):un[node]=u[i]
    mass=[Q(0)]*16;mdot=[Q(0)]*16;Ddot=b.zeros(24,22);flux={};strain=Q(0);strain_force=[Q(0)]*22;pressure_force=[Q(0)]*22
    for t,tri in enumerate(micro):
        p=[points[i]for i in tri]
        twice=b.cross(b.sub(p[1],p[0]),b.sub(p[2],p[0]))
        center=b.scale(b.add(b.add(p[0],p[1]),p[2]),Q(1,3))
        gradient=[]
        for i in range(3):
            j,k=(i+1)%3,(i+2)%3
            gx=(p[j][1]-p[k][1])/twice;gy=(p[k][0]-p[j][0])/twice
            gradient.append((gx.v,gy.v))
            mass[m.INITIAL['ids'][tri[i]]]+=twice.v/2;mdot[m.INITIAL['ids'][tri[i]]]+=twice.d/2
            for col in range(22):Ddot[t][col]+=gx.d*m.INITIAL['R'][3*tri[i]][col]+gy.d*m.INITIAL['R'][3*tri[i]+1][col]
            segment=b.sub(center,b.scale(b.add(p[i],p[j]),Q(1,2)))
            relative=[sum(weight*(u[tri[l]][d]-p[l][d].d)for l,weight in zip((i,j,k),(Q(5,12),Q(5,12),Q(1,6))))for d in range(2)]
            f=3*(relative[0]*segment[1].v-relative[1]*segment[0].v)
            i0,j0=m.INITIAL['ids'][tri[i]],m.INITIAL['ids'][tri[j]]
            pair=min(i0,j0),max(i0,j0);flux[pair]=flux.get(pair,Q(0))+(f if i0<j0 else -f)
        exx=sum(gx*u[i][0]for i,(gx,gy)in zip(tri,gradient))
        eyy=sum(gy*u[i][1]for i,(gx,gy)in zip(tri,gradient))
        exy=sum(gy*u[i][0]+gx*u[i][1]for i,(gx,gy)in zip(tri,gradient))
        strain+=Q(1,20)*areas[t]*(2*exx**2+2*eyy**2+exy**2)
        pt=sum(D[t][j]*pressure[k]for k,j in enumerate(m.PIVOT))
        for col in range(22):
            e0=sum(gx*m.INITIAL['R'][3*i][col]for i,(gx,gy)in zip(tri,gradient))
            e1=sum(gy*m.INITIAL['R'][3*i+1][col]for i,(gx,gy)in zip(tri,gradient))
            e2=sum(gy*m.INITIAL['R'][3*i][col]+gx*m.INITIAL['R'][3*i+1][col]for i,(gx,gy)in zip(tri,gradient))
            strain_force[col]+=Q(1,20)*areas[t]*(2*exx*e0+2*eyy*e1+exy*e2)
            pressure_force[col]-=areas[t]*pt*D[t][col]
    acceleration=b.solve(L,[-b.dot(Ddot[i],z)for i in m.ROWS]+list(alpha))
    b.require([a+d for a,d in zip(b.mv(D,acceleration),b.mv(Ddot,z))]==[Q(0)]*24,'independent full strong acceleration')
    acceleration_nodes=[b.mv(Rn[3*i:3*i+3],acceleration)for i in range(16)]
    convection=[[Q(0)]*3 for _ in range(16)];adv=Q(0)
    for (i,j),f in flux.items():
        donor=un[i]if f>=0 else un[j]
        for d in range(3):
            convection[i][d]+=f*donor[d];convection[j][d]-=f*donor[d]
            adv+=abs(f)*(un[i][d]-un[j][d])**2/2
    b.require([a+c for a,c in zip(mdot,b.flux_divergence(flux,16))]==[Q(0)]*16,'independent exact instantaneous GCL')
    energy=sum(mass[i]*b.dot(un[i],un[i])/2 for i in range(16))
    force=strain_force[:];inertia=[Q(0)]*22
    for i in range(16):
        for d in range(3):
            for col in range(22):
                force[col]+=Rn[3*i+d][col]*convection[i][d]
                inertia[col]+=Rn[3*i+d][col]*(mass[i]*acceleration_nodes[i][d]+mdot[i]*un[i][d])
    force=[f+p for f,p in zip(force,pressure_force)]
    residual=[a+f for a,f in zip(inertia,force)]
    work=b.dot(z,residual)
    energy_rate=sum(mass[i]*b.dot(un[i],acceleration_nodes[i])+mdot[i]*b.dot(un[i],un[i])/2 for i in range(16))
    b.require(work==energy_rate+adv+strain,'independent exact full energy-rate identity')
    return dict(energy=energy,adv=adv,strain=strain,work=work,force=force,
                momentum=[sum(Rn[3*i+d][col]*mass[i]*un[i][d]for i in range(16)for d in range(3))for col in range(22)],
                mass=mass,flux=flux)


def mpq(value):return mp.mpf(value.numerator)/value.denominator


def verify(cell,order):
    from cell import initial
    q,eta=initial(cell['initial_kind']);q0=list(map(Q,q));eta0=list(map(Q,eta));alpha=list(map(Q,cell['unknowns'][:6]));pressure=list(map(Q,cell['unknowns'][6:]));h=Q(cell['interval'])
    start=point(q0,eta0,alpha,pressure,Q(0));end=point(q0,eta0,alpha,pressure,h)
    sums=[mp.mpf(0)]*3;force=[mp.mpf(0)]*22;transfer={ij:mp.mpf(0)for ij in m.PAIR}
    nodes,weights=np.polynomial.legendre.leggauss(order)
    for index,(node,weight)in enumerate(zip(nodes,weights)):
        time=h*(1+Q(node))/2;w=mpq(h)*mpq(Q(weight))/2
        p=point(q0,eta0,alpha,pressure,time)
        for i,key in enumerate(['adv','strain','work']):sums[i]+=w*mpq(p[key])
        for i,value in enumerate(p['force']):force[i]+=w*mpq(value)
        for ij,value in p['flux'].items():transfer[ij]+=w*mpq(value)
    residual=[mpq(end['momentum'][i]-start['momentum'][i])+force[i]for i in range(22)]
    gcl=[mpq(end['mass'][i]-start['mass'][i])for i in range(16)]
    for (i,j),f in transfer.items():gcl[i]+=f;gcl[j]-=f
    defect=mpq(end['energy']-start['energy'])+sums[0]+sums[1]
    allowance=mpq(Q(cell['fixed_work_allowance']))
    b.require(abs(defect)>allowance*100000,'independent fixed work gate failure')
    b.require(abs(defect-sums[2])<mp.mpf('1e-14'),'independent energy vs weighted residual work')
    return dict(precision_digits=80,quadrature_order=order,actual_binary_interval=str(h),
                energy_old=mp.nstr(mpq(start['energy']),40),energy_new=mp.nstr(mpq(end['energy']),40),
                advection_loss=mp.nstr(sums[0],40),strain_loss=mp.nstr(sums[1],40),
                independent_energy_defect=mp.nstr(defect,40),independent_residual_work=mp.nstr(sums[2],40),
                full_integrated_momentum_residual_max=mp.nstr(max(map(abs,residual)),40),
                finite_local_GCL_defect_max=mp.nstr(max(map(abs,gcl)),40),
                work_gate_pass=False,all_pointwise_strong_divergence_acceleration_GCL_exact=True)


if __name__=='__main__':
    cell=json.load(open(sys.argv[1]));print(json.dumps(verify(cell,int(sys.argv[2])if len(sys.argv)>2 else 16),indent=2))
