import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Positivity

/-! Exact finite teaching contracts. Unit volumes, spacing and density are
specified by the lesson. No mesh reader, geometric quadrature, general
Rhie–Chow/SIMPLE convergence, PDE, Rust/JS or IEEE refinement is proved. -/
namespace Rheon.EngineeringFlux
noncomputable section

theorem shared_face_cancellation (flux component : ℝ) :
    flux * component + -(flux * component) = 0 := by ring

theorem two_cell_update_conserves (left right dt flux : ℝ) :
    (left - dt * flux) + (right + dt * flux) = left + right := by ring

theorem boundary_work (area velocity q pL pR : ℝ) :
    pL * (area * velocity + q) + pR * (-area * velocity - q) =
      -(pR - pL) * area * velocity + (pL - pR) * q := by ring

theorem balanced_flow (area velocity pL pR : ℝ) :
    -(pR - pL) * area * velocity + (pL - pR) * (-area * velocity) = 0 := by ring

theorem omitted_boundary_counterexample :
    (1 : ℝ) * (1/2 + 1/2) + 3 * (-1/2 - 1/2) = -2 ∧
    -(3 - 1) * (1/2 : ℝ) = -1 := by norm_num

theorem checkerboard_central_blind (a h : ℝ) :
    (-a - -a) / (2*h) = 0 ∧ (a-a) / (2*h) = 0 := by simp

theorem checkerboard_face_response (a r : ℝ) :
    -r * (-a-a) = 2*r*a ∧ -r * (a- -a) = -2*r*a := by constructor <;> ring

theorem checkerboard_divergence (a r : ℝ) :
    (2*r*a)-(-2*r*a) = 4*r*a := by ring

theorem periodic_pressure_energy (a b c d r : ℝ) :
    a*(r*(2*a-b-d)) + b*(r*(2*b-c-a)) +
      c*(r*(2*c-d-b)) + d*(r*(2*d-a-c)) =
      r*((b-a)^2+(c-b)^2+(d-c)^2+(a-d)^2) := by ring

def planarPower (mu a b c d : ℝ) : ℝ :=
    2*mu*a^2 + 2*mu*d^2 + mu*(b+c)^2

theorem planar_power_nonnegative (mu a b c d : ℝ) (hmu : 0 ≤ mu) :
    0 ≤ planarPower mu a b c d := by unfold planarPower; positivity

-- Face-centre quadrature for the specified centred unit cube reduces to
-- tau:gradient. Quadrature correctness for arbitrary geometry is not asserted.
theorem planar_boundary_power (mu a b c d : ℝ) :
    (2*mu*a)*a + (mu*(b+c))*b + (mu*(b+c))*c + (2*mu*d)*d =
      planarPower mu a b c d := by unfold planarPower; ring

theorem rigid_rotation_power (mu rate : ℝ) :
    planarPower mu 0 (-rate) rate 0 = 0 := by unfold planarPower; ring

theorem simple_shear_power (mu rate : ℝ) :
    planarPower mu 0 rate 0 0 = mu*rate^2 := by unfold planarPower; ring

theorem planar_extension_power (mu rate : ℝ) :
    planarPower mu rate 0 0 (-rate) = 4*mu*rate^2 := by unfold planarPower; ring

end
end Rheon.EngineeringFlux
