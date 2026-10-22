"""Prospective same-polygon phase comparison on the exact union partition."""
from fractions import Fraction as F
import exact_polynomial as ep
from continuum import reconstruction
from physical_norms import metrics_reconstructed
from validated import require


def common_reconstruction(op_a, ua, pa, op_b, ub, pb, budget):
    require(op_a['source'] == op_b['source'] and op_a['n'] == op_b['n'], 'same world source/level cross-phase')
    a,b = reconstruction(op_a,ua,pa),reconstruction(op_b,ub,pb)
    result = []
    for ra in a:
        budget.check()
        for rb in b:
            lo = tuple(max(ra['patch']['lo'][d],rb['patch']['lo'][d]) for d in range(2))
            hi = tuple(min(ra['patch']['hi'][d],rb['patch']['hi'][d]) for d in range(2))
            if any(lo[d] >= hi[d] for d in range(2)):
                continue
            poly = ep.clip_box(op_a['source'],lo,hi); area = ep.signed_twice_area(poly)/2
            require(area >= 0, 'nonnegative exact cross-phase common domain')
            if not area:
                continue
            result.append({'patch':{'lo':lo,'hi':hi,'poly':poly,'area':area},
                           'u':tuple(ep.add(x,ep.scale(y,-1)) for x,y in zip(ra['u'],rb['u'])),
                           'p':ep.add(ra['p'],ep.scale(rb['p'],-1)),
                           'sigma':tuple(ep.add(x,ep.scale(y,-1)) for x,y in zip(ra['sigma'],rb['sigma']))})
    require(sum((p['patch']['area'] for p in result),F(0)) == sum(op_a['volumes'],F(0)) == sum(op_b['volumes'],F(0)), 'exact complete union-partition coverage')
    return result


def compare(op_a, ua, pa, op_b, ub, pb, analytic_fields, ledger):
    reconstructed = common_reconstruction(op_a,ua,pa,op_b,ub,pb,ledger.budget)
    zero_fields = {'u':({},{}),'p':{},'sigma':({},{},{})}
    result = metrics_reconstructed(op_a,reconstructed,zero_fields,ledger,denominator_fields=analytic_fields)
    result.pop('reconstructed')
    result['common_positive_pieces'] = len(reconstructed)
    result['decision'] = 'same-domain phase diagnostic with intervals; no pass cutoff'
    return result
