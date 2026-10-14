"""Original, bounded educational ideal-gas Euler finite-volume reference.

State layout is (nx, ny, nz, 5): rho, rho*u, rho*v, rho*w, total energy.
All boundaries are periodic. There is no floor, clipping, or positivity repair.
"""

from dataclasses import dataclass
import numpy as np


@dataclass(frozen=True)
class Config:
    shape: tuple[int, int, int] = (12, 12, 12)
    gamma: float = 1.4
    cfl: float = 0.25

    def __post_init__(self):
        if len(self.shape) != 3 or any(type(n) is not int or n < 2 or n > 64 for n in self.shape):
            raise ValueError("shape must contain three integer cell counts in [2, 64]")
        if not np.isfinite(self.gamma) or self.gamma <= 1:
            raise ValueError("gamma must be finite and greater than one")
        if not np.isfinite(self.cfl) or not 0 < self.cfl <= 0.3:
            raise ValueError("cfl must be finite in (0, 0.3]")

    @property
    def spacing(self):
        return np.reciprocal(np.asarray(self.shape, dtype=float))


def centers(config: Config):
    return np.meshgrid(*[(np.arange(n) + 0.5) / n for n in config.shape], indexing="ij")


def conservative(density, velocity, pressure, gamma=1.4):
    density, pressure = np.broadcast_arrays(density, pressure)
    velocity = np.broadcast_to(velocity, density.shape + (3,))
    state = np.empty(density.shape + (5,), dtype=float)
    state[..., 0] = density
    state[..., 1:4] = density[..., None] * velocity
    state[..., 4] = pressure / (gamma - 1) + 0.5 * density * np.sum(velocity**2, axis=-1)
    primitive(state, gamma)
    return state


def primitive(state, gamma=1.4):
    if not np.isfinite(gamma) or gamma <= 1:
        raise ValueError("gamma must be finite and greater than one")
    if state.ndim != 4 or state.shape[-1] != 5 or not np.all(np.isfinite(state)):
        raise ValueError("invalid state: expected finite (nx, ny, nz, 5) array")
    density = state[..., 0]
    if np.any(density <= 0):
        raise ValueError("invalid state: nonpositive density")
    velocity = state[..., 1:4] / density[..., None]
    pressure = (gamma - 1) * (state[..., 4] - 0.5 * np.sum(state[..., 1:4] * velocity, axis=-1))
    if not np.all(np.isfinite(pressure)) or np.any(pressure <= 0):
        raise ValueError("invalid state: nonpositive or nonfinite pressure")
    return density, velocity, pressure


def flux(state, axis, gamma):
    density, velocity, pressure = primitive(state, gamma)
    normal_velocity = velocity[..., axis]
    result = state * normal_velocity[..., None]
    result[..., 1 + axis] += pressure
    result[..., 4] += pressure * normal_velocity
    wavespeed = np.abs(normal_velocity) + np.sqrt(gamma * pressure / density)
    return result, wavespeed


def stable_dt(state, config):
    density, velocity, pressure = primitive(state, config.gamma)
    sound = np.sqrt(config.gamma * pressure / density)
    # The unsplit update uses the SUM of directional characteristic rates.
    rates = np.sum((np.abs(velocity) + sound[..., None]) / config.spacing, axis=-1)
    return float(config.cfl / np.max(rates))


def step(state, dt, config):
    if state.shape != config.shape + (5,):
        raise ValueError("state shape disagrees with config")
    if not np.isfinite(dt) or dt <= 0 or dt > stable_dt(state, config) * (1 + 1e-12):
        raise ValueError("dt must be positive and obey summed-direction CFL")
    change = np.zeros_like(state)
    for axis, dx in enumerate(config.spacing):
        left_flux, left_speed = flux(state, axis, config.gamma)
        right = np.roll(state, -1, axis=axis)
        right_flux = np.roll(left_flux, -1, axis=axis)
        speed = np.maximum(left_speed, np.roll(left_speed, -1, axis=axis))
        interface = 0.5 * (left_flux + right_flux) - 0.5 * speed[..., None] * (right - state)
        change -= (interface - np.roll(interface, 1, axis=axis)) / dx
    updated = state + dt * change
    primitive(updated, config.gamma)  # Reject the entire step on failure; never clip.
    return updated


def integrate(initial, config, output_times):
    times = np.asarray(output_times, dtype=float)
    if times.ndim != 1 or len(times) == 0 or times[0] != 0 or not np.all(np.isfinite(times)):
        raise ValueError("output times must start at zero and be finite")
    if np.any(np.diff(times) <= 0) or times[-1] > 1:
        raise ValueError("output times must increase and end by dimensionless t=1")
    state = np.asarray(initial, dtype=float).copy()
    if state.shape != config.shape + (5,):
        raise ValueError("state shape disagrees with config")
    primitive(state, config.gamma)
    frames = [(0.0, state.copy(), 0)]
    time, steps = 0.0, 0
    for target in times[1:]:
        while time < target:
            dt = min(stable_dt(state, config), float(target - time))
            state = step(state, dt, config)
            time = float(target) if dt == target - time else time + dt
            steps += 1
            if steps > 10000:
                raise RuntimeError("bounded reference exceeded 10000 steps")
        frames.append((time, state.copy(), steps))
    return frames


def sod_initial(config):
    x, _, _ = centers(config)
    density = np.where(x < 0.5, 1.0, 0.125)
    pressure = np.where(x < 0.5, 1.0, 0.1)
    return conservative(density, np.zeros(config.shape + (3,)), pressure, config.gamma)


ACOUSTIC_MODES = ((1, 1, 1), (1, -1, 2))


def acoustic_initial(config, amplitude=0.02):
    if not np.isfinite(amplitude) or not 0 < amplitude <= 0.05:
        raise ValueError("acoustic amplitude must be in (0, 0.05]")
    coordinates = centers(config)
    perturbation = np.zeros(config.shape)
    velocity = np.zeros(config.shape + (3,))
    sound = np.sqrt(config.gamma)
    for mode, weight, phase in zip(ACOUSTIC_MODES, (1.0, 0.5), (0.0, np.pi / 3)):
        direction = np.asarray(mode, dtype=float)
        wave = amplitude * weight * np.sin(2 * np.pi * sum(k * x for k, x in zip(mode, coordinates)) + phase)
        perturbation += wave
        velocity += sound * wave[..., None] * direction / np.linalg.norm(direction)
    return conservative(1 + perturbation, velocity, 1 + config.gamma * perturbation, config.gamma)


def diagnostics(state, config, initial_totals=None, steps=0):
    density, velocity, pressure = primitive(state, config.gamma)
    totals = np.sum(state, axis=(0, 1, 2)) / np.prod(config.shape)
    drift = 0.0 if initial_totals is None else np.max(np.abs(totals - initial_totals) / np.maximum(np.abs(initial_totals), 1))
    return {"mass": float(totals[0]), "momentum": totals[1:4].tolist(), "energy": float(totals[4]),
            "conserved_totals": totals.tolist(), "min_density": float(np.min(density)),
            "min_pressure": float(np.min(pressure)), "max_speed": float(np.max(np.linalg.norm(velocity, axis=-1))),
            "relative_conservation_drift": float(drift), "steps": steps}
