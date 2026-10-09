import Rheon.SphereFrictionInterval
/-! A single frictionless normal constraint and a strict stationary subset.
Physical load/geometry correspondence and matched updates are premises. No
settling, generic resting solver, IEEE or collision-completeness theorem. -/
namespace Rheon.SphereSupport
noncomputable section
open Rheon.MeshTraction
open Rheon.RigidMotion (scale)
open Rheon.SphereContact (squared)
def reaction (normalForce : ℝ) : ℝ := max 0 (-normalForce)

theorem reaction_nonnegative (f : ℝ) : 0 ≤ reaction f := le_max_left _ _
theorem net_normal_nonnegative (f : ℝ) : 0 ≤ f+reaction f := by
  have h := le_max_right (0:ℝ) (-f)
  unfold reaction
  linarith
theorem normal_complementarity (f : ℝ) : reaction f*(f+reaction f)=0 := by
  by_cases hf : 0 ≤ f
  · have h : -f ≤ 0 := by linarith
    simp [reaction, max_eq_left h]
  · have h : 0 ≤ -f := by linarith
    rw [reaction, max_eq_right h]
    ring

/-- Single-constraint uniqueness; no matrix/contact-set solver is inferred. -/
theorem reaction_unique (f N : ℝ) (hN : 0 ≤ N) (ha : 0 ≤ f+N)
    (hcomp : N*(f+N)=0) : N=reaction f := by
  rcases mul_eq_zero.mp hcomp with hzero | hnet
  · have hf : -f ≤ 0 := by rw [hzero] at ha; linarith
    rw [hzero, reaction, max_eq_left hf]
  · have hf : 0 ≤ -f := by linarith
    rw [reaction, max_eq_right hf]
    linarith

theorem normal_acceleration_nonnegative (f m : ℝ) (hm : 0 < m) :
    0 ≤ (f+reaction f)/m := div_nonneg (net_normal_nonnegative f) (le_of_lt hm)

theorem inward_equilibrium (f : ℝ) (hf : f ≤ 0) : f+reaction f=0 := by
  have h : 0 ≤ -f := by linarith
  rw [reaction,max_eq_right h]
  ring
theorem outward_requires_zero_support (f : ℝ) (hf : 0 < f) : reaction f=0 := by
  have h : -f ≤ 0 := by linarith
  exact max_eq_left h

theorem radial_support_torque (n : Vec) (radius N : ℝ) :
    cross (scale (-radius) n) (scale N n)=⟨0,0,0⟩ := by
  simp only [cross,scale]
  congr 1 <;> ring

/-- Exact load correspondence must establish F=-N*n. The stored nearest
reduced native resultant does not itself prove that physical premise. -/
theorem matched_force_balance (F n : Vec) (N : ℝ) (hF : F=scale (-N) n) :
    add F (scale N n)=⟨0,0,0⟩ := by
  rw [hF]
  simp only [add,scale]
  congr 1 <;> ring

theorem zero_twist_support_power (F r : Vec) :
    dot F (Rheon.SphereFriction.pointVelocity ⟨0,0,0⟩ ⟨0,0,0⟩ r)=0 := by
  simp only [Rheon.SphereFriction.pointVelocity,add,cross,dot]
  ring

theorem balanced_kick_drift (c h m net : ℝ) (hnet : net=0) :
    Rheon.RigidMotion.drift c h (0+h*net/m)=c := by
  rw [hnet]
  simp [Rheon.RigidMotion.drift]

theorem stationary_energy_and_potential (m i : ℝ) (force c after : Vec)
    (hpose : after=c) :
    Rheon.SphereFriction.totalEnergy m i ⟨0,0,0⟩ ⟨0,0,0⟩-dot force after =
    -dot force c := by
  rw [hpose]
  simp only [Rheon.SphereFriction.totalEnergy,squared,dot]
  ring

/-- The exact tangent plane gives a Pythagorean identity at every facet point.
Face membership and coplanarity are explicit premises, not native refinement. -/
theorem tangent_plane_distance (c p x : Vec) (radius : ℝ)
    (touch : squared (sub c p)=radius^2)
    (plane : dot (sub c p) (sub x p)=0) :
    squared (sub c x)=radius^2+squared (sub p x) := by
  simp only [squared, sub, dot] at *
  nlinarith

/-- Physical interior membership/support is assumed. This result reuses the
real separating-plane theorem; it does not refine the native sign kernel. -/
theorem stationary_facet_clearance (c p x : Vec) (radius : ℝ) (hr : 0 < radius)
    (touch : squared (sub c p)=radius^2)
    (support : dot (sub c p) (sub x p)≤0) :
    radius^2 ≤ squared (sub c x) := by
  exact Rheon.SphereInterval.supported_distance c p x radius hr (le_of_eq touch.symm) support
end
end Rheon.SphereSupport
