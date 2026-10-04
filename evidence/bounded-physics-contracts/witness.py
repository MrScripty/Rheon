"""Small exact-rational witnesses, not solver or floating-point qualification."""
from fractions import Fraction as F
import json


def require(condition, message):
    if not condition:
        raise ValueError(message)


def wall(a, b):
    hit = a / (a - b)
    value = lambda t: (1 - t) * a + t * b
    return hit, value


def viscous_step(mu):
    # E=[-1,1], unit mass, old velocity [1,-1], dt=1/4.
    dt = F(1, 4)
    u = (F(1), F(-1))
    v = tuple(x / (1 + 2 * dt * mu) for x in u)
    strain = -v[0] + v[1]
    operator = (-mu * strain, mu * strain)
    require(all(v[i] + dt * operator[i] == u[i] for i in range(2)),
            'backward-Euler equation')
    dissipation = mu * strain**2
    work = sum((u[i] - v[i]) * v[i] for i in range(2))
    require(work == dt * dissipation, 'derived viscous work')
    before, after = sum(x*x for x in u), sum(x*x for x in v)
    increment = sum((u[i] - v[i])**2 for i in range(2))
    require(before - after == increment + 2 * dt * dissipation, 'energy identity')
    return {'mu': str(mu), 'new_velocity': list(map(str, v)),
            'dissipation': str(dissipation), 'weighted_norm_before': str(before),
            'weighted_norm_after': str(after), 'nonincrease': after <= before}


def main():
    hit, value = wall(F(1), F(-1))
    require(0 < hit < 1 and value(hit) == 0, 'first hit')
    prefix = [F(0), F(1, 4), hit]
    require(all(value(t) >= 0 for t in prefix), 'permitted prefix')
    require(value(F(3, 4)) < 0, 'unclipped continuation crosses the wall')
    no_crossing, _ = wall(F(1), F(1, 2))
    already_solid, _ = wall(F(-1), F(-2))
    require(no_crossing > 1 and already_solid < 0, 'omitted endpoint assumptions')
    positive, negative = viscous_step(F(2)), viscous_step(F(-1, 2))
    require(positive['nonincrease'] and not negative['nonincrease'], 'viscosity sign')
    # Nonnegative weights alone do not constrain an arbitrary returned velocity.
    arbitrary_before, arbitrary_after = F(0), F(2)
    require(arbitrary_after > arbitrary_before, 'omitted update equation')
    require(F(1) + F(1, 4) * F(4) != 0, 'arbitrary candidate violates backward Euler')
    print(json.dumps({'wall': {'hit': str(hit), 'prefix_values': [str(value(t)) for t in prefix],
          'unclipped_value_at_3_over_4': str(value(F(3, 4))),
          'same_side_endpoint_hit': str(no_crossing), 'solid_start_hit': str(already_solid)},
          'nonnegative_viscosity': positive, 'negative_viscosity_counterexample': negative,
          'omitted_equation_counterexample': {'old_velocity': ['0', '0'],
              'arbitrary_new_velocity': ['1', '-1'], 'mu': '2', 'dt': '1/4',
              'weighted_norm_before': str(arbitrary_before), 'weighted_norm_after': str(arbitrary_after)}},
          indent=2))


if __name__ == '__main__':
    main()
