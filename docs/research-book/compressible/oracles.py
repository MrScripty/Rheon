"""Analytic continuum checks, independent of reference flux/time stepping.

Sod: scalar pressure matching and self-similar rarefaction/shock relations.
Acoustics: linear ideal-gas sound modes; only valid to first order in amplitude.
These formulas are original implementations of public mathematical relations.
"""

import numpy as np


def acoustic_density(shape, time, amplitude=1e-6, gamma=1.4):
    coordinates = np.meshgrid(*[(np.arange(n) + 0.5) / n for n in shape], indexing="ij")
    density = np.ones(shape)
    for mode, weight, phase in (((1, 1, 1), 1.0, 0.0), ((1, -1, 2), 0.5, np.pi / 3)):
        argument = 2 * np.pi * sum(k * x for k, x in zip(mode, coordinates))
        argument -= 2 * np.pi * np.linalg.norm(mode) * np.sqrt(gamma) * time
        density += amplitude * weight * np.sin(argument + phase)
    return density


def acoustic_phase_speed(initial_density, final_density, time):
    # Orthogonal Fourier projection isolates k=(1,1,1) from the second mode.
    coordinates = np.meshgrid(*[(np.arange(n) + 0.5) / n for n in initial_density.shape], indexing="ij")
    basis = np.exp(-2j * np.pi * sum(coordinates))
    start = np.mean((initial_density - 1) * basis)
    end = np.mean((final_density - 1) * basis)
    phase = np.angle(end / start)
    return float(-phase / (2 * np.pi * np.sqrt(3) * time)), float(abs(end) / abs(start))


def sod_star(gamma=1.4):
    left, right = (1.0, 0.0, 1.0), (0.125, 0.0, 0.1)

    def pressure_curve(p, side):
        rho, _, p0 = side
        sound = np.sqrt(gamma * p0 / rho)
        if p > p0:
            return (p - p0) * np.sqrt(2 / ((gamma + 1) * rho) / (p + (gamma - 1) * p0 / (gamma + 1)))
        return 2 * sound / (gamma - 1) * ((p / p0) ** ((gamma - 1) / (2 * gamma)) - 1)

    low, high = 1e-12, 1.0
    for _ in range(100):
        mid = 0.5 * (low + high)
        residual = pressure_curve(mid, left) + pressure_curve(mid, right)
        if residual > 0:
            high = mid
        else:
            low = mid
    pressure = 0.5 * (low + high)
    velocity = 0.5 * (pressure_curve(pressure, right) - pressure_curve(pressure, left))
    return pressure, velocity


def sod_exact(x, time, gamma=1.4):
    """Exact isolated Sod fan at x=0.5; valid before periodic image arrives."""
    x = np.asarray(x, dtype=float)
    if time <= 0:
        return np.where(x < 0.5, 1.0, 0.125), np.zeros_like(x), np.where(x < 0.5, 1.0, 0.1)
    pstar, ustar = sod_star(gamma)
    sound_left = np.sqrt(gamma)
    star_sound = sound_left * pstar ** ((gamma - 1) / (2 * gamma))
    rho_left_star = pstar ** (1 / gamma)
    ratio = pstar / 0.1
    beta = (gamma - 1) / (gamma + 1)
    rho_right_star = 0.125 * (ratio + beta) / (beta * ratio + 1)
    shock_speed = np.sqrt(gamma * 0.1 / 0.125) * np.sqrt((gamma + 1) * ratio / (2 * gamma) + (gamma - 1) / (2 * gamma))
    xi = (x - 0.5) / time
    rho, velocity, pressure = np.full_like(x, 0.125), np.zeros_like(x), np.full_like(x, 0.1)
    left = xi <= -sound_left
    rho[left], pressure[left] = 1.0, 1.0
    fan = (xi > -sound_left) & (xi <= ustar - star_sound)
    velocity[fan] = 2 * (sound_left + xi[fan]) / (gamma + 1)
    sound_fan = 2 * (sound_left - (gamma - 1) * xi[fan] / 2) / (gamma + 1)
    rho[fan] = (sound_fan / sound_left) ** (2 / (gamma - 1))
    pressure[fan] = (sound_fan / sound_left) ** (2 * gamma / (gamma - 1))
    left_star = (xi > ustar - star_sound) & (xi <= ustar)
    rho[left_star], velocity[left_star], pressure[left_star] = rho_left_star, ustar, pstar
    right_star = (xi > ustar) & (xi <= shock_speed)
    rho[right_star], velocity[right_star], pressure[right_star] = rho_right_star, ustar, pstar
    return rho, velocity, pressure
