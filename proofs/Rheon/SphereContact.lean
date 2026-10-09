import Rheon.MeshTraction
import Rheon.RigidMotion
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith

/-! Finite exact-real contact algebra. No feature classification, root ordering
in IEEE arithmetic, libm, physical mass reconstruction or contact continuation
is proved. Unit normal, unique contact and physical correspondence are premises. -/
namespace Rheon.SphereContact
noncomputable section
open Rheon.MeshTraction
open Rheon.RigidMotion (scale)
def squared (v : Vec) : ℝ := dot v v
def normalSpeed (v n : Vec) : ℝ := dot v n
def response (v n : Vec) (e : ℝ) : Vec :=
  sub v (scale ((1+e)*normalSpeed v n) n)
def kinetic (m : ℝ) (v : Vec) : ℝ := m/2*squared v

theorem squared_trajectory (w d : Vec) (s R : ℝ) :
    squared (add w (scale s d))-R^2 =
    squared d*s^2+2*dot w d*s+(squared w-R^2) := by
  simp only [squared,dot,add,scale]
  ring

/-- Orthogonal projection onto the finite edge line is a geometric premise;
the resulting perpendicular vectors obey the same quadratic identity. -/
theorem perpendicular_trajectory (w d e : Vec) (s : ℝ)
    (hw : dot w e=0) (hd : dot d e=0) :
    dot (add w (scale s d)) e=0 := by
  have h : dot (add w (scale s d)) e=dot w e+s*dot d e := by
    simp only [dot,add,scale]
    ring
  rw [h,hw,hd]
  ring

theorem conditional_quadratic_root (a b c r : ℝ) (ha : a≠0)
    (hr : r^2=b^2-a*c) :
    a*((-b-r)/a)^2+2*b*((-b-r)/a)+c=0 := by
  field_simp
  nlinarith [hr]

theorem quadratic_factorization (a b c lo hi s : ℝ)
    (hb : 2*b= -a*(lo+hi)) (hc : c=a*lo*hi) :
    a*s^2+2*b*s+c=a*(s-lo)*(s-hi) := by
  rw [show 2*b*s=(2*b)*s by ring,hb,hc]
  ring

/-- Conditional first entry of one scalar distance quadratic. Exact roots and
ordering are premises; this does not establish mesh-wide floating coverage. -/
theorem quadratic_positive_before (a lo hi s : ℝ)
    (ha : 0<a) (hs : s<lo) (hh : lo≤hi) :
    0<a*(s-lo)*(s-hi) := by
  have h1 : s-lo<0 := by linarith
  have h2 : s-hi<0 := by linarith
  have hp : 0<(s-lo)*(s-hi) := mul_pos_of_neg_of_neg h1 h2
  nlinarith

theorem face_linear_root (height dh target : ℝ) (hd : dh≠0) :
    height+((target-height)/dh)*dh=target := by
  field_simp

theorem radial_impulse_zero_torque (n : Vec) (R j : ℝ) :
    cross (scale (-R) n) (scale j n)=⟨0,0,0⟩ := by
  simp only [cross,scale]
  congr 1 <;> ring

theorem unit_normal_restitution (v n : Vec) (e : ℝ)
    (hn : squared n=1) :
    normalSpeed (response v n e) n= -e*normalSpeed v n := by
  have h : normalSpeed (response v n e) n =
      normalSpeed v n-(1+e)*normalSpeed v n*squared n := by
    simp only [normalSpeed,response,sub,scale,dot,squared]
    ring
  rw [h,hn]
  ring

theorem response_momentum (v n : Vec) (m e : ℝ) :
    scale m (sub (response v n e) v)=
    scale (-(1+e)*m*normalSpeed v n) n := by
  simp only [response,scale,sub,normalSpeed,dot]
  congr 1 <;> ring

theorem unit_normal_energy_change (v n : Vec) (m e : ℝ)
    (hn : squared n=1) :
    kinetic m (response v n e)-kinetic m v =
    -(m/2)*(1-e^2)*(normalSpeed v n)^2 := by
  have h : kinetic m (response v n e)-kinetic m v =
      m/2*((1+e)^2*(normalSpeed v n)^2*squared n-
        2*(1+e)*(normalSpeed v n)^2) := by
    simp only [kinetic,response,normalSpeed,squared,sub,scale,dot]
    ring
  rw [h,hn]
  ring

theorem conditional_energy_nonincrease (v n : Vec) (m e : ℝ)
    (hm : 0<m) (he0 : 0≤e) (he1 : e≤1) (hn : squared n=1) :
    kinetic m (response v n e)≤kinetic m v := by
  have he : 0≤1-e^2 := by nlinarith
  have hv : 0≤(normalSpeed v n)^2 := sq_nonneg _
  have hprod : 0≤(m/2)*(1-e^2)*(normalSpeed v n)^2 :=
    mul_nonneg (mul_nonneg (by linarith) he) hv
  have h := unit_normal_energy_change v n m e hn
  linarith

end
end Rheon.SphereContact
