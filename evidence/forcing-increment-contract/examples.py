"""Small exact-input contract discriminators, not a Rheon step or solver.

Fractions supply exact algebra. float conversion/operations supply the observed
binary64 examples. No compensated native implementation is tested or assumed.
"""
from fractions import Fraction as Q
import json
import math


def require(condition, message):
    if not condition:
        raise ValueError(message)


def qstr(value):
    value = Q(value)
    return f"{value.numerator}/{value.denominator}"


def negative(name, condition):
    try:
        require(condition, name)
    except ValueError:
        return name
    raise ValueError(f"corruption did not fail: {name}")


def run():
    controls = []
    h = Q(1, 2**54)
    u0 = Q(1)
    latent = u0 + h
    stored = Q(float(latent))
    endpoint_rate = (stored-u0-h)/h
    latent_rate = (latent-u0-h)/h
    t0 = u0*u0/2
    t1 = stored*stored/2
    dbe = (stored-u0)**2/2
    force_work = h*stored
    endpoint_work = stored*h*endpoint_rate
    ledger = t1-t0+dbe-force_work-endpoint_work
    hybrid = t1-t0+dbe-force_work-stored*h*latent_rate
    require(stored == 1 and endpoint_rate == -1 and latent_rate == 0,
            "lost increment endpoint/latent distinction")
    require(ledger == 0 and hybrid == -h, "endpoint ledger binding")
    controls.append(negative("latent residual passed as stored endpoint", endpoint_rate == latent_rate))
    controls.append(negative("latent residual paired with stored energy", hybrid == 0))
    lost = dict(h=qstr(h), u0=qstr(u0), latent=qstr(latent), stored_hex=float(stored).hex(),
                endpoint_rate=qstr(endpoint_rate), latent_rate=qstr(latent_rate),
                stored_ledger=qstr(ledger), hybrid_ledger=qstr(hybrid),
                discarded_momentum=qstr(stored-latent),
                discarded_energy=qstr((stored*stored-latent*latent)/2))

    # Same exact binary inputs, changed sum order; both are scalar residuals.
    threshold = 1e-13
    x = math.nextafter(threshold, math.inf)
    seq = (1.0+x)-1.0
    alternate = (1.0-1.0)+x
    exact = Q(1.0)+Q(x)-Q(1.0)
    require(seq <= threshold < alternate and exact > Q(threshold),
            "same input arithmetic can change acceptance")
    require(math.sqrt(seq*seq) <= threshold < math.sqrt(alternate*alternate),
            "one-component native norm illustration")
    controls.append(negative("ephemeral precision declared behavior neutral", seq == alternate))
    accumulation = dict(inputs_hex=[float(1).hex(), x.hex(), float(-1).hex()],
                        threshold_hex=threshold.hex(), sequential=seq, alternate=alternate,
                        exact=qstr(exact), sequential_pass=True, alternate_pass=False,
                        is_rheon_counterexample=False)

    # A two-column moving constraint. z0 is deliberately off the exact chart.
    defect = Q(1, 64)
    z0 = [Q(1), -Q(1)+defect]
    d0, d1 = [Q(1), Q(1)], [Q(1), Q(2)]
    dk = Q(1, 8)
    absolute = -(z0[0]+dk)/2
    rhs = -sum(a*b for a, b in zip(d1, z0))-dk
    du = rhs/2
    dropped = (-sum((a-b)*z for a, b, z in zip(d1, d0, z0))-dk)/2
    endpoint = [z0[0]+dk, z0[1]+du]
    wrong = [endpoint[0], z0[1]+dropped]
    require(endpoint[1] == absolute and sum(a*b for a, b in zip(d1, endpoint)) == 0,
            "absolute and increment chart equations")
    wrong_constraint = sum(a*b for a, b in zip(d1, wrong))
    require(wrong_constraint == defect, "omitted initial defect quantified")
    start_delta = -sum(a*b for a, b in zip(d0, z0))
    require(start_delta == -defect, "chart start differs from stored start")
    controls.append(negative("omitted accepted constraint defect", wrong_constraint == 0))
    controls.append(negative("off-chart accepted state called continuous chart start", start_delta == 0))
    chart = dict(z0=list(map(qstr, z0)), d0=list(map(qstr, d0)), d1=list(map(qstr, d1)),
                 delta_known=qstr(dk), delta_unknown=qstr(du),
                 absolute_unknown=qstr(absolute), wrong_constraint=qstr(wrong_constraint),
                 chart_start_unknown_delta=qstr(start_delta))

    # One interpolated nodal row, not a full native embedding/counterexample.
    eps = Q(1, 2**52)
    z0_scalar, z1_scalar = Q(1), Q(1)
    old, new = Q(1)+eps, Q(1)
    e0, e1 = z0_scalar-old, z1_scalar-new
    stable = z1_scalar-z0_scalar
    direct = new-old
    wr = z1_scalar*stable
    wn = new*direct
    lhs = new*new/2-old*old/2+(new-old)**2/2
    formula = -e1*direct-(new+e1)*(e1-e0)
    require(stable-direct == e1-e0, "stable/direct embedding identity")
    require(lhs == wn and lhs-wr == formula == -eps, "embedding ledger identity")
    controls.append(negative("stable/direct declared exact for rounded embedding", stable == direct))
    embedding = dict(e0=qstr(e0), e1=qstr(e1), stable=qstr(stable), direct=qstr(direct),
                     nodal_work=qstr(wn), reduced_work=qstr(wr), ledger_defect=qstr(formula))

    # Same mass/velocity endpoints with transport in both directions.
    m0, m1 = [Q(2), Q(3)], [Q(1), Q(4)]
    old, new = [Q(2), Q(-1)], [Q(3), Q(1)]
    fp, fm = Q(2), Q(1)
    net = fp-fm
    gcl = [m1[0]-m0[0]+net, m1[1]-m0[1]-net]
    transported = fp*new[0]-fm*new[1]
    residual = [m1[i]*new[i]-m0[i]*old[i] + (transported if i == 0 else -transported)
                for i in range(2)]
    delta_energy = sum(m1[i]*new[i]**2-m0[i]*old[i]**2 for i in range(2))/2
    be = sum(m0[i]*(new[i]-old[i])**2 for i in range(2))/2
    mix = (fp+fm)*(new[0]-new[1])**2/2
    work = sum(a*b for a, b in zip(new, residual))
    require(gcl == [0, 0] and delta_energy+be+mix == work == 14,
            "directional donor changing-mass work identity")
    wrong_mix = abs(net)*(new[0]-new[1])**2/2
    controls.append(negative("net transfer substituted for directional mixing", delta_energy+be+wrong_mix == work))
    donor = dict(gcl=list(map(qstr, gcl)), plus=qstr(fp), minus=qstr(fm),
                 delta_energy=qstr(delta_energy), be=qstr(be), mixing=qstr(mix),
                 residual=list(map(qstr, residual)), residual_work=qstr(work),
                 wrong_net_only_mixing=qstr(wrong_mix))

    # A different rounded endpoint requires recomputing both adjoint and strain.
    vlatent, vbinary = [Q(1), -Q(1)], [Q(1), -Q(1)+eps]
    pi = Q(3)
    pressure_force = [-pi, -pi]  # B^T*pi for D=[1,1], W=Q=1.
    pressure = sum(v*f for v, f in zip(vbinary, pressure_force))
    require(pressure == -pi*sum(vbinary) == -3*eps,
            "pressure pairing on same stored endpoint")
    # Full positive scalar strain K=g*g^T, g=[-1,1], on a separate tangential field.
    tangent0, tangent1 = [Q(1), Q(1)], [Q(1), Q(1)+eps]
    g = [-Q(1), Q(1)]
    gradient = sum(a*b for a, b in zip(g, tangent1))
    forces = [a*gradient for a in g]
    strain = sum(a*b for a, b in zip(tangent1, forces))
    require(strain == gradient**2 == eps**2, "strain work on same endpoint")
    controls.append(negative("latent divergence claimed for stored endpoint", sum(vbinary) == sum(vlatent)))
    controls.append(negative("stale strain claimed for changed endpoint", strain == sum(a*b for a, b in zip(g, tangent0))**2))
    operators = dict(stored_divergence=qstr(sum(vbinary)), pressure_work=qstr(pressure),
                     strain_work=qstr(strain), stale_latent_pressure_work="0/1", stale_strain_work="0/1")

    qlatent = Q(1)+h
    qbinary = Q(float(qlatent))
    require(qlatent != qbinary == 1, "rounded geometry distinction")
    controls.append(negative("latent mass and rounded geometry conflated", qlatent == qbinary))
    geometry = dict(model="illustrative m(q)=q and K(q)=q, U=1", qlatent=qstr(qlatent),
                    qbinary=qstr(qbinary), latent_mass=qstr(qlatent), stored_mass=qstr(qbinary),
                    mixed_operator_gap=qstr(qlatent-qbinary))

    # Illustrative retained exact low part versus an ephemeral increment reset.
    retained = Q(1)
    ephemeral = Q(1)
    retained_public, ephemeral_public = [], []
    for _ in range(3):
        retained += h
        ephemeral = Q(float(ephemeral+h))
        retained_public.append(float(retained).hex())
        ephemeral_public.append(float(ephemeral).hex())
    save_latent = Q(1)+2*h
    save_high = Q(float(save_latent))
    resumed_with_low = float(save_latent+h)
    resumed_high_only = float(save_high+h)
    require(retained_public[:2] == ephemeral_public[:2] and retained_public[2] != ephemeral_public[2],
            "persistent low part changes later observable endpoint")
    require(resumed_with_low != resumed_high_only, "retained tail is checkpoint state")
    controls.append(negative("omitted persistent tail called restart parity", resumed_with_low == resumed_high_only))
    restart = dict(retained_public_hex=retained_public, ephemeral_public_hex=ephemeral_public,
                   saved_high_hex=float(save_high).hex(), saved_low=qstr(save_latent-save_high),
                   resumed_with_low_hex=resumed_with_low.hex(), resumed_high_only_hex=resumed_high_only.hex(),
                   is_rheon_restart=False)
    require(len(controls) == 11, "all discriminating corruptions exercised")
    return dict(status="PASS_SMALL_EXAMPLES_ONLY", base="f0b81b4a5f76cb706e645fad4f38faf028c5727e",
                current_baseline="stored endpoint with original native operation graph",
                proposal="ephemeral workspace, stored endpoints remain authoritative; arithmetic policy needs review",
                production_changes=0, owner_advances=0, acceptance_promotions=0,
                original_native_refusals=5, original_temporal_band="FAIL_ORIGINAL_TEMPORAL_BAND",
                cases=dict(lost_increment=lost, same_inputs_acceptance=accumulation,
                           moving_constraint=chart, embedding=embedding, directional_donor=donor,
                           operator_recomputation=operators, rounded_geometry=geometry, retained_restart=restart),
                rejected_corruptions=controls,
                limitations="No native compensated evaluator, complete Rheon equation, restart API, public-call repair or numerical floor.")


if __name__ == "__main__":
    print(json.dumps(run(), indent=2, sort_keys=True))
