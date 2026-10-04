import Mathlib.Tactic.Ring

namespace Rheon.Indexing

def flatten (nx ny i j k : ℕ) : ℕ := i + nx * (j + ny * k)

theorem flatten_in_bounds (nx ny nz i j k : ℕ)
    (hi : i < nx) (hj : j < ny) (hk : k < nz) :
    flatten nx ny i j k < nx * ny * nz := by
  unfold flatten
  have hinner : j + ny * k < ny * nz := by
    calc
      j + ny * k < ny + ny * k := Nat.add_lt_add_right hj _
      _ = ny * (k + 1) := by ring
      _ ≤ ny * nz := Nat.mul_le_mul_left ny hk
  calc
    i + nx * (j + ny * k) < nx + nx * (j + ny * k) := Nat.add_lt_add_right hi _
    _ = nx * (j + ny * k + 1) := by ring
    _ ≤ nx * (ny * nz) := Nat.mul_le_mul_left nx hinner
    _ = nx * ny * nz := by ring

theorem staggered_count (n : ℕ) :
    (n+1)*n*n + n*(n+1)*n + n*n*(n+1) = 3*n*n*(n+1) := by ring

end Rheon.Indexing
