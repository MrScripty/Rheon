import Mathlib.Algebra.Order.BigOperators.Ring.Finset
import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring

/-! Exact-real P1 load algebra. The moment theorem explicitly assumes the
triangle's barycentric integrals. Geometry, floating arithmetic, mesh topology,
pressure interpolation and fluid reaction are not proved here. -/
namespace Rheon.MeshTraction
noncomputable section
structure Vec where
  x : ℝ
  y : ℝ
  z : ℝ

def dot (a b : Vec) : ℝ := a.x*b.x + a.y*b.y + a.z*b.z
def cross (a b : Vec) : Vec :=
  ⟨a.y*b.z-a.z*b.y, a.z*b.x-a.x*b.z, a.x*b.y-a.y*b.x⟩
def add (a b : Vec) : Vec := ⟨a.x+b.x,a.y+b.y,a.z+b.z⟩
def sub (a b : Vec) : Vec := ⟨a.x-b.x,a.y-b.y,a.z-b.z⟩
/-- One component of the first corner's consistent force; other corners relabel. -/
def nodal (A a b c : ℝ) : ℝ := A/12*(a+b+c+a)

/-- With exact P1 second moments, the integral of lambda_0*t equals the
consistent first-corner force. Integral/area correspondence is a premise. -/
theorem conditional_moment_reduction (A a b c m00 m01 m02 : ℝ)
    (h00 : m00=A/6) (h01 : m01=A/12) (h02 : m02=A/12) :
    m00*a+m01*b+m02*c = nodal A a b c := by
  rw [h00,h01,h02]
  unfold nodal
  ring

theorem resultant_closed_form (A a b c : ℝ) :
    nodal A a b c + nodal A b a c + nodal A c a b = A/3*(a+b+c) := by
  unfold nodal
  ring

theorem point_virtual_work (r f V omega : Vec) :
    dot f (add V (cross omega r)) = dot V f + dot omega (cross r f) := by
  simp only [dot,add,cross]
  ring

/-- Triangle virtual work for three consistent forces derived from actual
moments. This does not prove a fluid-to-mesh transfer formula. -/
theorem triangle_virtual_work (r0 r1 r2 f0 f1 f2 V omega : Vec) :
    dot f0 (add V (cross omega r0)) + dot f1 (add V (cross omega r1)) +
      dot f2 (add V (cross omega r2)) =
    dot V (add (add f0 f1) f2) +
      dot omega (add (add (cross r0 f0) (cross r1 f1)) (cross r2 f2)) := by
  simp only [dot,add,cross]
  ring

theorem reference_shift (r f s : Vec) : cross (sub r s) f = sub (cross r f) (cross s f) := by
  simp only [cross,sub]
  congr 1 <;> ring
end
end Rheon.MeshTraction
