import Rheon.SphereInterval

/-! Exact-real isolated isotropic sphere impulse algebra. Unit normal, radial
lever, constitutive law and matched impulse updates are explicit premises.
No floating geometry, libm, Rust refinement, mode certification, persistent
contact or general Coulomb collision integration is asserted. -/
namespace Rheon.SphereFriction
noncomputable section
open Rheon.MeshTraction
open Rheon.RigidMotion (scale)
open Rheon.SphereContact (squared)

def pointVelocity (v w r : Vec) : Vec := add v (cross w r)
def kick (v j : Vec) (m : ℝ) : Vec := add v (scale (1/m) j)
def spinKick (w r j : Vec) (i : ℝ) : Vec := add w (scale (1/i) (cross r j))
def totalEnergy (m i : ℝ) (v w : Vec) : ℝ := m/2*squared v+i/2*squared w
def contactAction (j r : Vec) (m i : ℝ) : Vec :=
  add (scale (1/m) j) (cross (scale (1/i) (cross r j)) r)
def tangentObjective (v z : Vec) (k : ℝ) : ℝ := dot v z+k/2*squared z

theorem positive_tangent_inverse_mass (m i d : ℝ) (hm : 0<m) (hi : 0<i) :
    0<1/m+d^2/i := by
  have h1 : 0<1/m := one_div_pos.mpr hm
  have h2 : 0≤d^2/i := div_nonneg (sq_nonneg d) (le_of_lt hi)
  linarith

theorem unit_tangent_projection (g n : Vec) (hn : squared n=1) :
    dot (sub g (scale (dot g n) n)) n=0 := by
  have h : dot (sub g (scale (dot g n) n)) n=dot g n-(dot g n)*squared n := by
    simp only [dot,sub,scale,squared]
    ring
  rw [h,hn]
  ring

theorem point_velocity_increment (v w r j : Vec) (m i : ℝ) :
    sub (pointVelocity (kick v j m) (spinKick w r j i) r)
      (pointVelocity v w r) = contactAction j r m i := by
  simp only [pointVelocity,kick,spinKick,contactAction,add,sub,cross,scale]
  congr 1 <;> ring

theorem matched_linear_impulse (v j : Vec) (m : ℝ) (hm : m≠0) :
    scale m (sub (kick v j m) v)=j := by
  simp only [kick,scale,sub,add]
  congr 1 <;> field_simp <;> ring

theorem matched_spin_impulse (w r j : Vec) (i : ℝ) (hi : i≠0) :
    scale i (sub (spinKick w r j i) w)=cross r j := by
  simp only [spinKick,scale,sub,add,cross]
  congr 1 <;> field_simp <;> ring

theorem matched_world_angular_impulse (c q v w j : Vec) (m i : ℝ)
    (hm : m≠0) (hi : i≠0) :
    add (cross c (scale m (sub (kick v j m) v)))
      (scale i (sub (spinKick w (sub q c) j i) w)) = cross q j := by
  rw [matched_linear_impulse v j m hm,matched_spin_impulse w (sub q c) j i hi]
  simp only [cross,sub,add]
  congr 1 <;> ring

theorem matched_kinetic_increment (v w r j : Vec) (m i : ℝ)
    (hm : m≠0) (hi : i≠0) :
    totalEnergy m i (kick v j m) (spinKick w r j i)-totalEnergy m i v w =
    dot (pointVelocity v w r) j+squared j/(2*m)+squared (cross r j)/(2*i) := by
  simp only [totalEnergy,kick,spinKick,pointVelocity,squared,add,scale,dot,cross]
  field_simp
  ring

theorem matched_midpoint_work (v w r j : Vec) (m i : ℝ)
    (hm : m≠0) (hi : i≠0) :
    dot (scale (1/2) (add (pointVelocity v w r)
      (pointVelocity (kick v j m) (spinKick w r j i) r))) j =
      totalEnergy m i (kick v j m) (spinKick w r j i)-totalEnergy m i v w := by
  rw [matched_kinetic_increment v w r j m i hm hi]
  simp only [pointVelocity,kick,spinKick,squared,scale,add,cross,dot]
  ring

theorem radial_contact_action (j n : Vec) (d m i : ℝ) :
    contactAction j (scale (-d) n) m i =
    add (scale (1/m) j)
      (scale (d^2/i) (sub (scale (squared n) j) (scale (dot j n) n))) := by
  simp only [contactAction,scale,cross,add,sub,squared,dot]
  congr 1 <;> ring

theorem unit_radial_tangent_action (j n : Vec) (d m i : ℝ)
    (hn : squared n=1) (ht : dot j n=0) :
    contactAction j (scale (-d) n) m i=scale (1/m+d^2/i) j := by
  rw [radial_contact_action,hn,ht]
  simp only [scale,add,sub]
  congr 1 <;> ring

theorem radial_normal_point_speed (v w n : Vec) (d : ℝ) :
    dot (pointVelocity v w (scale (-d) n)) n=dot v n := by
  simp only [pointVelocity,scale,add,cross,dot]
  ring

/-- The reported candidate is the exact scalar minimizer only under these real
premises. Rust min/comparison is separately observed and not enclosed here. -/
theorem disk_magnitude_bounds (s k cap : ℝ) (hs : 0≤s)
    (hk : 0<k) (hc : 0≤cap) :
    0≤min (s/k) cap ∧ min (s/k) cap≤cap ∧ k*min (s/k) cap≤s := by
  have hq : 0≤min (s/k) cap := le_min (div_nonneg hs (le_of_lt hk)) hc
  have hl := min_le_left (s/k) cap
  have hp : k*min (s/k) cap≤s := by
    have h := (le_div_iff₀ hk).mp hl
    nlinarith
  exact ⟨hq,min_le_right _ _,hp⟩

/-- Minimum in scalar magnitude; anti-slip direction additionally
requires the vector disk/Cauchy premises, not a generic friction theorem. -/
theorem disk_magnitude_optimal (s k cap t : ℝ) (hk : 0<k)
    (htc : t≤cap) :
    -s*min (s/k) cap+k/2*(min (s/k) cap)^2≤ -s*t+k/2*t^2 := by
  by_cases h : s/k≤cap
  · rw [min_eq_left h]
    have hsq := sq_nonneg (t-s/k)
    have hp := mul_nonneg (le_of_lt hk) hsq
    have hid : -s*t+k/2*t^2-(-s*(s/k)+k/2*(s/k)^2)=k/2*(t-s/k)^2 := by
      field_simp
      ring
    nlinarith

  · have hcs : cap≤s/k := le_of_lt (lt_of_not_ge h)
    rw [min_eq_right hcs]
    have hks : k*cap≤s := by
      have h' := (le_div_iff₀ hk).mp hcs
      nlinarith
    have hb : k/2*(t+cap)-s≤0 := by nlinarith
    have ha : t-cap≤0 := by linarith
    have hp := mul_nonneg_of_nonpos_of_nonpos ha hb
    nlinarith

/-- Exact vector objective gap. The cap term's nonnegativity follows from
Cauchy and disk membership in the capped case; it vanishes in cancellation. -/
theorem vector_disk_objective_gap (v z : Vec) (s k q : ℝ)
    (hs : s≠0) (hv : squared v=s^2) :
    tangentObjective v z k-tangentObjective v (scale (-q/s) v) k =
      k/2*squared (sub z (scale (-q/s) v))+
      (s-k*q)*(q+dot v z/s) := by
  have h : tangentObjective v z k-tangentObjective v (scale (-q/s) v) k =
      k/2*squared (sub z (scale (-q/s) v))+
      (s-k*q)*(q+dot v z/s)+(q/s-k*q^2/s^2)*(squared v-s^2) := by
    simp only [tangentObjective,scale,sub,squared,dot]
    field_simp
    ring
  rw [h,hv]
  ring

theorem conditional_vector_disk_optimal (v z : Vec) (s k q : ℝ)
    (hs : s≠0) (hv : squared v=s^2) (hk : 0≤k)
    (hresidual : 0≤s-k*q) (hdisk : 0≤q+dot v z/s) :
    tangentObjective v (scale (-q/s) v) k≤tangentObjective v z k := by
  have hn := Rheon.SphereInterval.squared_nonnegative (sub z (scale (-q/s) v))
  have hp : 0≤k/2*squared (sub z (scale (-q/s) v)) := mul_nonneg (by linarith) hn
  have hc : 0≤(s-k*q)*(q+dot v z/s) := mul_nonneg hresidual hdisk
  have h := vector_disk_objective_gap v z s k q hs hv
  linarith

theorem disk_cauchy_lower (v z : Vec) (s cap : ℝ)
    (hs : 0<s) (hc : 0≤cap) (hv : squared v=s^2) (hz : squared z≤cap^2) :
    -s*cap≤dot v z := by
  have h := Rheon.SphereInterval.squared_cauchy v z
  rw [hv] at h
  have hp := mul_le_mul_of_nonneg_left hz (sq_nonneg s)
  have hsc := mul_nonneg (le_of_lt hs) hc
  by_contra hn
  have hd : dot v z< -s*cap := lt_of_not_ge hn
  nlinarith

/-- Full disk optimality follows from a norm bound and Cauchy. All vectors
in the ball are covered, hence also its tangent-plane intersection. -/
theorem vector_disk_optimal (v z : Vec) (s k cap : ℝ)
    (hs : 0<s) (hk : 0<k) (hc : 0≤cap)
    (hv : squared v=s^2) (hz : squared z≤cap^2) :
    tangentObjective v (scale (-(min (s/k) cap)/s) v) k≤tangentObjective v z k := by
  by_cases h : s/k≤cap
  · rw [min_eq_left h]
    have hg := vector_disk_objective_gap v z s k (s/k) (ne_of_gt hs) hv
    have he : s-k*(s/k)=0 := by field_simp
    rw [he] at hg
    have hn := Rheon.SphereInterval.squared_nonnegative (sub z (scale (-(s/k)/s) v))
    have hp : 0≤k/2*squared (sub z (scale (-(s/k)/s) v)) := mul_nonneg (by linarith) hn
    linarith
  · have hcs : cap≤s/k := le_of_lt (lt_of_not_ge h)
    rw [min_eq_right hcs]
    have hks : k*cap≤s := by
      have ht := (le_div_iff₀ hk).mp hcs
      nlinarith
    have hd := disk_cauchy_lower v z s cap hs hc hv hz
    have hdiv : -cap≤dot v z/s := (le_div_iff₀ hs).mpr (by nlinarith)
    exact conditional_vector_disk_optimal v z s k cap (ne_of_gt hs) hv
      (le_of_lt hk) (by linarith) (by linarith)

theorem conditional_tangent_multiplier (s k q : ℝ) (hs : 0<s)
    (hk : 0≤k) (hq : 0≤q) (hcap : k*q≤s) :
    0≤1-k*q/s ∧ 1-k*q/s≤1 := by
  have hlo : 0≤k*q/s := div_nonneg (mul_nonneg hk hq) (le_of_lt hs)
  have hhi : k*q/s≤1 := (div_le_one hs).mpr hcap
  constructor <;> linarith

theorem conditional_tangent_nonincrease (s k q : ℝ) (hk : 0≤k)
    (hq : 0≤q) (hcap : k*q≤s) : -q*s+k/2*q^2≤0 := by
  have h := mul_le_mul_of_nonneg_left hcap hq
  have hp : 0≤k*q^2 := mul_nonneg hk (sq_nonneg q)
  nlinarith

theorem normal_energy_identity (m e vn : ℝ) (hm : m≠0) :
    vn*(-(1+e)*m*vn)+(-(1+e)*m*vn)^2/(2*m)=
      -(m/2)*(1-e^2)*vn^2 := by
  field_simp
  ring

/-- Generic exact kinetic increment becomes the stated split balance under
explicit measured-law identities. These identities are NOT IEEE assertions. -/
theorem conditional_split_balance (v w r j : Vec) (m i d e vn s q : ℝ)
    (hm : m≠0) (hi : i≠0)
    (hwork : dot (pointVelocity v w r) j=vn*(-(1+e)*m*vn)-q*s)
    (himp : squared j=(-(1+e)*m*vn)^2+q^2)
    (htorque : squared (cross r j)=d^2*q^2) :
    totalEnergy m i (kick v j m) (spinKick w r j i)-totalEnergy m i v w =
      -(m/2)*(1-e^2)*vn^2-q*s+(1/m+d^2/i)/2*q^2 := by
  rw [matched_kinetic_increment v w r j m i hm hi,hwork,himp,htorque]
  have h := normal_energy_identity m e vn hm
  calc
    _ = (vn*(-(1+e)*m*vn)+(-(1+e)*m*vn)^2/(2*m))
        -q*s+(1/m+d^2/i)/2*q^2 := by ring
    _ = _ := by rw [h]

theorem conditional_total_nonincrease (before after m e vn s k q : ℝ)
    (hm : 0≤m) (he0 : 0≤e) (he1 : e≤1) (hk : 0≤k)
    (hq : 0≤q) (hcap : k*q≤s)
    (hmatched : after-before= -(m/2)*(1-e^2)*vn^2-q*s+k/2*q^2) :
    after≤before := by
  have he : 0≤1-e^2 := by nlinarith
  have hn : 0≤(m/2)*(1-e^2)*vn^2 :=
    mul_nonneg (mul_nonneg (by linarith) he) (sq_nonneg vn)
  have ht := conditional_tangent_nonincrease s k q hk hq hcap
  linarith

end
end Rheon.SphereFriction
