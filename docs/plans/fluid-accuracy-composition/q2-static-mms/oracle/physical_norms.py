"""Prospective physical norm intervals, including oblique traction measure."""
from fractions import Fraction as F
import exact_polynomial as ep
from validated import require, sqrt_interval, ratio_interval, add_radius
from bounded_norms import l1_enclosure, volume_l2_squared, square_polynomial, supremum_bounds
from continuum import reconstruction, volume_items, boundary_pieces


def sum_intervals(items):
    return sum((x[0] for x in items), F(0)), sum((x[1] for x in items), F(0))


def squared_integral_interval(items):
    q = volume_l2_squared([(p, tri) for kind, p, tri in items])
    require(q >= 0, 'nonnegative exact volume squared norm')
    return sqrt_interval(q)


def norm_metrics(op, velocity, pressure, continuum_fields, ledger):
    reconstructed = reconstruction(op, velocity, pressure)
    return metrics_reconstructed(op,reconstructed,continuum_fields,ledger)


def metrics_reconstructed(op, reconstructed, continuum_fields, ledger, denominator_fields=None):
    denominator_fields = continuum_fields if denominator_fields is None else denominator_fields
    volume = sum((op['volumes'][c] for c in op['cells']), F(0))
    require(volume > 0, 'positive physical volume')
    errors, denominators, result = {}, {}, {}
    for key, field in (('velocity', 'u'), ('pressure', 'p')):
        exact = continuum_fields[field]
        items = volume_items(reconstructed, field, exact)
        expected = denominator_fields[field] if field == 'u' else (denominator_fields[field],)
        target_items = [('triangle', expected, tri) for kind, p, tri in items]
        tolerance = volume/F(10**8)
        a1, b1 = l1_enclosure(items, tolerance, ledger), l1_enclosure(target_items, tolerance, ledger)
        a2, b2 = squared_integral_interval(items), squared_integral_interval(target_items)
        errors[key] = {'L1': a1, 'L2': a2}
        denominators[key] = {'L1': b1, 'L2': b2}
        result[key] = {'L1': ratio_interval(a1, b1), 'L2': ratio_interval(a2, b2),
                       'Linfinity_enclosure': supremum_bounds(items)}
    source_lengths = [sqrt_interval(sum(((b[d]-a[d])**2 for d in range(2)), F(0)))
                      for a, b in zip(op['source'], op['source'][1:]+op['source'][:1])]
    perimeter = sum_intervals(source_lengths)
    traction_items = []; l2_parts = []; sup_lower = F(0); sup_upper = F(0)
    normal_items = []; tangent_items = []; normal_l2 = []; tangent_l2 = []; edge_maxima = {}
    for boundary in boundary_pieces(op, reconstructed):
        nx, ny = boundary['normal_measure']; pa, pb = boundary['a'], boundary['b']; weight = boundary['parameter_weight']
        a, b = boundary['source_a'], boundary['source_b']; dx, dy = b[0]-a[0], b[1]-a[1]
        length = source_lengths[boundary['edge']]
        require(length[0] > 0, 'resolved source-edge length')
        dsigma = tuple(ep.add(got, ep.scale(exact, -1)) for got, exact in zip(boundary['piece']['sigma'], continuum_fields['sigma']))
        q = (ep.add(ep.scale(dsigma[0], nx), ep.scale(dsigma[1], ny)),
             ep.add(ep.scale(dsigma[1], nx), ep.scale(dsigma[2], ny)))
        item = ('segment', q, (pa, pb, weight)); traction_items.append(item)
        moment = ep.segment_integral_parameter(square_polynomial(q), pa, pb)*weight
        require(moment >= 0, 'nonnegative physical traction squared moment')
        l2_parts.append((moment/length[1], moment/length[0]))
        local_sup = supremum_bounds([item])
        sup_lower = max(sup_lower, local_sup[0]/length[1]); sup_upper = max(sup_upper, local_sup[1]/length[0])
        edge = str(boundary['edge']); elo,ehi = edge_maxima.get(edge,(F(0),F(0)))
        edge_maxima[edge] = max(elo,local_sup[0]/length[1]),max(ehi,local_sup[1]/length[0])
        # Scalar physical normal/tangent L1: (nraw^T dsigma nraw)/length^2,
        # with ds=length dt. Separate exact numerator then divide by length.
        qn = ep.add(ep.scale(q[0], nx), ep.scale(q[1], ny))
        qt = ep.add(ep.scale(q[0], dx), ep.scale(q[1], dy))
        normal_items.append((('segment', (qn,), (pa, pb, weight)), length))
        tangent_items.append((('segment', (qt,), (pa, pb, weight)), length))
        n2 = ep.segment_integral_parameter(ep.multiply(qn,qn),pa,pb)*weight
        t2 = ep.segment_integral_parameter(ep.multiply(qt,qt),pa,pb)*weight
        require(n2 >= 0 and t2 >= 0, 'nonnegative component traction squared moments')
        normal_l2.append((n2/length[1]**3,n2/length[0]**3))
        tangent_l2.append((t2/length[1]**3,t2/length[0]**3))
    tolerance = perimeter[0]/F(10**8)
    t1 = l1_enclosure(traction_items, tolerance, ledger)
    t2square = sum_intervals(l2_parts)
    t2 = (sqrt_interval(t2square[0])[0], sqrt_interval(t2square[1])[1])
    # q=1: integral/perimeter. q=2: sqrt(integral/perimeter).
    normalized_l1 = ratio_interval(t1, perimeter)
    normalized_l2 = ratio_interval(t2, (sqrt_interval(perimeter[0])[0], sqrt_interval(perimeter[1])[1]))
    result['traction'] = {'L1': normalized_l1, 'L2': normalized_l2, 'Linfinity_enclosure': (sup_lower, sup_upper),
                          'per_source_edge_Linfinity_enclosures':edge_maxima}
    for name, pieces in (('normal_L1', normal_items), ('tangent_L1', tangent_items)):
        bounds = []
        for item, length in pieces:
            # Reserve the same overall normalized-width budget by splitting it
            # proportionally to exact source parameter measure / edge count.
            local_tolerance = tolerance*item[2][2]*length[0]/len(source_lengths)
            interval = l1_enclosure([item], local_tolerance, ledger)
            bounds.append(ratio_interval(interval, length))
        result['traction'][name] = ratio_interval(sum_intervals(bounds), perimeter)
    for name,parts in (('normal_L2',normal_l2),('tangent_L2',tangent_l2)):
        square = ratio_interval(sum_intervals(parts),perimeter)
        result['traction'][name] = sqrt_interval(square[0])[0],sqrt_interval(square[1])[1]
    return {'metrics': result, 'absolute_volume_error_intervals': errors,
            'exact_field_denominator_intervals': denominators, 'perimeter': perimeter,
            'volume': volume, 'reconstructed': reconstructed,
            'subdivision_created': ledger.created, 'subdivision_maximum_depth': ledger.maximum_depth}


def normalized_algebraic_radii(op, coefficient_radius, measured):
    """Safe original-coordinate norm Lipschitz bounds; fixed units all1."""
    epsu = F(coefficient_radius['velocity']); epsp = F(coefficient_radius['pressure'])
    require(epsu >= 0 and epsp >= 0, 'nonnegative separate physical coefficient radii')
    volume = measured['volume']; sqrt2 = sqrt_interval(F(2))[1]
    # Complete component partitions give ||delta u||<=sqrt(2)*eps pointwise.
    ur1 = sqrt2*epsu*volume
    ur2 = sqrt2*epsu*sqrt_interval(volume)[1]
    pr1 = epsp*volume; pr2 = epsp*sqrt_interval(volume)[1]
    v = 4*op['mu']*epsu/op['h']
    traction_radius = sqrt_interval(2*(v+epsp)**2+2*v*v)[1]
    denominators = measured['exact_field_denominator_intervals']
    return {'velocity': {'L1': ur1/denominators['velocity']['L1'][0], 'L2': ur2/denominators['velocity']['L2'][0]},
            'pressure': {'L1': pr1/denominators['pressure']['L1'][0], 'L2': pr2/denominators['pressure']['L2'][0]},
            'traction': {'L1': traction_radius, 'L2': traction_radius}}


def deciding_intervals(measured, radii):
    out = {}
    for field in ('velocity', 'pressure', 'traction'):
        out[field] = {norm: add_radius(measured['metrics'][field][norm], radii[field][norm]) for norm in ('L1', 'L2')}
    return out
