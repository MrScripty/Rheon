import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith

/-! Exact-real conditional contracts for the explicit symmetric-strain stage.
The small operator is the unit-spacing 2x2x1 free-slip MAC assembly. General
mesh assembly, spectral bound, IEEE rounding and physical validation are not
proved. The original implicit viscosity contracts remain unchanged. -/
namespace Rheon.BoxViscosity
noncomputable section

theorem mac_patch_work (a b c d : ℝ) :
    a*(4*a-(b-a+d-c)) + b*(4*b+(b-a+d-c)) +
    c*(4*c-(b-a+d-c)) + d*(4*d+(b-a+d-c)) =
    4*(a^2+b^2+c^2+d^2)+(b-a+d-c)^2 := by ring

theorem mac_patch_dissipation_nonnegative (a b c d : ℝ) :
    0 ≤ 4*(a^2+b^2+c^2+d^2)+(b-a+d-c)^2 := by positivity

theorem mac_patch_symmetry (a b c d x y z w : ℝ) :
    x*(4*a-(b-a+d-c)) + y*(4*b+(b-a+d-c)) +
    z*(4*c-(b-a+d-c)) + w*(4*d+(b-a+d-c)) =
    a*(4*x-(y-x+w-z)) + b*(4*y+(y-x+w-z)) +
    c*(4*z-(y-x+w-z)) + d*(4*w+(y-x+w-z)) := by ring

theorem rigid_rotation_local_shear (omega : ℝ) : -omega+omega = 0 := by ring

theorem affine_shear_local_dissipation (mu volume gamma : ℝ) :
    mu*volume*(gamma+0)^2 = mu*volume*gamma^2 := by ring

theorem explicit_coordinate_energy (mass u step k : ℝ) :
    mass/2*((u-step*k)^2-u^2) =
    -mass*step*u*k + mass/2*(step*k)^2 := by ring

theorem rounded_coordinate_energy (mass proposed rounding : ℝ) :
    mass/2*((proposed+rounding)^2-proposed^2) =
    mass*rounding*(proposed+rounding/2) := by ring

theorem explicit_energy_nonincrease (before after dt dissip update : ℝ)
    (identity : after-before = -dt*dissip+update)
    (bound : update ≤ dt*dissip) : after ≤ before := by linarith

theorem modal_factor_nonexpansive (r : ℝ) (h0 : 0 ≤ r) (h2 : r ≤ 2) :
    (1-r)^2 ≤ 1 := by nlinarith

theorem modal_step (amplitude r : ℝ) :
    amplitude-r*amplitude = (1-r)*amplitude := by ring

end
end Rheon.BoxViscosity
