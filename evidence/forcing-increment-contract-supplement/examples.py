"""Additional exact work terms. No Rheon solve or accepted trajectory."""
from fractions import Fraction as Q
import json


def require(ok, message):
    if not ok:
        raise ValueError(message)


def qstr(value):
    value = Q(value)
    return f"{value.numerator}/{value.denominator}"


def reject(name, condition):
    try:
        require(condition, name)
    except ValueError:
        return name
    raise ValueError(f"missing corruption rejection: {name}")


def run():
    controls = []
    r = [Q(1, 2), Q(1, 2)]
    old_z, new_z = [Q(1), Q(1)], [Q(1), Q(1)+Q(1, 2**52)]
    old = Q(1)
    exact_embedding = sum(a*b for a, b in zip(r, new_z))
    # Actual binary64 embedding graph, not a hypothetical stored value.
    embedded_binary = 0.0
    for a, b in zip(r, new_z):
        embedded_binary += float(a)*float(b)
    new = Q(embedded_binary)
    e0 = sum(a*b for a, b in zip(r, old_z))-old
    e1 = exact_embedding-new
    h, acceleration = Q(1, 32), Q(-64)
    external_work = h*acceleration*new
    nodal_residual = new-old-h*acceleration
    increment = sum(a*(b-c) for a, b, c in zip(r, new_z, old_z))
    stable_nodal = increment-h*acceleration
    nodal_work = new*nodal_residual
    reduced_work = exact_embedding*stable_nodal
    stored_energy_change = (new*new-old*old)/2
    be = (new-old)**2/2
    first = -e1*nodal_residual
    second = -(new+e1)*(e1-e0)
    ledger = stored_energy_change+be-external_work-reduced_work
    require(new == old == 1 and e0 == 0 and e1 == Q(1, 2**53), "actual rounded nonzero endpoint embedding")
    require(first != 0 and second != 0 and nodal_work == 2, "both ledger-defect terms active")
    require(ledger == nodal_work-reduced_work == first+second, "complete exact ledger-defect identity")
    controls.append(reject("omitted first nonzero endpoint-embedding work term", ledger == second))
    controls.append(reject("omitted second nonzero embedding work term", ledger == first))
    controls.append(reject("reversed first nonzero endpoint-embedding work term", ledger == -first+second))
    controls.append(reject("reversed second nonzero embedding work term", ledger == first-second))
    endpoint = dict(r=list(map(qstr, r)), old_z=list(map(qstr, old_z)), new_z=list(map(qstr, new_z)),
                    old_u=qstr(old), new_u_hex=embedded_binary.hex(), exact_embedding=qstr(exact_embedding),
                    e0=qstr(e0), e1=qstr(e1), h=qstr(h), acceleration=qstr(acceleration),
                    nodal_residual=qstr(nodal_residual), stable_nodal_residual=qstr(stable_nodal),
                    external_work=qstr(external_work), energy_change=qstr(stored_energy_change), be_loss=qstr(be),
                    nodal_work=qstr(nodal_work), reduced_work=qstr(reduced_work),
                    first_defect_term=qstr(first), second_defect_term=qstr(second), ledger_defect=qstr(ledger))

    rows = []
    for g in (Q(1, 2**48), -Q(1, 2**48)):
        m0, m1 = [Q(2), Q(3)], [Q(1)+g, Q(4)]
        old, new = [Q(2), Q(-1)], [Q(3), Q(1)]
        plus, minus = Q(2), Q(1)
        net = plus-minus
        gcl = [m1[0]-m0[0]+net, m1[1]-m0[1]-net]
        transferred = plus*new[0]-minus*new[1]
        residual = [m1[i]*new[i]-m0[i]*old[i]+(transferred if i == 0 else -transferred) for i in range(2)]
        delta_t = sum(m1[i]*new[i]**2-m0[i]*old[i]**2 for i in range(2))/2
        be = sum(m0[i]*(new[i]-old[i])**2 for i in range(2))/2
        mix = (plus+minus)*(new[0]-new[1])**2/2
        wg = sum(gcl[i]*new[i]**2 for i in range(2))/2
        work = sum(a*b for a, b in zip(new, residual))
        ledger = delta_t+be+mix+wg-work
        omitted = delta_t+be+mix-work
        reversed_sign = delta_t+be+mix-wg-work
        require(gcl == [g, 0] and wg == 9*g/2 != 0, "nonzero signed GCL work")
        require(ledger == 0 and work == 14+9*g and omitted == -wg and reversed_sign == -2*wg,
                "GCL work exact changing-mass identity")
        label = "positive" if g > 0 else "negative"
        controls.append(reject(f"omitted {label} GCL work", omitted == 0))
        controls.append(reject(f"reversed {label} GCL-work sign", reversed_sign == 0))
        rows.append(dict(g=qstr(g), m0=list(map(qstr, m0)), m1=list(map(qstr, m1)),
                         old_u=list(map(qstr, old)), new_u=list(map(qstr, new)),
                         plus=qstr(plus), minus=qstr(minus), gcl=list(map(qstr, gcl)),
                         gcl_work=qstr(wg), energy_change=qstr(delta_t), be_loss=qstr(be), mixing_loss=qstr(mix),
                         residual=list(map(qstr, residual)), residual_work=qstr(work), ledger=qstr(ledger),
                         omitted_gcl_ledger=qstr(omitted), reversed_gcl_ledger=qstr(reversed_sign)))
    require(len(controls) == 8, "all eight analytical corruption controls")
    return dict(status="PASS_ADDITIONAL_ANALYTICAL_EXAMPLES_ONLY", production_changes=0, owner_advances=0,
                acceptance_promotions=0, cases=dict(nonzero_e1_first_ledger_term=endpoint, nonzero_signed_gcl_work=rows),
                rejected_corruptions=controls, native_compensation_implemented=False,
                limitations="Illustrative exact identities, not a native solver, conservation acceptance, restart or floor proof.")


if __name__ == "__main__":
    print(json.dumps(run(), indent=2, sort_keys=True))
