import Rheon.Discrete
import Mathlib.Tactic.Linarith

/-! Exact-real overlap, volume complement and shared-face algebra. These
contracts do not prove Rust mesh admission, IEEE/predicate robustness, component
construction, arbitrary cut meshes or a pressure/viscous solver refinement.
Cells are intervals at represented world endpoints; zero-volume cells are not
pressure unknowns. Active graph weights and component labels are assumptions. -/
namespace Rheon.StaticObstacle
noncomputable section

def overlap (a b c d : ℝ) : ℝ := max 0 (min b d - max a c)

theorem overlap_nonnegative (a b c d : ℝ) : 0 ≤ overlap a b c d := by
  exact le_max_left _ _

theorem overlap_symmetric (a b c d : ℝ) : overlap a b c d = overlap c d a b := by
  simp only [overlap, min_comm b d, max_comm a c]

theorem overlap_bounded (a b c d : ℝ) (interval : a ≤ b) :
    overlap a b c d ≤ b - a := by
  apply max_le
  · linarith
  · have h₁ := min_le_left b d
    have h₂ := le_max_left a c
    linarith

def fluidVolume (hx hy hz dx dy dz : ℝ) : ℝ := hx * hy * hz - dx * dy * dz

theorem fluid_volume_bounds (hx hy hz dx dy dz : ℝ)
    (hx0 : 0 ≤ hx) (hy0 : 0 ≤ hy) (hz0 : 0 ≤ hz)
    (dx0 : 0 ≤ dx) (dy0 : 0 ≤ dy) (dz0 : 0 ≤ dz)
    (dxh : dx ≤ hx) (dyh : dy ≤ hy) (dzh : dz ≤ hz) :
    0 ≤ fluidVolume hx hy hz dx dy dz ∧ fluidVolume hx hy hz dx dy dz ≤ hx * hy * hz := by
  have lower : 0 ≤ dx * dy * dz := mul_nonneg (mul_nonneg dx0 dy0) dz0
  have upper : dx * dy * dz ≤ hx * hy * hz := by
    calc
      dx * dy * dz ≤ hx * dy * dz := mul_le_mul_of_nonneg_right (mul_le_mul_of_nonneg_right dxh dy0) dz0
      _ ≤ hx * hy * dz := mul_le_mul_of_nonneg_right (mul_le_mul_of_nonneg_left dyh hx0) dz0
      _ ≤ hx * hy * hz := mul_le_mul_of_nonneg_left dzh (mul_nonneg hx0 hy0)
  unfold fluidVolume
  constructor <;> linarith

theorem shared_flux_cancels (area speed : ℝ) : area * speed + (-(area * speed)) = 0 := by ring

theorem component_constant_jump_zero {n m : ℕ}
    (tail head : Fin m → Fin n) (weight : Fin m → ℝ)
    (label : Fin n → ℕ) (value : ℕ → ℝ)
    (connected : ∀ e, weight e ≠ 0 → label (tail e) = label (head e)) :
    ∀ e, weight e * (value (label (head e)) - value (label (tail e))) = 0 := by
  intro e
  by_cases zero : weight e = 0
  · simp [zero]
  · rw [connected e zero]
    simp

theorem residual_divergence_scale (volume dt residual divergence : ℝ)
    (active : 0 < volume) (integrated : volume * divergence = dt * residual) :
    divergence = dt * residual / volume := by
  apply (eq_div_iff (ne_of_gt active)).mpr
  simpa only [mul_comm] using integrated

end
end Rheon.StaticObstacle
