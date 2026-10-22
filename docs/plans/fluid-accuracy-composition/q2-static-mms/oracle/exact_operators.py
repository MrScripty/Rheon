"""Prospective CV3 exact-integral operator/trace oracle, candidate-free.

This independently reuses the reviewer's simplex polynomial primitives. It does
not import candidate code, native Gauss samples or rounded native operators.
No calculations run on import. Material/geometry inputs are exact Fractions of
represented native physical primitives; MU is NOT silently replaced by1/10.
"""
from fractions import Fraction as F
import exact_polynomial as ep


def require(condition,message):
    if not condition:raise RuntimeError('CV3 exact-operator guard: '+message)


def zero(rows,cols):return [[F(0) for _ in range(cols)] for _ in range(rows)]


def split(a,b,planes):
    cuts={F(0),F(1)}
    for d in range(2):
        if a[d]!=b[d]:
            for plane in planes[d]:
                t=(plane-a[d])/(b[d]-a[d])
                if 0<t<1:cuts.add(t)
    points=[tuple(a[d]+t*(b[d]-a[d]) for d in range(2)) for t in sorted(cuts)]
    return list(zip(points,points[1:]))


def hat(node,point,h):
    factors=[]
    for d,coordinate in enumerate((ep.X,ep.Y)):
        center=node['point'][d]
        if point[d]<=center-h or point[d]>=center+h:return {}
        if point[d]<=center:factors.append(ep.scale(ep.add(coordinate,ep.constant(h-center)),1/h))
        else:factors.append(ep.scale(ep.add(ep.constant(h+center),ep.scale(coordinate,-1)),1/h))
    return ep.multiply(*factors)


def interval(source,axis,plane,lower,upper):
    # Halfplane inequality in one scalar tangent coordinate. This is independent
    # of native endpoint crossing/line-cache implementation.
    for a,b in zip(source,source[1:]+source[:1]):
        ex,ey=b[0]-a[0],b[1]-a[1]
        alpha,beta=(ex,-ex*a[1]-ey*(plane-a[0])) if axis==0 else (-ey,ex*(plane-a[1])+ey*a[0])
        if alpha>0:lower=max(lower,-beta/alpha)
        elif alpha<0:upper=min(upper,-beta/alpha)
        elif beta<0:return None
    if lower>=upper:return None
    return ((plane,lower),(plane,upper)) if axis==0 else ((upper,plane),(lower,plane))


def build(source,n,origin,extent,rho,mu):
    source=[tuple(map(F,p)) for p in source];origin=tuple(map(F,origin));extent=F(extent)
    rho,mu=F(rho),F(mu)
    require(isinstance(n,int) and 2<=n<=16 and extent>0 and rho>0 and mu>0,'bounded shape/material')
    require(len(source)>=3 and len(set(source))==len(source) and ep.signed_twice_area(source)>0,'unique CCW source')
    for a,b in zip(source,source[1:]+source[:1]):
        require(all(ep.cross((b[0]-a[0],b[1]-a[1]),(p[0]-a[0],p[1]-a[1]))>=0 for p in source),'convex represented source')
    h=extent/n
    require(all(origin[d]+h/2<=p[d]<=origin[d]+extent-h/2 for p in source for d in range(2)), 'complete component nodal hulls')
    nodes=[];lookup={}
    for axis in range(2):
        counts=(n+1,n) if axis==0 else (n,n+1)
        shift=(F(0),F(1,2)) if axis==0 else (F(1,2),F(0))
        for j in range(counts[1]):
            for i in range(counts[0]):
                node={'global':len(nodes),'axis':axis,'face':i+counts[0]*j,'i':i,'j':j,
                      'point':tuple(origin[d]+((i,j)[d]+shift[d])*h for d in range(2))}
                lookup[axis,i,j]=node;nodes.append(node)
    planes=[[origin[d]+i*h/2 for i in range(2*n+1)] for d in range(2)]
    patches=[];flags=set();volumes=[F(0)]*(n*n)
    for j in range(2*n):
        for i in range(2*n):
            lo=(planes[0][i],planes[1][j]);hi=(planes[0][i+1],planes[1][j+1])
            poly=ep.clip_box(source,lo,hi);area=ep.signed_twice_area(poly)/2
            require(area>=0,'nonnegative exact common cut')
            local=[];moments={};cell=i//2+n*(j//2)
            if area:
                require(i>0 and j>0 and i+1<2*n and j+1<2*n,'positive region has complete basis halo')
                for axis in range(2):
                    ix,iy=(i//2,(j-1)//2) if axis==0 else ((i-1)//2,j//2)
                    shift=(F(0),F(1,2)) if axis==0 else (F(1,2),F(0))
                    elo=(origin[0]+(ix+shift[0])*h,origin[1]+(iy+shift[1])*h);ehi=tuple(x+h for x in elo)
                    for dj in range(2):
                        for di in range(2):
                            node=lookup[axis,ix+di,iy+dj]
                            phi=ep.q1_on_element(node['point'],elo,ehi)
                            local.append((node,phi,ep.derivative(phi,0),ep.derivative(phi,1)))
                            flags.add(node['global'])
                moments={(a,b):ep.polygon_integral({(a,b):F(1)},poly) for a in range(5) for b in range(5-a)}
                volumes[cell]+=area
            else:poly=[]
            patches.append({'i':i,'j':j,'cell':cell,'lo':lo,'hi':hi,'poly':poly,'area':area,'local':local,'moments':moments})
    active=sorted(flags);amap={g:i for i,g in enumerate(active)}
    cells=[c for c,a in enumerate(volumes) if a>0];cmap={c:i for i,c in enumerate(cells)}
    nv,nc=len(active),len(cells);require(nv>0 and nc>0,'positive active/pressure spaces')
    require(nv<=544 and nc<=256,'declared bounded dimensions')
    M,K,B=zero(nv,nv),zero(nv,nv),zero(nc,nv)
    H1=zero(nv,nv)
    G,moment=zero(nv,3),zero(nv,3)
    def integral(poly,p):return sum((v*p['moments'][e] for e,v in poly.items()),F(0))
    for p in patches:
        if not p['area']:continue
        c=cmap[p['cell']]
        for node,phi,dx,dy in p['local']:
            i=amap[node['global']];axis=node['axis']
            B[c][i]+=integral(dx if axis==0 else dy,p)
            if axis==0:
                G[i][0]+=integral(dx,p);G[i][1]+=integral(dy,p)
                moment[i][0]+=rho*integral(phi,p);moment[i][2]-=rho*integral(ep.multiply(ep.Y,phi),p)
                strain=(dx,{},dy)
            else:
                G[i][1]+=integral(dx,p);G[i][2]+=integral(dy,p)
                moment[i][1]+=rho*integral(phi,p);moment[i][2]+=rho*integral(ep.multiply(ep.X,phi),p)
                strain=({},dy,dx)
            for node2,phi2,dx2,dy2 in p['local']:
                k=amap[node2['global']];axis2=node2['axis']
                if axis==axis2:
                    M[i][k]+=rho*integral(ep.multiply(phi,phi2),p)
                    H1[i][k]+=integral(ep.add(ep.multiply(phi,phi2),ep.multiply(dx,dx2),ep.multiply(dy,dy2)),p)
                s2=(dx2,{},dy2) if axis2==0 else ({},dy2,dx2)
                K[i][k]+=mu*integral(ep.add(*(ep.scale(ep.multiply(strain[d],s2[d]),2 if d<2 else 1) for d in range(3))),p)
    TG,PF,GF=zero(nv,3),zero(nc,nv),zero(nc,nv);faces={};segments=[]
    def add(kind,owner,axis,edge,a,b):
        for a,b in split(a,b,planes):
            midpoint=tuple((a[d]+b[d])/2 for d in range(2));normal=(b[1]-a[1],a[0]-b[0])
            integrals=[ep.segment_integral_parameter(hat(nodes[g],midpoint,h),a,b) for g in active]
            flux=[v*normal[nodes[g]['axis']] for g,v in zip(active,integrals)]
            if kind=='global':
                for i,g in enumerate(active):
                    if nodes[g]['axis']==0:
                        TG[i][0]+=integrals[i]*normal[0];TG[i][1]+=integrals[i]*normal[1]
                    else:
                        TG[i][1]+=integrals[i]*normal[0];TG[i][2]+=integrals[i]*normal[1]
            elif kind=='grid':
                row=faces.setdefault((axis,owner),[F(0)]*nv)
                for i,v in enumerate(flux):row[i]+=v
            else:
                for i,v in enumerate(flux):PF[cmap[owner]][i]+=v
            segments.append({'kind':kind,'owner':owner,'axis':axis,'edge':edge,'a':a,'b':b,'basis_integrals':integrals,'normal':normal})
    for edge,(a,b) in enumerate(zip(source,source[1:]+source[:1])):add('global',edge,2,edge,a,b)
    for axis in range(2):
        for face in range(n*(n+1)):
            normal,tangent=(face%(n+1),face//(n+1)) if axis==0 else (face//n,face%n)
            piece=interval(source,axis,origin[axis]+normal*h,origin[1-axis]+tangent*h,origin[1-axis]+(tangent+1)*h)
            if piece:add('grid',face,axis,None,*piece)
    for cell in cells:
        lo=(origin[0]+(cell%n)*h,origin[1]+(cell//n)*h);hi=tuple(x+h for x in lo)
        for edge,(a,b) in enumerate(zip(source,source[1:]+source[:1])):
            piece=ep.clip_segment(a,b,lo,hi)
            if piece:add('cell',cell,2,edge,*piece)
    for c,cell in enumerate(cells):
        i,j=cell%n,cell//n
        for axis,face,sign in ((0,i+(n+1)*j,-1),(0,i+1+(n+1)*j,1),(1,i+n*j,-1),(1,i+n*(j+1),1)):
            for k,v in enumerate(faces.get((axis,face),[F(0)]*nv)):GF[c][k]+=sign*v
    require(TG==G,'exact source-stress/volume derivative identity')
    require(all(B[c][i]==GF[c][i]+PF[c][i] for c in range(nc) for i in range(nv)),'exact each-cell physical flux bridge')
    require(M==list(map(list,zip(*M))) and K==list(map(list,zip(*K))), 'exact symmetric Gram blocks')
    require(all(M[i][i]>0 for i in range(nv)), 'positive full mass diagonal')
    rank,pivots=ep.exact_rank(B);require(rank==nc,'exact full pressure rank without gauge')
    old_dual=[]
    for g in active:
        point=nodes[g]['point']
        old_dual.append(ep.polygon_integral(ep.constant(1),ep.clip_box(source,
            tuple(x-h/2 for x in point),tuple(x+h/2 for x in point))))
    return {'n':n,'h':h,'origin':origin,'rho':rho,'mu':mu,'source':source,'nodes':nodes,'active':active,
            'cells':cells,'M':M,'K':K,'B':B,'H1':H1,'gradient':G,'trace_gradient':TG,'moment':moment,
            'grid_flux':GF,'physical_flux':PF,'segments':segments,'volumes':volumes,'old_dual':old_dual,
            'patches':patches,'rank':rank,'rank_pivots':pivots}


def system(operators,dt):
    dt=F(dt);require(dt>0,'positive timestep')
    M,K,B=operators['M'],operators['K'],operators['B'];nv=len(M);nc=len(B)
    a=[[M[i][j]/dt+K[i][j] for j in range(nv)] for i in range(nv)]
    return [a[i]+[-B[c][i] for c in range(nc)] for i in range(nv)]+[[-x for x in row]+[F(0)]*nc for row in B]


def load(operators,gradient,pressure):
    require(len(gradient)==4,'four represented affine-gradient primitives')
    ax,ay,bx,by=map(F,gradient);mu=operators['mu'];pressure=F(pressure)
    sigma=(2*mu*ax-pressure,mu*(ay+bx),2*mu*by-pressure)
    return [sum((g[d]*sigma[d] for d in range(3)),F(0)) for g in operators['trace_gradient']]


def rhs(operators,dt,star,traction_load):
    dt=F(dt);M=operators['M'];nv=len(M);nc=len(operators['B'])
    require(dt>0 and len(star)==nv and len(traction_load)==nv,'complete state/load and positive dt')
    return [sum((M[i][j]*F(star[j]) for j in range(nv)),F(0))/dt+F(traction_load[i]) for i in range(nv)]+[F(0)]*nc
