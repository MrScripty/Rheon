"""Independent rational CV1 geometry/polynomial primitives; no candidate imports.

Prepared prospectively. This module performs no work on import. Its use against
native raw evidence waits for source/protocol freeze and execution authorization.
Polynomials map (x power,y power) to fractions; inputs are represented floats.
"""
from fractions import Fraction as F
from math import factorial

def require(condition,message):
    if not condition:
        raise RuntimeError(message)

def represented(value):
    return F.from_float(float(value))

def constant(value):
    value=F(value)
    return {} if value==0 else {(0,0):value}

X={(1,0):F(1)}
Y={(0,1):F(1)}

def add(*polynomials):
    result={}
    for p in polynomials:
        for power,value in p.items():
            result[power]=result.get(power,F(0))+value
    return {power:value for power,value in result.items() if value}

def scale(p,value):
    value=F(value)
    return {power:coefficient*value for power,coefficient in p.items() if coefficient*value}

def multiply(a,b):
    result={}
    for (i,j),x in a.items():
        for (k,l),y in b.items():
            power=i+k,j+l
            result[power]=result.get(power,F(0))+x*y
    return {power:value for power,value in result.items() if value}

def power(p,n):
    require(n>=0,'nonnegative polynomial exponent')
    result=constant(1)
    for _ in range(n):
        result=multiply(result,p)
    return result

def derivative(p,axis):
    require(axis in (0,1),'polynomial axis')
    result={}
    for exponent,value in p.items():
        if exponent[axis]:
            target=list(exponent);target[axis]-=1
            result[tuple(target)]=value*exponent[axis]
    return result

def evaluate(p,point):
    return sum((value*point[0]**i*point[1]**j for (i,j),value in p.items()),F(0))

def substitute(p,x,y):
    result={}
    for (i,j),value in p.items():
        result=add(result,scale(multiply(power(x,i),power(y,j)),value))
    return result

def cross(a,b):
    return a[0]*b[1]-a[1]*b[0]

def signed_twice_area(vertices):
    return sum((cross(a,b) for a,b in zip(vertices,vertices[1:]+vertices[:1])),F(0))

def triangle_integral(p,triangle):
    """Affine map to the standard simplex; analytic factorial monomial rule."""
    a,b,c=triangle
    ab=(b[0]-a[0],b[1]-a[1]);ac=(c[0]-a[0],c[1]-a[1])
    determinant=cross(ab,ac)
    require(determinant>0,'positive exact triangle orientation required')
    x=add(constant(a[0]),scale(X,ab[0]),scale(Y,ac[0]))
    y=add(constant(a[1]),scale(X,ab[1]),scale(Y,ac[1]))
    local=substitute(p,x,y)
    return determinant*sum((value*F(factorial(i)*factorial(j),factorial(i+j+2))
        for (i,j),value in local.items()),F(0))

def polygon_integral(p,vertices):
    if len(vertices)<3:
        return F(0)
    result=F(0)
    for i in range(1,len(vertices)-1):
        triangle=vertices[0],vertices[i],vertices[i+1]
        determinant=cross((triangle[1][0]-triangle[0][0],triangle[1][1]-triangle[0][1]),
            (triangle[2][0]-triangle[0][0],triangle[2][1]-triangle[0][1]))
        require(determinant>=0,'convex CCW fan required')
        if determinant:
            result+=triangle_integral(p,triangle)
    return result

def segment_integral_parameter(p,a,b):
    """Integral from t=0 to1; multiply by oriented normal measure separately."""
    x=add(constant(a[0]),scale(X,b[0]-a[0]))
    y=add(constant(a[1]),scale(X,b[1]-a[1]))
    local=substitute(p,x,y)
    require(all(j==0 for i,j in local),'univariate segment substitution')
    return sum((value/F(i+1) for (i,j),value in local.items()),F(0))

def clip_halfplane(vertices,axis,plane,keep_above):
    if not vertices:
        return []
    result=[]
    for a,b in zip(vertices,vertices[1:]+vertices[:1]):
        ia=a[axis]>=plane if keep_above else a[axis]<=plane
        ib=b[axis]>=plane if keep_above else b[axis]<=plane
        if ia!=ib:
            t=(plane-a[axis])/(b[axis]-a[axis])
            point=tuple(a[d]+t*(b[d]-a[d]) for d in range(2))
            if not result or result[-1]!=point:
                result.append(point)
        if ib and (not result or result[-1]!=b):
            result.append(b)
    if len(result)>1 and result[0]==result[-1]:
        result.pop()
    return result

def clip_box(vertices,lo,hi):
    require(all(lo[d]<hi[d] for d in range(2)),'positive exact box')
    result=list(vertices)
    for axis in range(2):
        result=clip_halfplane(result,axis,lo[axis],True)
        result=clip_halfplane(result,axis,hi[axis],False)
    return result

def clip_segment(a,b,lo,hi):
    lower,upper=F(0),F(1)
    for axis in range(2):
        delta=b[axis]-a[axis]
        if not delta:
            if not lo[axis]<=a[axis]<=hi[axis]:
                return None
        else:
            t0,t1=(lo[axis]-a[axis])/delta,(hi[axis]-a[axis])/delta
            lower=max(lower,min(t0,t1));upper=min(upper,max(t0,t1))
    if upper<=lower:
        return None
    return (tuple(a[d]+lower*(b[d]-a[d]) for d in range(2)),
        tuple(a[d]+upper*(b[d]-a[d]) for d in range(2)))

def q1_on_element(node,element_lo,element_hi):
    """Scalar nodal Q1 polynomial on its own knot rectangle, not a dual box."""
    factors=[]
    for axis,coordinate in enumerate((X,Y)):
        lo,hi=element_lo[axis],element_hi[axis]
        require(lo<hi and node[axis] in (lo,hi),'Q1 corner and positive knot span')
        if node[axis]==lo:
            factor=scale(add(constant(hi),scale(coordinate,-1)),1/(hi-lo))
        else:
            factor=scale(add(coordinate,constant(-lo)),1/(hi-lo))
        factors.append(factor)
    return multiply(*factors)

def exact_rank(matrix):
    """Deterministic rational elimination; zero means exact zero, no tolerance."""
    rows=[[F(value) for value in row] for row in matrix]
    if not rows:
        return 0,[]
    columns=len(rows[0]);require(all(len(row)==columns for row in rows),'rectangular rank input')
    rank=0;pivots=[]
    for column in range(columns):
        pivot=next((i for i in range(rank,len(rows)) if rows[i][column]),None)
        if pivot is None:
            continue
        rows[rank],rows[pivot]=rows[pivot],rows[rank]
        value=rows[rank][column]
        for j in range(column,columns):
            rows[rank][j]/=value
        for i in range(rank+1,len(rows)):
            factor=rows[i][column]
            if factor:
                for j in range(column,columns):
                    rows[i][j]-=factor*rows[rank][j]
        pivots.append(column);rank+=1
        if rank==len(rows):
            break
    return rank,pivots
