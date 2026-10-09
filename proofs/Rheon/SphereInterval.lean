import Rheon.SphereContact
/-! Exact-real support and finite-ledger algebra. No refinement of binary64 bit
predicates, CCD completeness, libm or simultaneous/resting evolution is proved. -/
namespace Rheon.SphereInterval
noncomputable section
open Rheon.MeshTraction
open Rheon.RigidMotion (scale)
open Rheon.SphereContact (squared)
theorem squared_nonnegative (a : Vec) : 0 ≤ squared a := by
  simp only [squared,dot]
  nlinarith [sq_nonneg a.x, sq_nonneg a.y, sq_nonneg a.z]
theorem squared_cauchy (a b : Vec) : (dot a b)^2 ≤ squared a*squared b := by
  have identity : squared a*squared b-(dot a b)^2 = squared (cross a b) := by
    simp only [squared,dot,cross]
    ring
  have hn := squared_nonnegative (cross a b)
  linarith

theorem convex_support (a p x0 x1 x2 : Vec) (l0 l1 l2 : ℝ)
    (h0 : 0≤l0) (h1 : 0≤l1) (h2 : 0≤l2) (hs : l0+l1+l2=1)
    (s0 : dot a (sub x0 p)≤0) (s1 : dot a (sub x1 p)≤0)
    (s2 : dot a (sub x2 p)≤0) :
    dot a (sub (add (add (scale l0 x0) (scale l1 x1)) (scale l2 x2)) p)≤0 := by
  have identity : dot a (sub (add (add (scale l0 x0) (scale l1 x1)) (scale l2 x2)) p) =
      l0*dot a (sub x0 p)+l1*dot a (sub x1 p)+l2*dot a (sub x2 p)+(l0+l1+l2-1)*dot a p := by
    simp only [dot,sub,add,scale]
    ring
  rw [identity,hs]
  have z0 := mul_nonpos_of_nonneg_of_nonpos h0 s0
  have z1 := mul_nonpos_of_nonneg_of_nonpos h1 s1
  have z2 := mul_nonpos_of_nonneg_of_nonpos h2 s2
  linarith

/-- A rounded endpoint is covered only IF its exact outward predicate holds.
The support point p need not be an exact closest point on the facet. -/
theorem departure_endpoint (c p x endpoint : Vec) (R : ℝ) (hr : 0<R)
    (clearance : R^2≤squared (sub c p))
    (support : dot (sub c p) (sub x p)≤0)
    (outward : 0≤dot (sub c p) (sub endpoint c)) :
    R^2≤squared (sub endpoint x) := by
  let a := sub c p
  let b := sub endpoint x
  have hd : dot a b = squared a-dot a (sub x p)+dot a (sub endpoint c) := by
    dsimp [a,b]
    simp only [dot,sub,squared]
    ring
  have ha : 0<squared a := by dsimp [a]; nlinarith [sq_pos_of_pos hr]
  have hb : squared a≤dot a b := by rw [hd]; dsimp [a] at *; linarith
  have hc := squared_cauchy a b
  have ha2 : (squared a)^2≤(dot a b)^2 := by nlinarith
  have hdist : squared a≤squared b := by nlinarith
  dsimp [a,b] at *
  linarith

theorem supported_distance (c p x : Vec) (R : ℝ) (hr : 0<R)
    (clearance : R^2≤squared (sub c p))
    (support : dot (sub c p) (sub x p)≤0) : R^2≤squared (sub c x) := by
  apply departure_endpoint c p x c R hr clearance support
  simp only [dot,sub]
  ring_nf
  exact le_refl 0

theorem departure_free_flight (c p x v : Vec) (R t : ℝ) (hr : 0<R)
    (clearance : R^2≤squared (sub c p)) (support : dot (sub c p) (sub x p)≤0)
    (velocity : 0≤dot (sub c p) v) (ht : 0≤t) :
    R^2≤squared (sub (add c (scale t v)) x) := by
  apply departure_endpoint c p x (add c (scale t v)) R hr clearance support
  have identity : dot (sub c p) (sub (add c (scale t v)) c)=t*dot (sub c p) v := by
    simp only [dot,sub,add,scale]
    ring
  rw [identity]
  exact mul_nonneg ht velocity

/-- A scalar finite accepted-prefix ledger. Correspondence of each increment
with a physical momentum component, energy change or duration is a premise. -/
def ledger (initial : ℝ) : List ℝ → ℝ
  | [] => initial
  | increment :: rest => ledger (initial+increment) rest

theorem ledger_sum (initial : ℝ) (changes : List ℝ) :
    ledger initial changes = initial+changes.sum := by
  induction changes generalizing initial with
  | nil => simp [ledger]
  | cons d ds ih => rw [ledger,ih]; simp only [List.sum_cons]; ring

theorem conditional_ledger_nonincrease (initial : ℝ) (changes : List ℝ)
    (nonpositive : ∀d∈changes, d≤0) : ledger initial changes≤initial := by
  induction changes generalizing initial with
  | nil => simp [ledger]
  | cons d ds ih =>
    have hd : d≤0 := nonpositive d (by simp)
    have hs : ∀e∈ds,e≤0 := by intro e he; exact nonpositive e (by simp [he])
    rw [ledger]
    have h := ih (initial+d) hs
    linarith

theorem completion_requires_all_time (requested : ℝ) (durations : List ℝ) :
    ledger requested (durations.map Neg.neg)=0 ↔ durations.sum=requested := by
  rw [ledger_sum]
  have h : (durations.map Neg.neg).sum = -durations.sum := by
    induction durations with
    | nil => simp
    | cons d ds ih => simp [ih]; ring
  rw [h]
  constructor <;> intro he <;> linarith
end
end Rheon.SphereInterval
