"""Reference-only pressure traction integration; no Rheon body-load claim."""
import math


def pressure_wrench(samples, origin):
    """Samples are (position_m, outward_unit_normal, area_m2, pressure_Pa).

    Return force N and moment N m on the enclosed body about origin_m.
    Viscous traction must be assessed separately before total-load acceptance.
    """
    force = [0.0]*3
    moment = [0.0]*3
    if len(origin) != 3 or not all(math.isfinite(x) for x in origin):
        raise ValueError("invalid moment origin")
    for position, normal, area, pressure in samples:
        if (len(position) != 3 or len(normal) != 3 or not all(
                math.isfinite(x) for x in (*position, *normal, area, pressure))
                or area <= 0 or abs(sum(x*x for x in normal)-1) > 1e-12):
            raise ValueError("invalid pressure surface sample")
        traction = [-pressure*area*n for n in normal]
        arm = [x-o for x, o in zip(position, origin)]
        for d in range(3):
            force[d] += traction[d]
            moment[d] += arm[(d+1)%3]*traction[(d+2)%3]-arm[(d+2)%3]*traction[(d+1)%3]
    return force, moment
