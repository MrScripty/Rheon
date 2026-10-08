import Rheon.MeshTraction
import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith

/-! Finite exact-real pose algebra. Unit quaternion, exponential correspondence
and physical spherical-inertia premises are explicit; no IEEE/ODE proof. -/
namespace Rheon.RigidMotion
noncomputable section
structure Q where
  w : ℝ
  x : ℝ
  y : ℝ
  z : ℝ
def product (a b : Q) : Q :=
  ⟨a.w*b.w-a.x*b.x-a.y*b.y-a.z*b.z,
   a.w*b.x+a.x*b.w+a.y*b.z-a.z*b.y,
   a.w*b.y+a.y*b.w+a.z*b.x-a.x*b.z,
   a.w*b.z+a.z*b.w+a.x*b.y-a.y*b.x⟩
def conjugate (q : Q) : Q := ⟨q.w,-q.x,-q.y,-q.z⟩
def normSquared (q : Q) : ℝ := q.w^2+q.x^2+q.y^2+q.z^2
def drift (c h v : ℝ) : ℝ := c+h*v

theorem product_norm (a b : Q) :
    normSquared (product a b)=normSquared a*normSquared b := by
  unfold normSquared product
  ring

theorem conjugate_norm (q : Q) : normSquared (conjugate q)=normSquared q := by
  unfold normSquared conjugate
  ring

/-- Delta quaternion is left-multiplied. Its unit exponential premise is assumed,
not proved for Rust sin/cos or rounded normalization. -/
theorem conditional_unit_drift (dq q : Q)
    (hd : normSquared dq=1) (hq : normSquared q=1) :
    normSquared (product dq q)=1 := by
  rw [product_norm,hd,hq]
  ring

theorem conditional_rotation_norm (q p : Q) (hq : normSquared q=1) :
    normSquared (product (product q p) (conjugate q))=normSquared p := by
  rw [product_norm,product_norm,conjugate_norm,hq]
  ring

theorem rotation_pure (q : Q) (x y z : ℝ) :
    (product (product q ⟨0,x,y,z⟩) (conjugate q)).w=0 := by
  unfold product conjugate
  ring

theorem rotation_sign_equivalence (q p : Q) :
    product (product ⟨-q.w,-q.x,-q.y,-q.z⟩ p)
      (conjugate ⟨-q.w,-q.x,-q.y,-q.z⟩) =
    product (product q p) (conjugate q) := by
  simp only [product,conjugate]
  congr 1 <;> ring

theorem drift_displacement (c h v : ℝ) : drift c h v-c=h*v := by
  unfold drift
  ring

theorem positive_clock (t h : ℝ) (hh : 0<h) : t<t+h := by linarith

/-- Explicit finite-step error under constant acceleration. This algebra does
not infer an ODE solution or prove convergence of arbitrary force histories. -/
theorem kick_drift_constant_force_error (c v a h : ℝ) :
    drift c h (v+h*a)-(c+h*v+a*h^2/2)=a*h^2/2 := by
  unfold drift
  ring
open Rheon.MeshTraction
def scale (s : ℝ) (a : Vec) : Vec := ⟨s*a.x,s*a.y,s*a.z⟩
def totalAngular (c V omega : Vec) (m I : ℝ) : Vec :=
  add (scale I omega) (cross c (scale m V))

/-- Free COM drift adds no orbital angular momentum about the world origin. -/
theorem orbital_free_drift (c V : Vec) (h m : ℝ) :
    cross (add c (scale h V)) (scale m V)=cross c (scale m V) := by
  simp only [cross,add,scale]
  congr 1 <;> ring

theorem endpoint_angular_increment (c V V' omega omega' : Vec) (h m I : ℝ) :
    sub (totalAngular (add c (scale h V')) V' omega' m I)
      (totalAngular c V omega m I) =
    add (scale I (sub omega' omega)) (cross c (scale m (sub V' V))) := by
  simp only [totalAngular,cross,add,sub,scale]
  congr 1 <;> ring

/-- Exact matched momentum increments yield total angular impulse including
orbital momentum. Premises do not assert exact floating endpoint identities. -/
theorem conditional_total_angular_impulse (c V V' omega omega' F tau : Vec)
    (h m I : ℝ) (hP : scale m (sub V' V)=scale h F)
    (hL : scale I (sub omega' omega)=scale h tau) :
    sub (totalAngular (add c (scale h V')) V' omega' m I)
      (totalAngular c V omega m I) = scale h (add tau (cross c F)) := by
  rw [endpoint_angular_increment,hP,hL]
  simp only [cross,add,scale]
  congr 1 <;> ring
end
end Rheon.RigidMotion
