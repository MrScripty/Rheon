import Rheon.Discrete
import Rheon.Physics

/-! Exact-real reference contracts for stationary obstacle pressure and reduced
extruded shear. `edgeIncidence` is +1 at the head and -1 at the tail; outward
volume flux is its negative applied to area times speed. Pressure inertia is
the declared face-area mass rho*A*d, not exact clipped dual volume. Shear uses
rho*V layer masses, shared-area/centroid-distance conductances and stationary
wall diagonals. The scalar shear interpretation is restricted to u=(u(y),0,0)
with constant Newtonian coefficients and invariant/periodic tangential axes.

Geometry admission and coefficient assembly are external obligations. These
statements do not certify Rust, IEEE arithmetic, PCG/Thomas convergence,
existence/uniqueness, complete nullspace characterization, general 3D viscous
traction, continuum convergence or moving walls. -/
namespace Rheon.ObstacleOperators
noncomputable section
open scoped BigOperators
open Rheon.Discrete

/-- One shared oriented face; tail and head are retained wet-cell indices. -/
def edgeIncidence {n m : ℕ} (tail head : Fin m → Fin n)
    (i : Fin n) (e : Fin m) : ℝ :=
  (if i = head e then 1 else 0) - (if i = tail e then 1 else 0)

theorem edge_gradient {n m : ℕ} (tail head : Fin m → Fin n)
    (p : Fin n → ℝ) (e : Fin m) :
    gradient (edgeIncidence tail head) p e = p (head e) - p (tail e) := by
  simp [gradient, edgeIncidence, sub_mul, Finset.sum_sub_distrib, ite_mul]

theorem edge_balanced {n m : ℕ} (tail head : Fin m → Fin n) (e : Fin m) :
    (∑ i, edgeIncidence tail head i e) = 0 := by
  simp [edgeIncidence, Finset.sum_sub_distrib]

def outwardFlux {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (area u : Fin m → ℝ) (i : Fin n) : ℝ :=
  -incidence B (fun e => area e * u e) i

def pressureWeight {m : ℕ} (rho : ℝ) (area distance : Fin m → ℝ)
    (e : Fin m) : ℝ := area e / (rho * distance e)

def faceMass {m : ℕ} (rho : ℝ) (area distance : Fin m → ℝ)
    (e : Fin m) : ℝ := rho * area e * distance e

def pressureCorrected {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (rho dt : ℝ) (distance u : Fin m → ℝ) (p : Fin n → ℝ) (e : Fin m) : ℝ :=
  u e - dt * gradient B p e / (rho * distance e)

def pressureRhs {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (area u : Fin m → ℝ) (dt : ℝ) (i : Fin n) : ℝ :=
  -outwardFlux B area u i / dt

def pressureResidual {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (rho dt : ℝ) (area distance u : Fin m → ℝ) (p : Fin n → ℝ) (i : Fin n) : ℝ :=
  pressureRhs B area u dt i - laplace B (pressureWeight rho area distance) p i

def kineticEnergy {n : ℕ} (mass u : Fin n → ℝ) : ℝ :=
  ∑ i, mass i / 2 * u i ^ 2

/-- This algebraic identity makes the implicit/new-time work convention explicit. -/
theorem kinetic_increment {n : ℕ} (mass u v : Fin n → ℝ) :
    kineticEnergy mass v - kineticEnergy mass u +
      kineticEnergy mass (fun i => v i - u i) =
      ∑ i, mass i * (v i - u i) * v i := by
  unfold kineticEnergy
  rw [← Finset.sum_sub_distrib, ← Finset.sum_add_distrib]
  apply Finset.sum_congr rfl
  intro i _
  ring

theorem face_mass_coefficient {m : ℕ} (rho : ℝ) (area distance : Fin m → ℝ)
    (hr : rho ≠ 0) (hd : ∀ e, distance e ≠ 0) (e : Fin m) :
    faceMass rho area distance e / (rho * distance e) = area e := by
  unfold faceMass
  field_simp [hr, hd e]
  ring

theorem pressure_corrected_flux {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (rho dt : ℝ) (area distance u : Fin m → ℝ) (p : Fin n → ℝ) :
    (fun e => area e * pressureCorrected B rho dt distance u p e) =
      corrected B (pressureWeight rho area distance) (fun e => area e * u e) p dt := by
  funext e
  unfold pressureCorrected corrected pressureWeight
  ring

/-- r=b-Lp, b=-Q_old/dt imply Qnew=-dt*r, including all gauge rows. -/
theorem pressure_flux_residual {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (rho dt : ℝ) (area distance u : Fin m → ℝ) (p : Fin n → ℝ)
    (ht : dt ≠ 0) (i : Fin n) :
    outwardFlux B area (pressureCorrected B rho dt distance u p) i =
      -dt * pressureResidual B rho dt area distance u p i := by
  have rhs : ∀ i, incidence B (fun e => area e * u e) i =
      dt * pressureRhs B area u dt i := by
    intro i
    unfold pressureRhs outwardFlux
    simp only [neg_neg]
    field_simp [ht]
  unfold outwardFlux
  rw [pressure_corrected_flux, projection_residual B _ _ _ _ dt rhs i]
  unfold pressureResidual
  ring

/-- A positive wet volume is essential when converting an integrated residual. -/
theorem pressure_divergence_residual {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (rho dt : ℝ) (area distance u : Fin m → ℝ) (p volume : Fin n → ℝ)
    (ht : 0 < dt) (hv : ∀ i, 0 < volume i) (i : Fin n) :
    outwardFlux B area (pressureCorrected B rho dt distance u p) i / volume i =
      -dt * pressureResidual B rho dt area distance u p i / volume i ∧
    |outwardFlux B area (pressureCorrected B rho dt distance u p) i / volume i| =
      dt * |pressureResidual B rho dt area distance u p i| / volume i := by
  rw [pressure_flux_residual B rho dt area distance u p (ne_of_gt ht) i]
  constructor
  · rfl
  · rw [abs_div, abs_mul, abs_neg, abs_of_pos ht, abs_of_pos (hv i)]

/-- Compatibility is local to each supplied connected label, not just global. -/
theorem component_rhs_compatible {n m : ℕ} (tail head : Fin m → Fin n)
    (label : Fin n → ℕ) (connected : ∀ e, label (tail e) = label (head e))
    (area u : Fin m → ℝ) (dt : ℝ) (c : ℕ) :
    (∑ i, (if label i = c then (1 : ℝ) else 0) *
      pressureRhs (edgeIncidence tail head) area u dt i) = 0 := by
  let potential : Fin n → ℝ := fun i => if label i = c then 1 else 0
  have hgrad : ∀ e, gradient (edgeIncidence tail head) potential e = 0 := by
    intro e
    rw [edge_gradient]
    simp only [potential, connected e, sub_self]
  have h := adjoint_identity (edgeIncidence tail head) (fun e => area e * u e) potential
  simp only [hgrad, zero_mul, Finset.sum_const_zero] at h
  unfold pressureRhs outwardFlux
  simp only [neg_neg, div_eq_mul_inv, ← mul_assoc, ← Finset.sum_mul]
  change (∑ i, potential i * incidence (edgeIncidence tail head) (fun e => area e * u e) i) * dt⁻¹ = 0
  rw [h, zero_mul]

/-- Derived from the coordinate correction and rho*A*d inertia, without a solve assumption. -/
theorem pressure_correction_energy {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (rho dt : ℝ) (area distance u : Fin m → ℝ) (p : Fin n → ℝ)
    (hr : rho ≠ 0) (hd : ∀ e, distance e ≠ 0) :
    kineticEnergy (faceMass rho area distance) (pressureCorrected B rho dt distance u p) -
      kineticEnergy (faceMass rho area distance) u +
      kineticEnergy (faceMass rho area distance)
        (fun e => pressureCorrected B rho dt distance u p e - u e) =
      dt * ∑ i, p i * outwardFlux B area (pressureCorrected B rho dt distance u p) i := by
  rw [kinetic_increment]
  have local_step : ∀ e, faceMass rho area distance e *
      (pressureCorrected B rho dt distance u p e - u e) =
      -dt * gradient B p e * area e := by
    intro e
    unfold pressureCorrected faceMass
    field_simp [hr, hd e]
    ring
  simp_rw [local_step]
  unfold outwardFlux
  simp only [mul_neg, Finset.sum_neg_distrib]
  rw [adjoint_identity]
  rw [Finset.mul_sum, ← Finset.sum_neg_distrib]
  apply Finset.sum_congr rfl
  intro e _
  ring

theorem exact_pressure_energy_nonincrease {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (rho dt : ℝ) (area distance u : Fin m → ℝ) (p : Fin n → ℝ)
    (hr : 0 < rho) (ha : ∀ e, 0 < area e) (hd : ∀ e, 0 < distance e)
    (ht : dt ≠ 0)
    (solved : ∀ i, pressureResidual B rho dt area distance u p i = 0) :
    kineticEnergy (faceMass rho area distance) (pressureCorrected B rho dt distance u p) ≤
      kineticEnergy (faceMass rho area distance) u := by
  have he := pressure_correction_energy B rho dt area distance u p
    (ne_of_gt hr) (fun e => ne_of_gt (hd e))
  simp_rw [pressure_flux_residual B rho dt area distance u p ht, solved, mul_zero] at he
  simp only [Finset.sum_const_zero, mul_zero] at he
  have inc : 0 ≤ kineticEnergy (faceMass rho area distance)
      (fun e => pressureCorrected B rho dt distance u p e - u e) := by
    apply Finset.sum_nonneg
    intro e _
    exact mul_nonneg (div_nonneg (le_of_lt (mul_pos (mul_pos hr (ha e)) (hd e)))
      (by norm_num)) (sq_nonneg _)
  linarith

/-- mu*A/d for shared fluid interfaces or stationary no-slip wall half-layers. -/
def shearConductance (mu area distance : ℝ) : ℝ := mu * area / distance

/-- Eliminated Navier wall trace; beta=0 is exactly free slip. -/
def navierConductance (mu area beta distance : ℝ) : ℝ :=
  area * mu * beta / (mu + beta * distance)

def navierTrace (mu beta distance speed : ℝ) : ℝ :=
  mu * speed / (mu + beta * distance)

theorem shear_conductance_positive (mu area distance : ℝ)
    (hm : 0 < mu) (ha : 0 < area) (hd : 0 < distance) :
    0 < shearConductance mu area distance := div_pos (mul_pos hm ha) hd

theorem navier_conductance_nonnegative (mu area beta distance : ℝ)
    (hm : 0 < mu) (ha : 0 ≤ area) (hb : 0 ≤ beta) (hd : 0 ≤ distance) :
    0 ≤ navierConductance mu area beta distance := by
  exact div_nonneg (mul_nonneg (mul_nonneg ha (le_of_lt hm)) hb)
    (le_of_lt (add_pos_of_pos_of_nonneg hm (mul_nonneg hb hd)))

theorem navier_free_slip (mu area distance : ℝ) :
    navierConductance mu area 0 distance = 0 := by
  simp [navierConductance]

/-- The effective force equals both fluid half-layer traction and wall friction. -/
theorem navier_series_traction (mu area beta distance speed : ℝ)
    (hm : 0 < mu) (hb : 0 ≤ beta) (hd : 0 < distance) :
    shearConductance mu area distance * (speed - navierTrace mu beta distance speed) =
      navierConductance mu area beta distance * speed ∧
    area * beta * navierTrace mu beta distance speed =
      navierConductance mu area beta distance * speed := by
  have hden : mu + beta * distance ≠ 0 :=
    ne_of_gt (add_pos_of_pos_of_nonneg hm (mul_nonneg hb (le_of_lt hd)))
  have hd0 : distance ≠ 0 := ne_of_gt hd
  unfold shearConductance navierTrace navierConductance
  constructor <;> field_simp [hden, hd0] <;> ring

/-- Boundary loss includes fluid half-layer strain and stationary-wall friction. -/
theorem navier_dissipation_split (mu area beta distance speed : ℝ)
    (hm : 0 < mu) (hb : 0 ≤ beta) (hd : 0 < distance) :
    shearConductance mu area distance * (speed - navierTrace mu beta distance speed)^2 +
      area * beta * (navierTrace mu beta distance speed)^2 =
      navierConductance mu area beta distance * speed^2 := by
  have hden : mu + beta * distance ≠ 0 :=
    ne_of_gt (add_pos_of_pos_of_nonneg hm (mul_nonneg hb (le_of_lt hd)))
  have hd0 : distance ≠ 0 := ne_of_gt hd
  unfold shearConductance navierTrace navierConductance
  field_simp [hden, hd0]
  ring

/-- Two wall contributions add even when both touch the same singleton layer. -/
def stationaryWallDiagonal {n : ℕ} (lower upper : Fin n) (kl ku : ℝ) (i : Fin n) : ℝ :=
  (if i = lower then kl else 0) + (if i = upper then ku else 0)

theorem stationary_wall_nonnegative {n : ℕ} (lower upper : Fin n) (kl ku : ℝ)
    (hl : 0 ≤ kl) (hu : 0 ≤ ku) (i : Fin n) :
    0 ≤ stationaryWallDiagonal lower upper kl ku i := by
  unfold stationaryWallDiagonal
  exact add_nonneg (by split <;> linarith) (by split <;> linarith)

theorem stationary_wall_force {n : ℕ} (lower upper : Fin n) (kl ku : ℝ)
    (v : Fin n → ℝ) :
    (∑ i, stationaryWallDiagonal lower upper kl ku i * v i) =
      kl * v lower + ku * v upper := by
  simp [stationaryWallDiagonal, add_mul, ite_mul, Finset.sum_add_distrib]

/-- wall sums all stationary boundary conductances at a layer; for one layer
both lower and upper wall coefficients contribute to this same diagonal. -/
def shearOperator {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (conductance : Fin m → ℝ) (wall v : Fin n → ℝ) (i : Fin n) : ℝ :=
  laplace B conductance v i + wall i * v i

def shearDissipation {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (conductance : Fin m → ℝ) (wall v : Fin n → ℝ) : ℝ :=
  (∑ e, conductance e * (gradient B v e)^2) + ∑ i, wall i * v i ^ 2

theorem shear_work {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (conductance : Fin m → ℝ) (wall v : Fin n → ℝ) :
    (∑ i, v i * shearOperator B conductance wall v i) =
      shearDissipation B conductance wall v := by
  unfold shearOperator shearDissipation
  simp only [mul_add, Finset.sum_add_distrib]
  rw [pressure_energy]
  congr 1
  apply Finset.sum_congr rfl
  intro i _
  ring

theorem shear_dissipation_nonnegative {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (conductance : Fin m → ℝ) (wall v : Fin n → ℝ)
    (hk : ∀ e, 0 ≤ conductance e) (hw : ∀ i, 0 ≤ wall i) :
    0 ≤ shearDissipation B conductance wall v := by
  exact add_nonneg
    (Finset.sum_nonneg (fun e _ => mul_nonneg (hk e) (sq_nonneg _)))
    (Finset.sum_nonneg (fun i _ => mul_nonneg (hw i) (sq_nonneg _)))

/-- Exact (M+dt*K)v=M*u+dt*q*V coordinate equations imply the forced work ledger. -/
theorem forced_shear_energy {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (conductance : Fin m → ℝ) (wall volume u v : Fin n → ℝ) (rho dt q : ℝ)
    (step : ∀ i, (rho * volume i) * v i + dt * shearOperator B conductance wall v i =
      (rho * volume i) * u i + dt * (q * volume i)) :
    kineticEnergy (fun i => rho * volume i) v - kineticEnergy (fun i => rho * volume i) u +
      kineticEnergy (fun i => rho * volume i) (fun i => v i - u i) +
      dt * shearDissipation B conductance wall v =
      dt * ∑ i, q * volume i * v i := by
  rw [kinetic_increment, ← shear_work B conductance wall v, Finset.mul_sum,
    ← Finset.sum_add_distrib, Finset.mul_sum]
  apply Finset.sum_congr rfl
  intro i _
  have hi := congrArg (fun x : ℝ => x * v i) (step i)
  dsimp only at hi
  ring_nf at hi ⊢
  linarith only [hi]

/-- Internal interface forces cancel by incidence; only body and wall impulses remain. -/
theorem forced_shear_momentum {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0)
    (conductance : Fin m → ℝ) (wall volume u v : Fin n → ℝ) (rho dt q : ℝ)
    (step : ∀ i, (rho * volume i) * v i + dt * shearOperator B conductance wall v i =
      (rho * volume i) * u i + dt * (q * volume i)) :
    (∑ i, (rho * volume i) * (v i - u i)) =
      dt * (∑ i, q * volume i) - dt * ∑ i, wall i * v i := by
  have internal : (∑ i, laplace B conductance v i) = 0 :=
    internal_flux_conservation B balanced (fun e => conductance e * gradient B v e)
  have coordinate : ∀ i, (rho * volume i) * (v i - u i) =
      dt * (q * volume i) - dt * laplace B conductance v i - dt * (wall i * v i) := by
    intro i
    have hi := step i
    unfold shearOperator at hi
    nlinarith only [hi]
  simp only [coordinate, Finset.sum_sub_distrib, ← Finset.mul_sum, internal,
    mul_zero, sub_zero]

/-- Actual oriented layer edges and lower/upper diagonal give the two-wall ledger. -/
theorem two_wall_shear_momentum {n m : ℕ} (tail head : Fin m → Fin n)
    (lower upper : Fin n) (kl ku : ℝ)
    (conductance : Fin m → ℝ) (volume u v : Fin n → ℝ) (rho dt q : ℝ)
    (step : ∀ i, (rho * volume i) * v i + dt *
      shearOperator (edgeIncidence tail head) conductance
        (stationaryWallDiagonal lower upper kl ku) v i =
      (rho * volume i) * u i + dt * (q * volume i)) :
    (∑ i, (rho * volume i) * (v i - u i)) =
      dt * (∑ i, q * volume i) - dt * (kl * v lower + ku * v upper) := by
  have h := forced_shear_momentum (edgeIncidence tail head) (edge_balanced tail head)
    conductance (stationaryWallDiagonal lower upper kl ku) volume u v rho dt q step
  simpa only [stationary_wall_force] using h

theorem unforced_shear_energy_nonincrease {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (conductance : Fin m → ℝ) (wall volume u v : Fin n → ℝ) (rho dt : ℝ)
    (hr : 0 ≤ rho) (hv : ∀ i, 0 ≤ volume i) (ht : 0 ≤ dt)
    (hk : ∀ e, 0 ≤ conductance e) (hw : ∀ i, 0 ≤ wall i)
    (step : ∀ i, (rho * volume i) * v i + dt * shearOperator B conductance wall v i =
      (rho * volume i) * u i) :
    kineticEnergy (fun i => rho * volume i) v ≤ kineticEnergy (fun i => rho * volume i) u := by
  have he := forced_shear_energy B conductance wall volume u v rho dt 0
    (by simpa using step)
  simp only [zero_mul, Finset.sum_const_zero, mul_zero] at he
  have inc : 0 ≤ kineticEnergy (fun i => rho * volume i) (fun i => v i - u i) := by
    exact Finset.sum_nonneg (fun i _ =>
      mul_nonneg (div_nonneg (mul_nonneg hr (hv i)) (by norm_num)) (sq_nonneg _))
  have diss := mul_nonneg ht (shear_dissipation_nonnegative B conductance wall v hk hw)
  linarith

/-- The gradient of the *specified* fully developed mode u=(u(y),0,0). -/
def reducedShearGradient (derivative : ℝ) (i j : Fin 3) : ℝ :=
  if i = 0 ∧ j = 1 then derivative else 0

/-- mu/2 times the squared symmetric-gradient norm, evaluated at this mode.
This finite algebra does not prove that arbitrary flows have this gradient. -/
theorem reduced_symmetric_strain_dissipation (mu derivative : ℝ) :
    mu / 2 * (∑ i : Fin 3, ∑ j : Fin 3,
      (reducedShearGradient derivative i j + reducedShearGradient derivative j i)^2) =
      mu * derivative^2 := by
  have enumeration : (Finset.univ : Finset (Fin 3)) = {0, 1, 2} := by decide
  simp [enumeration, reducedShearGradient]
  ring

end
end Rheon.ObstacleOperators
