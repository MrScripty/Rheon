"""Coupled moving geometry/divergence/momentum, bounded two-column research."""
from fractions import Fraction as Q
from pathlib import Path
import importlib.util
import sys
import numpy as np
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('frozen_pressure_path',ROOT/'evidence/fitted-pressure-path/reference.py')
old=importlib.util.module_from_spec(spec);sys.modules[spec.name]=old;spec.loader.exec_module(old)
base=old.base;require=old.require
INITIAL_CAP=old.INITIAL_CAP
def geometry(cap, cap_velocity):
    require(len(cap) == len(cap_velocity) == 3, 'two-column shape')
    require(cap[0][1] > 0 and cap[1][1] > 0 and cap[0][0] < cap[1][0] < cap[2][0], 'positive ordered cap')
    require(cap_velocity[0] == cap_velocity[2], 'periodic cap velocity')
    points = [(base.Dual(Q(i, 2)), base.Dual(Q(0))) for i in range(3)]
    points += [tuple(base.Dual(x, u) for x, u in zip(p, v)) for p, v in zip(cap, cap_velocity)]
    macro = [(0, 1, 4), (0, 4, 3), (1, 2, 5), (1, 5, 4)]
    centers = []
    for tri in macro:
        centers.append(len(points))
        points.append(base.scale(base.add(base.add(points[tri[0]], points[tri[1]]), points[tri[2]]), Q(1, 3)))
    adjacent = {}
    for t, tri in enumerate(macro):
        for k in range(3):
            adjacent.setdefault(tuple(sorted((tri[k], tri[(k + 1) % 3]))), []).append(t)
    left, right = (0, 3), (2, 5)
    adjacent[left].append(adjacent[right][0])
    adjacent[right].append(adjacent[left][0])
    edge_nodes, constraints = {}, {}
    for edge, owners in sorted(adjacent.items()):
        a, b = (points[i] for i in edge)
        if len(owners) == 2:
            p, q = (points[centers[i]] for i in owners)
            if edge == left:
                q = (q[0] - 1, q[1])
            elif edge == right:
                q = (q[0] + 1, q[1])
            ab, pq = base.sub(b, a), base.sub(q, p)
            fraction = base.cross(base.sub(p, a), pq) / base.cross(ab, pq)
            require(0 < fraction.v < 1, 'physical PS intersection inside edge')
            pos = base.add(a, base.scale(ab, fraction))
        else:
            pos = base.scale(base.add(a, b), Q(1, 2))
        edge_nodes[edge] = len(points)
        points.append(pos)
        if len(owners) == 1 and (all(i < 3 for i in edge) or all(3 <= i < 6 for i in edge)):
            constraints[edge_nodes[edge]] = edge
    micro = []
    for t, tri in enumerate(macro):
        for k in range(3):
            a, b = tri[k], tri[(k + 1) % 3]
            e, c = edge_nodes[tuple(sorted((a, b)))], centers[t]
            micro.extend([(a, e, c), (e, b, c)])
    representative = list(range(len(points)))
    representative[2], representative[5] = 0, 3
    representative[edge_nodes[right]] = edge_nodes[left]
    compact = {i: j for j, i in enumerate(sorted(set(representative)))}
    ids = [compact[i] for i in representative]
    constraints = {ids[i]: tuple(ids[j] for j in ends) for i, ends in constraints.items()}
    bottom = {ids[i] for i in range(3)} | {i for i, ends in constraints.items() if all(j in {ids[k] for k in range(3)} for j in ends)}
    return points, micro, ids, constraints, bottom

INITIAL=old.assemble()
IDS=np.array(INITIAL['ids'],int)
TRI=np.array(INITIAL['micro'],int)
R_RAW=np.array(INITIAL['R'],float).reshape(19,3,22)
R_NODE=np.zeros((16,3,22))
for raw,node in enumerate(IDS):R_NODE[node]=R_RAW[raw]
PIVOT=np.array(INITIAL['pressure_columns'],int)
FREE=np.array([j for j in range(22)if j not in PIVOT],int)
ROWS=np.array(base.independent_columns(old.transpose(INITIAL['D'])),int)
require(len(FREE)==6 and len(ROWS)==16,'fixed chart dimensions')
PAIR=sorted({tuple(sorted((int(IDS[tri[i]]),int(IDS[tri[(i+1)%3]]))))for tri in TRI for i in range(3)if IDS[tri[i]]!=IDS[tri[(i+1)%3]]})
PAIR_I=np.array([i for i,j in PAIR]);PAIR_J=np.array([j for i,j in PAIR])
PIECE_PAIR=[];PIECE_SIGN=[]
for tri in TRI:
 for i in range(3):
  u,v=int(IDS[tri[i]]),int(IDS[tri[(i+1)%3]])
  require(u!=v,'no self-pair in fixture')
  PIECE_PAIR.append(PAIR.index((min(u,v),max(u,v))));PIECE_SIGN.append(1 if u<v else -1)
PIECE_PAIR=np.array(PIECE_PAIR);PIECE_SIGN=np.array(PIECE_SIGN)


def cap_from_x(x):
 return [(x[0],x[1]),(x[2],x[3]),(x[0]+1,x[1])]


def cap_motion(value):
 v=np.einsum('ndj,j->nd',R_RAW,value)
 return tuple((v[i,0],v[i,1])for i in [3,4,5])


def spatial(x,xdot=None):
 velocity=[(0.,0.)]*3 if xdot is None else [(xdot[0],xdot[1]),(xdot[2],xdot[3]),(xdot[0],xdot[1])]
 points,_,_,_,_=geometry(cap_from_x(x),velocity)
 pos=np.array([[v.v for v in p]for p in points],float)
 motion=np.array([[v.d for v in p]for p in points],float)
 p=pos[TRI];w=motion[TRI]
 ab,ac=p[:,1]-p[:,0],p[:,2]-p[:,0]
 dab,dac=w[:,1]-w[:,0],w[:,2]-w[:,0]
 a2=ab[:,0]*ac[:,1]-ab[:,1]*ac[:,0]
 a2dot=dab[:,0]*ac[:,1]-dab[:,1]*ac[:,0]+ab[:,0]*dac[:,1]-ab[:,1]*dac[:,0]
 require(np.all(a2>0),'positive actual microareas')
 grad=np.empty((24,3,2));gdot=np.empty_like(grad)
 for i in range(3):
  j,k=(i+1)%3,(i+2)%3
  grad[:,i,0]=(p[:,j,1]-p[:,k,1])/a2
  grad[:,i,1]=(p[:,k,0]-p[:,j,0])/a2
  gdot[:,i,0]=(w[:,j,1]-w[:,k,1])/a2-grad[:,i,0]*a2dot/a2
  gdot[:,i,1]=(w[:,k,0]-w[:,j,0])/a2-grad[:,i,1]*a2dot/a2
 mass=np.bincount(IDS[TRI].reshape(-1),weights=np.repeat(a2/2,3),minlength=16)
 mdot=np.bincount(IDS[TRI].reshape(-1),weights=np.repeat(a2dot/2,3),minlength=16)
 rt=R_RAW[TRI]
 exx=np.einsum('ti,tij->tj',grad[:,:,0],rt[:,:,0,:]);eyy=np.einsum('ti,tij->tj',grad[:,:,1],rt[:,:,1,:])
 exy=np.einsum('ti,tij->tj',grad[:,:,1],rt[:,:,0,:])+np.einsum('ti,tij->tj',grad[:,:,0],rt[:,:,1,:])
 D=exx+eyy
 Ddot=np.einsum('ti,tij->tj',gdot[:,:,0],rt[:,:,0,:])+np.einsum('ti,tij->tj',gdot[:,:,1],rt[:,:,1,:])
 area=a2/2;pressure=D[:,PIVOT];B=-np.einsum('tp,t,tj->pj',pressure,area,D)
 M=np.einsum('n,ndj,ndk->jk',mass,R_NODE,R_NODE)
 K=.05*(np.einsum('t,tj,tk->jk',area,2*exx,exx)+np.einsum('t,tj,tk->jk',area,2*eyy,eyy)+np.einsum('t,tj,tk->jk',area,exy,exy))
 H=np.zeros((22,6));H[FREE]=np.eye(6)
 block=D[np.ix_(ROWS,PIVOT)]
 H[PIVOT]=-np.linalg.solve(block,D[np.ix_(ROWS,FREE)])
 require(np.max(np.abs(D@H))<=1e-11,'full constant-rank chart residual')
 require(np.all(mass>0)and np.all(np.isfinite(H)),'positive masses / finite chart')
 return dict(pos=pos,motion=motion,area=area,mass=mass,mdot=mdot,D=D,Ddot=Ddot,Q=pressure,B=B,M=M,K=K,H=H,
             chart_block=block)


def flux(spatial,z):
 u=np.einsum('ndj,j->nd',R_NODE,z)
 uraw=u[IDS][TRI][:,:,:2];w=spatial['motion'][TRI]
 pos=spatial['pos'][TRI];center=pos.mean(axis=1)
 pieces=[]
 for i in range(3):
  j,k=(i+1)%3,(i+2)%3
  segment=center-(pos[:,i]+pos[:,j])/2
  relative=(5/12)*(uraw[:,i]-w[:,i])+(5/12)*(uraw[:,j]-w[:,j])+(1/6)*(uraw[:,k]-w[:,k])
  pieces.append(3*(relative[:,0]*segment[:,1]-relative[:,1]*segment[:,0]))
 f=np.bincount(PIECE_PAIR,weights=np.array(pieces).T.reshape(-1)*PIECE_SIGN,minlength=len(PAIR))
 donor=np.where((f>=0)[:,None],u[PAIR_I],u[PAIR_J]);route=f[:,None]*donor
 convection=np.zeros((16,3));np.add.at(convection,PAIR_I,route);np.add.at(convection,PAIR_J,-route)
 net=np.zeros(16);np.add.at(net,PAIR_I,f);np.add.at(net,PAIR_J,-f)
 diss=.5*np.sum(np.abs(f)*np.sum((u[PAIR_I]-u[PAIR_J])**2,axis=1))
 return dict(flux=f,convection=convection,div_flux=net,dissipation=diss,velocity=u)


def evaluate(state):
 x,eta=state[:4],state[4:]
 s=spatial(x);z=s['H']@eta
 v=cap_motion(z);xdot=np.array([*v[0],*v[1]])
 s=spatial(x,xdot);f=flux(s,z)
 g=np.einsum('ndj,nd->j',R_NODE,s['mdot'][:,None]*f['velocity']+f['convection'])+s['K']@z
 saddle=np.block([[s['M'],s['B'].T],[s['B'],np.zeros((16,16))]])
 target=s['Q'].T@(s['area']*(s['Ddot']@z))
 solution=np.linalg.solve(saddle,np.r_[-g,target]);acceleration,pressure=solution[:22],solution[22:]
 strong=s['D']@acceleration+s['Ddot']@z
 momentum=s['M']@acceleration+s['B'].T@pressure+g
 chart_motion=np.zeros(22);chart_motion[PIVOT]=-np.linalg.solve(s['chart_block'],(s['Ddot']@z)[ROWS])
 etadot=acceleration[FREE]
 require(np.max(np.abs(s['D']@z))<=1e-11,'full strong velocity constraint')
 require(np.max(np.abs(strong))<=1e-11,'full strong acceleration compatibility')
 require(np.linalg.norm(momentum)<=1e-11,'true coupled momentum residual')
 require(np.max(np.abs(acceleration-s['H']@etadot-chart_motion))<=1e-11,'full chart chain rule')
 return np.r_[xdot,etadot],dict(spatial=s,z=z,pressure=pressure,acceleration=acceleration,flux=f,
   momentum_residual=momentum,strong_acceleration_residual=strong,chart_motion=chart_motion)


def initial_state():
 z=np.array(old.force_solve()['old'],float)
 return np.r_[0.,1.,.5,1.25,z[FREE]]


if __name__=='__main__':
 import json
 derivative,result=evaluate(initial_state())
 print(json.dumps(dict(state=initial_state().tolist(),derivative=derivative.tolist(),pressure=result['pressure'].tolist(),
    pressure_max=float(np.max(np.abs(result['spatial']['Q']@result['pressure']))),
    max_true_momentum_residual=float(np.max(np.abs(result['momentum_residual']))),
    max_full_strong_acceleration_residual=float(np.max(np.abs(result['strong_acceleration_residual']))),
    max_moving_constraint_term=float(np.max(np.abs(result['spatial']['Ddot']@result['z']))),
    faces=len(PAIR)),indent=2))

# The independently reviewed chart fixes three cap velocities and three selected
# reduced coefficients. It preserves all full divergence rows without imposing
# redundant cap equations on momentum.
C_CAP=np.array([R_RAW[3,0],R_RAW[4,0],R_RAW[3,1]])
SELECT=np.array([0,1,4],int)
CHART_RHS=np.vstack([np.zeros((16,6)),np.c_[np.eye(3),np.zeros((3,3))],np.c_[np.zeros((3,3)),np.eye(3)]])


def cap_chart(s):
 L=np.vstack([s['D'][ROWS],C_CAP,np.eye(22)[SELECT]])
 J=np.linalg.solve(L,CHART_RHS)
 require(np.max(np.abs(s['D']@J))<=1e-11,'all strong rows in cap chart')
 require(np.max(np.abs(C_CAP@J-CHART_RHS[16:19]))<=1e-11,'material cap lift')
 require(np.max(np.abs((R_RAW[4,1]+R_RAW[3,1])@J))<=1e-11,'fourth material cap velocity')
 return J,L


def q_to_x(q):return np.array([q[0],q[2],q[1],2.25-q[2]])


def cell_point(q0,eta0,alpha,time,pressure):
 eta=eta0+time*alpha
 q=q0+time*eta0[:3]+(time*time/2)*alpha[:3]
 qdot=eta[:3];x=q_to_x(q);xdot=np.array([qdot[0],qdot[2],qdot[1],-qdot[2]])
 s=spatial(x,xdot);J,L=cap_chart(s);z=J@eta
 Ldot=np.vstack([s['Ddot'][ROWS],np.zeros((6,22))])
 acceleration=J@alpha-np.linalg.solve(L,Ldot@z)
 f=flux(s,z)
 g=np.einsum('ndj,nd->j',R_NODE,s['mdot'][:,None]*f['velocity']+f['convection'])+s['K']@z
 residual=s['M']@acceleration+g+s['B'].T@pressure
 return dict(q=q,qdot=qdot,eta=eta,spatial=s,z=z,acceleration=acceleration,pressure=pressure,flux=f,
             momentum_residual=residual,strong_acceleration_residual=s['D']@acceleration+s['Ddot']@z,
             material_cap_residual=C_CAP@z-qdot)
