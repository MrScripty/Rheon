"""Bounded exact-rational research assembly; no native solver or finite motion step.

The moving-mesh witness differentiates the actual Powell–Sabin construction.
The separate finite transport witness uses a linearized mass change explicitly.
"""
from dataclasses import dataclass
from fractions import Fraction as Q
import json
import math
from pathlib import Path
import sys


def require(ok, message):
    if not ok:
        raise ValueError(message)


@dataclass(frozen=True)
class Dual:
    v: Q
    d: Q = Q(0)
    def __add__(self, other):
        other = lift(other)
        return Dual(self.v + other.v, self.d + other.d)
    __radd__ = __add__
    def __neg__(self):
        return Dual(-self.v, -self.d)
    def __sub__(self, other):
        return self + -lift(other)
    def __rsub__(self, other):
        return lift(other) + -self
    def __mul__(self, other):
        other = lift(other)
        return Dual(self.v * other.v, self.d * other.v + self.v * other.d)
    __rmul__ = __mul__
    def __truediv__(self, other):
        other = lift(other)
        require(other.v != 0, "degenerate geometry division")
        return Dual(self.v / other.v, (self.d * other.v - self.v * other.d) / other.v**2)


def lift(value):
    return value if isinstance(value, Dual) else Dual(Q(value))


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def scale(a, s):
    return tuple(x * s for x in a)


def cross(a, b):
    return a[0] * b[1] - a[1] * b[0]


def mesh(speed=Q(1, 4)):
    heights = [Q(1), Q(5, 4), Q(1), Q(3, 2), Q(1)]
    points = [(Dual(Q(i, 4)), Dual(Q(0))) for i in range(5)]
    points += [(Dual(Q(i, 4), speed), Dual(h)) for i, h in enumerate(heights)]
    macro = []
    for i in range(4):
        macro.extend([(i, i + 1, i + 6), (i, i + 6, i + 5)])
    centers = []
    for tri in macro:
        centers.append(len(points))
        points.append(scale(add(add(points[tri[0]], points[tri[1]]), points[tri[2]]), Q(1, 3)))
    adjacent = {}
    for t, tri in enumerate(macro):
        for k in range(3):
            edge = tuple(sorted((tri[k], tri[(k + 1) % 3])))
            adjacent.setdefault(edge, []).append(t)
    edge_nodes = {}
    boundary_constraints = {}
    for edge, owners in sorted(adjacent.items()):
        a, b = (points[i] for i in edge)
        if len(owners) == 2:
            p, q = (points[centers[i]] for i in owners)
            ab, pq = sub(b, a), sub(q, p)
            s = cross(sub(p, a), pq) / cross(ab, pq)
            require(0 < s.v < 1, "Powell–Sabin edge intersection outside edge")
            pos = add(a, scale(ab, s))
        else:
            pos = scale(add(a, b), Q(1, 2))
        node = len(points)
        points.append(pos)
        edge_nodes[edge] = node
        if len(owners) == 1 and (all(i < 5 for i in edge) or all(5 <= i < 10 for i in edge)):
            boundary_constraints[node] = edge
    micro = []
    for t, tri in enumerate(macro):
        for k in range(3):
            a, b = tri[k], tri[(k + 1) % 3]
            e, c = edge_nodes[tuple(sorted((a, b)))], centers[t]
            micro.extend([(a, e, c), (e, b, c)])
    representative = list(range(len(points)))
    representative[4] = 0
    representative[9] = 5
    representative[edge_nodes[(4, 9)]] = edge_nodes[(0, 5)]
    unique = sorted(set(representative))
    compact = {i: j for j, i in enumerate(unique)}
    ids = [compact[i] for i in representative]
    constraints = {ids[i]: tuple(ids[j] for j in ends) for i, ends in boundary_constraints.items()}
    bottom = {ids[i] for i in range(5)} | {ids[i] for i in boundary_constraints if all(j < 5 for j in boundary_constraints[i])}
    return points, micro, ids, constraints, bottom


def gradients(points, tri):
    p = [points[i] for i in tri]
    twice = cross(sub(p[1], p[0]), sub(p[2], p[0]))
    require(twice.v > 0, "inverted triangle")
    grad = []
    for i in range(3):
        j, k = (i + 1) % 3, (i + 2) % 3
        grad.append(((p[j][1].v - p[k][1].v) / twice.v,
                     (p[k][0].v - p[j][0].v) / twice.v))
    return twice * Q(1, 2), grad


def zeros(n, m):
    return [[Q(0) for _ in range(m)] for _ in range(n)]


def mv(a, x):
    return [sum((v * w for v, w in zip(row, x)), Q(0)) for row in a]


def dot(a, b):
    return sum((v * w for v, w in zip(a, b)), Q(0))


def solve(a, rhs):
    """Exact dense elimination for this fixed small research fixture only."""
    n = len(a)
    require(n <= 128 and len(rhs) == n, "research solve bound")
    b = [row[:] + [v] for row, v in zip(a, rhs)]
    for j in range(n):
        pivot = next((i for i in range(j, n) if b[i][j]), None)
        require(pivot is not None, "singular fixture matrix")
        b[j], b[pivot] = b[pivot], b[j]
        s = b[j][j]
        b[j] = [v / s for v in b[j]]
        for i in range(j + 1, n):
            s = b[i][j]
            if s:
                b[i] = [v - s * w for v, w in zip(b[i], b[j])]
    x = [Q(0)] * n
    for i in reversed(range(n)):
        x[i] = b[i][-1] - dot(b[i][i + 1:n], x[i + 1:])
    return x


def independent_columns(a):
    b = [row[:] for row in a]
    pivots, r = [], 0
    for j in range(len(b[0])):
        pivot = next((i for i in range(r, len(b)) if b[i][j]), None)
        if pivot is None:
            continue
        b[r], b[pivot] = b[pivot], b[r]
        s = b[r][j]
        b[r] = [v / s for v in b[r]]
        for i in range(r + 1, len(b)):
            s = b[i][j]
            if s:
                b[i] = [v - s * w for v, w in zip(b[i], b[r])]
        pivots.append(j)
        r += 1
        if r == len(b):
            break
    return pivots


def scalar_embedding(n, constraints, excluded=()):
    free = [i for i in range(n) if i not in constraints and i not in excluded]
    r = zeros(n, len(free))
    for j, i in enumerate(free):
        r[i][j] = Q(1)
    for i, ends in constraints.items():
        for j in range(len(free)):
            r[i][j] = (r[ends[0]][j] + r[ends[1]][j]) / 2
    return r


def operators(points, micro, rho=Q(3), mu=Q(1, 2)):
    n = len(points)
    mass, rate = [Q(0)] * n, [Q(0)] * n
    k = zeros(3 * n, 3 * n)
    div = zeros(len(micro), 3 * n)
    areas = []
    for t, tri in enumerate(micro):
        area, grads = gradients(points, tri)
        areas.append(area.v)
        e = zeros(5, 9)
        for j, (gx, gy) in enumerate(grads):
            mass[tri[j]] += rho * area.v / 3
            rate[tri[j]] += rho * area.d / 3
            div[t][3 * tri[j]] = gx
            div[t][3 * tri[j] + 1] = gy
            e[0][3*j] = gx
            e[1][3*j+1] = gy
            e[2][3*j] = gy
            e[2][3*j+1] = gx
            e[3][3*j+2] = gx
            e[4][3*j+2] = gy
        weights = [2, 2, 1, 1, 1]
        for a in range(9):
            ia = 3 * tri[a // 3] + a % 3
            for b in range(9):
                ib = 3 * tri[b // 3] + b % 3
                k[ia][ib] += mu * area.v * sum(weights[s] * e[s][a] * e[s][b] for s in range(5))
    return mass, rate, k, div, areas


def dual_flux(points, micro, ids, rho=Q(3), speed=Q(1, 4)):
    flux = {}
    for tri in micro:
        p = [points[i] for i in tri]
        center = scale(add(add(p[0], p[1]), p[2]), Q(1, 3))
        for a in range(3):
            b, c = (a + 1) % 3, (a + 2) % 3
            midpoint = scale(add(p[a], p[b]), Q(1, 2))
            segment = sub(center, midpoint)
            normal = (segment[1].v, -segment[0].v)
            # Linear velocity integral along this actual moving dual face.
            wm = [Q(5, 12)*p[a][d].d + Q(5, 12)*p[b][d].d + Q(1, 6)*p[c][d].d for d in range(2)]
            f = rho * ((speed - wm[0]) * normal[0] - wm[1] * normal[1])
            i, j = ids[tri[a]], ids[tri[b]]
            if i == j:
                continue
            pair = (min(i, j), max(i, j))
            flux[pair] = flux.get(pair, Q(0)) + (f if i < j else -f)
    return {pair: f for pair, f in flux.items() if f}


def merge(values, ids):
    out = [Q(0)] * (max(ids) + 1)
    for i, value in enumerate(values):
        out[ids[i]] += value
    return out


def flux_divergence(flux, n):
    out = [Q(0)] * n
    for (i, j), f in flux.items():
        out[i] += f
        out[j] -= f
    return out


def patch(points, micro, k, div, areas, mu=Q(1, 2), pressure=Q(1, 3)):
    # Unrestricted local affine patch: prescribed traction, not periodic free flow.
    a, b, c, d, e = Q(1, 2), Q(3, 4), Q(-1, 4), Q(1, 2), Q(1, 4)
    u = [v for x, y in points for v in (a*x.v+b*y.v, c*x.v-a*y.v, d*x.v+e*y.v)]
    require(all(v == 0 for v in mv(div, u)), "affine incompressibility")
    pressure_force = [sum(-pressure * area * row[i] for area, row in zip(areas, div)) for i in range(len(u))]
    sigma = [[2*mu*a-pressure, mu*(b+c), mu*d], [mu*(b+c), -2*mu*a-pressure, mu*e], [mu*d, mu*e, -pressure]]
    edges = {}
    for tri in micro:
        for j in range(3):
            edge = (tri[j], tri[(j+1)%3])
            key = tuple(sorted(edge))
            if key in edges:
                del edges[key]
            else:
                edges[key] = edge
    load = [Q(0)] * len(u)
    for a0, b0 in edges.values():
        segment = sub(points[b0], points[a0])
        normal = [segment[1].v, -segment[0].v, Q(0)]
        traction = mv(sigma, normal)
        for i in (a0, b0):
            for d0 in range(3):
                load[3*i+d0] += traction[d0] / 2
    require([x+y for x,y in zip(mv(k,u),pressure_force)] == load, "affine exact traction patch")
    expected = sum(areas) * mu * (4*a*a + (b+c)**2+d*d+e*e)
    require(dot(u, mv(k,u)) == expected, "full strain energy patch")
    rotation = [v for x,y in points for v in (-y.v, x.v, Q(1,8))]
    require(all(v == 0 for v in mv(k,rotation)), "rigid rotation null strain")
    v = [Q((i*7)%13-6, 11) for i in range(len(u))]
    p = [Q((i*5)%11-5, 7) for i in range(len(div))]
    bt = [sum(-p0*area*row[i] for p0,area,row in zip(p,areas,div)) for i in range(len(u))]
    require(dot(v,bt) == sum(-p0*area*d0 for p0,area,d0 in zip(p,areas,mv(div,v))), "pressure adjoint")
    return {"affine_full_strain_power": str(expected), "traction_patch_max_residual": "0", "rotation_strain_power": "0"}


def witness():
    points, micro, ids, constraints, bottom = mesh()
    raw_mass, raw_rate, k, div, areas = operators(points, micro)
    mass, rate = merge(raw_mass, ids), merge(raw_rate, ids)
    n = len(mass)
    flux = dual_flux(points, micro, ids)
    summed = flux_divergence(flux, n)
    require(all(m > 0 for m in mass), "positive nodal masses")
    require(all(dm+f == 0 for dm,f in zip(rate,summed)), "actual moving dual geometric conservation")
    require(sum(rate) == 0 and any(rate) and any(flux.values()), "nontrivial volume-preserving motion")
    # Reduced divergence image with periodicity, bottom impermeability and linear cap trace.
    rx = scalar_embedding(n, constraints)
    ry = scalar_embedding(n, constraints, bottom)
    reduced = zeros(len(micro), len(rx[0])+len(ry[0]))
    for t,row in enumerate(div):
        for old in range(len(points)):
            for j in range(len(rx[0])):
                reduced[t][j] += row[3*old] * rx[ids[old]][j]
            for j in range(len(ry[0])):
                reduced[t][len(rx[0])+j] += row[3*old+1] * ry[ids[old]][j]
    pivots = independent_columns(reduced)
    qbasis = [[row[j] for j in pivots] for row in reduced]
    b = [[-sum(areas[t]*qbasis[t][q]*reduced[t][j] for t in range(len(micro))) for j in range(len(reduced[0]))] for q in range(len(pivots))]
    require(len(independent_columns(b)) == len(pivots), "divergence image pressure has full row rank")
    # V=(0,y,0) is admissible and has divergence one: constant pressure is present.
    rep_y = {ids[i]: p[1].v for i,p in enumerate(points)}
    free_y = [i for i in range(n) if i not in constraints and i not in bottom]
    ycoeff = [Q(0)]*len(rx[0]) + [rep_y[i] for i in free_y]
    require(mv(reduced,ycoeff) == [Q(1)]*len(micro), "constant pressure included, no arbitrary gauge")
    # Nonconstant third component; solve the actual constrained semidiscrete rate.
    free_z = [i for i in range(n) if i not in constraints]
    zcoeff = [rep_y[i]**2 for i in free_z]
    uz = mv(rx,zcoeff)
    velocity = [(Q(1,4),Q(0),z) for z in uz]
    conv = [[Q(0)]*3 for _ in range(n)]
    diss = Q(0)
    for (i,j),f in flux.items():
        diss += abs(f)*sum((velocity[i][d]-velocity[j][d])**2 for d in range(3))/2
        for d in range(3):
            g = f*(velocity[i][d]+velocity[j][d])/2+abs(f)*(velocity[i][d]-velocity[j][d])/2
            conv[i][d] += g
            conv[j][d] -= g
    raw_u = [v for old in range(len(points)) for v in velocity[ids[old]]]
    require(all(v == 0 for v in mv(div,raw_u)), "manufactured semidiscrete divergence")
    ku = mv(k,raw_u)
    merged_kz = merge([ku[3*i+2] for i in range(len(points))],ids)
    mr = [[sum(mass[i]*rx[i][a]*rx[i][b0] for i in range(n)) for b0 in range(len(free_z))] for a in range(len(free_z))]
    rhs = [-sum(rx[i][a]*(rate[i]*uz[i]+conv[i][2]+merged_kz[i]) for i in range(n)) for a in range(len(free_z))]
    zdot = mv(rx, solve(mr,rhs))
    tdot = sum(rate[i]*sum(v*v for v in velocity[i])/2 + mass[i]*uz[i]*zdot[i] for i in range(n))
    strain = dot(raw_u,ku)
    require(tdot + diss + strain == 0, "constrained moving-mass energy law")
    pdotz = sum(rate[i]*uz[i]+mass[i]*zdot[i] for i in range(n))
    require(pdotz == 0, "third momentum conservation including trace reactions")
    # Deliberately distinct finite algebra fixture: masses linearized at this instant.
    tau = Q(1,64)
    new_mass = [m+tau*dm for m,dm in zip(mass,rate)]
    require(all(m > 0 for m in new_mass), "finite algebra positive masses")
    a = zeros(n,n)
    for i in range(n):
        a[i][i] = new_mass[i]
    for (i,j),f0 in flux.items():
        f = tau*f0
        if f > 0:
            a[i][i] += f
            a[j][i] -= f
        else:
            a[j][j] -= f
            a[i][j] += f
    require(mv(a,[Q(1)]*n) == mass, "transport constant field row balance")
    require([sum(a[i][j] for i in range(n)) for j in range(n)] == new_mass, "transport momentum column balance")
    old = [Q((i*7)%19-9,8) for i in range(n)]
    new = solve(a,[m*u for m,u in zip(mass,old)])
    require(dot(new_mass,new) == dot(mass,old), "implicit transport momentum")
    require(min(old) <= min(new) and max(new) <= max(old), "implicit transport convex range")
    loss_time = sum(m*(v-u)**2 for m,u,v in zip(mass,old,new))/2
    loss_flux = sum(abs(tau*f)*(new[i]-new[j])**2 for (i,j),f in flux.items())/2
    old_e, new_e = dot(mass,[u*u/2 for u in old]), dot(new_mass,[u*u/2 for u in new])
    require(new_e-old_e+loss_time+loss_flux == 0, "implicit transport exact changing-mass energy")
    # Same physically shared A carries each component; two extra independent fields.
    for component_old in ([Q(1,4)]*n, [Q((i*5)%7-3,9) for i in range(n)]):
        component_new = solve(a,[m*u for m,u in zip(mass,component_old)])
        require(dot(new_mass,component_new)==dot(mass,component_old), "all-component transport momentum")
        require(min(component_old)<=min(component_new) and max(component_new)<=max(component_old), "all-component convex range")
        energy_change = sum(m*v*v for m,v in zip(new_mass,component_new))/2-sum(m*u*u for m,u in zip(mass,component_old))/2
        temporal = sum(m*(v-u)**2 for m,u,v in zip(mass,component_old,component_new))/2
        mixing = sum(abs(tau*f)*(component_new[i]-component_new[j])**2 for (i,j),f in flux.items())/2
        require(energy_change+temporal+mixing==0, "all-component transport work")
    # Marginal-correct corruption: a circulation changes no node balance.
    corrupted = dict(flux)
    triple = (0,1,2)
    for i,j in zip(triple,triple[1:]+triple[:1]):
        pair = (min(i,j),max(i,j))
        corrupted[pair] = corrupted.get(pair,Q(0)) + (Q(1,101) if i<j else -Q(1,101))
    require(flux_divergence(corrupted,n) == summed and corrupted != dual_flux(points,micro,ids), "geometric provenance negative witness")
    output = {"scope": "exact semidiscrete research assembly; separate finite transport algebra on linearized masses, not a finite moving-surface replay",
        "macro_triangles": 8, "micro_triangles":len(micro), "geometric_nodes":len(points), "periodic_nodes":n,
        "pressure_image_rank":len(pivots), "reduced_xy_unknowns":len(reduced[0]), "third_unknowns":len(free_z),
        "liquid_area":str(sum(areas)), "total_mass_each_component":str(sum(mass)), "min_nodal_mass":str(min(mass)),
        "nonzero_geometric_mass_rates":sum(dm!=0 for dm in rate), "nonzero_dual_flux_pairs":len(flux),
        "geometric_conservation_max_residual":"0", "strong_divergence_max_residual":"0", "pressure_image_row_rank":len(pivots),
        "advection_dissipation":str(diss), "strain_power":str(strain), "kinetic_energy_rate":str(tdot), "third_momentum_rate":"0",
        "finite_algebra_carried_components_checked":3, "finite_algebra_reported_energies":"first component; each of all three separately checked", "finite_algebra_dt":str(tau), "finite_algebra_old_energy":str(old_e), "finite_algebra_new_energy":str(new_e),
        "finite_algebra_time_loss":str(loss_time), "finite_algebra_flux_loss":str(loss_flux),
        "finite_algebra_energy_residual":"0", "marginal_correct_wrong_motion":"rejected by direct geometric face-flux recomputation"}
    output.update(patch(points,micro,k,div,areas))
    output["flat_cap_manufactured"] = flat_cap()
    return output


def flat_cap():
    # Smooth instantaneous BE benchmark. No repeated time advancement implied.
    height, wave, amp, mu, rho, dt = 1.0, math.pi, .1, .5, 3.0, .001
    c = -wave**2/(6+wave**2*height**2)
    f = lambda y: y+c*y**3
    fp = lambda y: 1+3*c*y*y
    fpp = lambda y: 6*c*y
    shear = amp*(fpp(height)+wave*wave*f(height))
    pressure_amplitude = -2*mu*amp*wave*fp(height)
    max_div = max_tangent = max_normal = max_be = 0.0
    for j in range(17):
        x = j/16
        s, co = math.sin(wave*x), math.cos(wave*x)
        for l in range(9):
            y = l/8
            vx, vy = amp*s*fp(y), -amp*wave*co*f(y)
            dux, dvy = amp*wave*co*fp(y), -amp*wave*co*fp(y)
            max_div = max(max_div,abs(dux+dvy))
            lap = (amp*s*(6*c-wave*wave*fp(y)), -amp*wave*co*(fpp(y)-wave*wave*f(y)))
            gp = (2*mu*amp*wave*wave*s*fp(height),0.0)
            predictor = (vx-dt/rho*(mu*lap[0]-gp[0]),vy-dt/rho*(mu*lap[1]-gp[1]))
            max_be = max(max_be,*(abs(rho*(v-u)/dt-mu*la+g) for v,u,la,g in zip((vx,vy),predictor,lap,gp)))
        pi = pressure_amplitude*co
        max_tangent = max(max_tangent,abs(s*shear))
        max_normal = max(max_normal,abs(-pi-2*mu*amp*wave*co*fp(height)))
    require(max_tangent < 1e-14 and max_normal < 1e-14 and max_be < 1e-12, "instantaneous traction/BE analytic benchmark")
    require(abs(pressure_amplitude)>0.1, "zero pressure negative traction witness")
    return {"c":c,"pressure_amplitude":pressure_amplitude,"zero_pressure_normal_traction_error":abs(pressure_amplitude),
            "max_divergence":max_div,"max_tangential_traction":max_tangent,"max_normal_traction":max_normal,
            "derived_predictor_BE_residual":max_be,"scope":"instantaneous flat-cap exact field with derived predictor; not a moving-surface solution"}


if __name__ == "__main__":
    result = witness()
    text = json.dumps(result,indent=2)+"\n"
    if len(sys.argv)>1:
        Path(sys.argv[1]).write_text(text)
    print(text,end="")
