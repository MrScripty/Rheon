import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith

/-! Conditional exact-real two-parcel remap identities. Geometry partition,
arbitrary meshes, IEEE/Rust refinement, pressure coupling and physical accuracy
are not established by these algebraic statements. -/
namespace Rheon.ColumnMomentum
noncomputable section

theorem interval_partition (h cut : ℝ) : cut+(h-cut)=h := by ring

theorem target_mass_positive (rho area length : ℝ)
    (hr : 0 < rho) (ha : 0 < area) (hl : 0 < length) :
    0 < rho*area*length := mul_pos (mul_pos hr ha) hl

theorem mass_with_explicit_caps (old kept removed added new : ℝ)
    (ho : old=kept+removed) (hn : new=kept+added) :
    new-old-added+removed=0 := by rw [ho,hn];ring

theorem weighted_momentum (a b u w v : ℝ)
    (update : (a+b)*v=a*u+b*w) :
    (a+b)*v-a*u-b*w=0 := by linarith

theorem constant_preservation (a b u v : ℝ)
    (positive : 0<a+b) (update : (a+b)*v=a*u+b*u) : v=u := by
  have h : (a+b)*(v-u)=0 := by nlinarith [update]
  rcases mul_eq_zero.mp h with hm | hv
  · linarith
  · linarith

theorem mixing_identity (a b u w v : ℝ)
    (update : (a+b)*v=a*u+b*w) :
    (a*u^2+b*w^2)/2-(a+b)*v^2/2 =
    (a*(u-v)^2+b*(w-v)^2)/2 := by
  nlinarith [congrArg (fun x : ℝ => x*v) update]

theorem mixing_nonnegative (a b u w v : ℝ) (ha : 0≤a) (hb : 0≤b) :
    0≤(a*(u-v)^2+b*(w-v)^2)/2 := by positivity

theorem energy_nonincrease (a b u w v : ℝ) (ha : 0≤a) (hb : 0≤b)
    (update : (a+b)*v=a*u+b*w) :
    (a+b)*v^2/2 ≤ (a*u^2+b*w^2)/2 := by
  have hid := mixing_identity a b u w v update
  have hn := mixing_nonnegative a b u w v ha hb
  linarith

theorem rounding_momentum (m v r : ℝ) :
    m*(v+r)-m*v=m*r := by ring

theorem rounding_energy_work (m v r : ℝ) :
    m*(v+r)^2/2-m*v^2/2=m*r*(v+r/2) := by ring

theorem closed_caps_preserve_momentum (before after added removed : ℝ)
    (ledger : after-before-added+removed=0) (closed : added=removed) :
    after=before := by linarith

theorem convex_bounds (a b u w v lo hi : ℝ)
    (ha : 0≤a) (hb : 0≤b) (hm : 0<a+b)
    (hlu : lo≤u) (hlw : lo≤w) (huh : u≤hi) (hwh : w≤hi)
    (update : (a+b)*v=a*u+b*w) : lo≤v ∧ v≤hi := by
  have lower : (a+b)*(v-lo)≥0 := by
    nlinarith [mul_nonneg ha (sub_nonneg.mpr hlu),mul_nonneg hb (sub_nonneg.mpr hlw)]
  have upper : (a+b)*(hi-v)≥0 := by
    nlinarith [mul_nonneg ha (sub_nonneg.mpr huh),mul_nonneg hb (sub_nonneg.mpr hwh)]
  constructor
  · nlinarith
  · nlinarith

end
end Rheon.ColumnMomentum
