# Primary evidence and limits

Read2026-10-20; remote source observations are versioned evidence, not native
Rheon qualification. No third-party source code is copied into Rheon. Downloaded
reference source is for private inspection outside the repository/evidence bundle.
Main2017author PDF is about18MB: web reader refused size, direct public retrieval
returned403. Do not claim its body was read. Accessible primary sources:

1. Larionov/Batty/Bridson2017, supplemental propositions2.2/2.3, eqs5–18:
   https://cs.uwaterloo.ca/~c2batty/papers/Larionov2017/supplemental.pdf
   Full liquid stress is -pI+2muD; the free-surface functional yields natural
   total traction. It does not prescribe independent p=0 at a viscous interface.
2. Author project and official repository:
   https://elrnv.com/projects/variational-stokes-a-unified-pressure-viscosity-solver-for-accurate-viscous-liquids/
   https://github.com/elrnv/stokes-houdini/blob/1dbccfdb09d62f96b9d778f3febfad12c4e6e6f5/SIM/SIM_Stokes.C
   Code inspection binds to this exact commit. Dual-volume inclusion/support
   checks and minimum sampled weights are practical implementation choices,
   not proof of Rheon's exact-cut conservation or physical applicability.
3. Batty/Bridson2011, author preprint arXiv1010.2832v2, sections4–8:
   https://arxiv.org/pdf/1010.2832
   A predecessor, not the2017paper. It derives weighted variational difference
   operators and nonhomogeneous boundary terms, and explains support-nullspace
   elimination. Distinguish these choices from a consistent-basis Galerkin variant.
4. Burman/Claus/Massing2015, stabilized cut finite elements for three-field Stokes:
   https://arxiv.org/pdf/1408.5165
   Their particular stabilization establishes cut-independent stability. It is
   not a theorem for the proposed unstabilized staggeredQ1/P0pair; positivity of
   geometry/mass alone supplies neither inf-sup nor small-cut conditioning.
5. NIST DLMF§3.5(v), Gauss quadrature:
   https://dlmf.nist.gov/3.5#v
   Positive Gaussian weights and polynomial exactness ground the selected
   quadrature rule; source geometry/basis partition and native rounding still
   need separate independent checks.

Own derivations in the decision derive explicit finite operators, trace and work
from the declared basis/functional. These are mathematical proposals until the
independent method review and prospective fixed-corpus gates pass. No citation
is used as a substitute for source-bound native or physical verification.
