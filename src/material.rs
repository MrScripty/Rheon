//! SI material values for the particle preview. Presets are approximate
//! reference-state starting points, not temperature-dependent property tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidPhase {
    Liquid,
    Gas,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluidMaterial {
    pub phase: FluidPhase,
    /// kg/m³ at the supplied reference temperature.
    pub density: f64,
    /// Dynamic Newtonian viscosity, Pa·s.
    pub viscosity: f64,
    /// N/m; used by a pairwise surface-cohesion approximation for liquids.
    pub surface_tension: f64,
    /// Tangential velocity damping rate at a wall, s⁻¹. Not a contact angle.
    pub wall_drag: f64,
    pub temperature_kelvin: f64,
    /// kg/mol; the ideal-gas constant is R / molar_mass.
    pub molar_mass: f64,
}
impl FluidMaterial {
    pub fn is_valid(self) -> bool {
        self.density.is_finite()
            && self.density > 0.0
            && self.viscosity.is_finite()
            && self.viscosity >= 0.0
            && self.surface_tension.is_finite()
            && self.surface_tension >= 0.0
            && self.wall_drag.is_finite()
            && self.wall_drag >= 0.0
            && self.temperature_kelvin.is_finite()
            && self.temperature_kelvin > 0.0
            && self.molar_mass.is_finite()
            && self.molar_mass > 0.0
            && (self.viscosity / self.density).is_finite()
            && (self.surface_tension / self.density).is_finite()
            && (self.gas_constant() * self.temperature_kelvin).is_normal()
    }
    pub fn gas_constant(self) -> f64 {
        8.314_462_618_153_24 / self.molar_mass
    }
    /// Liquid preview stiffness is deliberately reduced, not a physical
    /// acoustic measurement. Gases use their isothermal ideal-gas sound speed.
    pub fn sound_speed(self, temperature: f64, liquid_speed: f64) -> f64 {
        match self.phase {
            FluidPhase::Liquid => liquid_speed,
            FluidPhase::Gas => (self.gas_constant() * temperature).sqrt(),
        }
    }
    pub fn pressure(self, density: f64, temperature: f64, liquid_speed: f64) -> f64 {
        match self.phase {
            FluidPhase::Liquid => (liquid_speed * liquid_speed * (density - self.density)).max(0.0),
            FluidPhase::Gas => density * self.gas_constant() * temperature,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidPreset {
    Water,
    Mercury,
    MoltenAluminium,
    MoltenTin,
    MoltenIron,
    Air,
    Argon,
    Helium,
    Slime,
    Syrup,
}
impl FluidPreset {
    pub const ALL: [Self; 10] = [
        Self::Water,
        Self::Mercury,
        Self::MoltenAluminium,
        Self::MoltenTin,
        Self::MoltenIron,
        Self::Air,
        Self::Argon,
        Self::Helium,
        Self::Slime,
        Self::Syrup,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Water => "Water · 20 °C",
            Self::Mercury => "Mercury · 20 °C",
            Self::MoltenAluminium => "Molten aluminium · 700 °C",
            Self::MoltenTin => "Molten tin · 300 °C",
            Self::MoltenIron => "Molten iron · 1,600 °C",
            Self::Air => "Air · 20 °C",
            Self::Argon => "Argon · 20 °C",
            Self::Helium => "Helium · 20 °C",
            Self::Slime => "Slime-like · viscous approximation",
            Self::Syrup => "Syrup · viscous approximation",
        }
    }
    pub fn material(self) -> FluidMaterial {
        let mut m = FluidMaterial {
            phase: FluidPhase::Liquid,
            density: 998.2,
            viscosity: 0.001_002,
            surface_tension: 0.0728,
            wall_drag: 5.0,
            temperature_kelvin: 293.15,
            molar_mass: 0.018_015_28,
        };
        match self {
            Self::Water => {}
            Self::Mercury => {
                m.density = 13_546.0;
                m.viscosity = 0.001_526;
                m.surface_tension = 0.485;
                m.molar_mass = 0.200_59;
            }
            Self::MoltenAluminium => {
                m.density = 2375.0;
                m.viscosity = 0.0013;
                m.surface_tension = 0.86;
                m.temperature_kelvin = 973.15;
                m.molar_mass = 0.026_981_538_5;
            }
            Self::MoltenTin => {
                m.density = 6950.0;
                m.viscosity = 0.0018;
                m.surface_tension = 0.55;
                m.temperature_kelvin = 573.15;
                m.molar_mass = 0.118_71;
            }
            Self::MoltenIron => {
                m.density = 7000.0;
                m.viscosity = 0.006;
                m.surface_tension = 1.85;
                m.temperature_kelvin = 1873.15;
                m.molar_mass = 0.055_845;
            }
            Self::Air | Self::Argon | Self::Helium => {
                m.phase = FluidPhase::Gas;
                m.surface_tension = 0.0;
                m.wall_drag = 0.0;
                (m.molar_mass, m.viscosity) = match self {
                    Self::Air => (0.028_965_46, 0.000_018_2),
                    Self::Argon => (0.039_948, 0.000_022_3),
                    _ => (0.004_002_602, 0.000_019_6),
                };
                m.density = 101_325.0 / (m.gas_constant() * m.temperature_kelvin);
            }
            Self::Slime => {
                m.density = 1100.0;
                m.viscosity = 25.0;
                m.surface_tension = 0.06;
                m.wall_drag = 25.0;
            }
            Self::Syrup => {
                m.density = 1400.0;
                m.viscosity = 3.0;
                m.surface_tension = 0.08;
                m.wall_drag = 10.0;
            }
        }
        m
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainFill {
    Vacuum,
    EmittedFluid,
    OtherFluid,
}

#[derive(Debug, Clone, Copy)]
pub struct ParticleFluidConfig {
    pub material: FluidMaterial,
    pub fill: DomainFill,
    pub ambient_material: FluidMaterial,
    pub ambient_temperature_kelvin: f64,
    pub gravity: [f64; 3],
    /// Approximate samples across the longest domain side, separate from
    /// preview voxels. Particle support is isotropic in physical metres.
    pub samples_per_axis: usize,
    pub max_particles: usize,
    pub memory_limit: usize,
    /// Reduced acoustic speed used for weakly compressible liquid previews.
    pub liquid_sound_speed: f64,
    /// Newtonian temperature relaxation on wall contact, s⁻¹.
    pub wall_heat_exchange: f64,
}
impl Default for ParticleFluidConfig {
    fn default() -> Self {
        Self {
            material: FluidPreset::Water.material(),
            fill: DomainFill::Vacuum,
            ambient_material: FluidPreset::Air.material(),
            ambient_temperature_kelvin: 293.15,
            gravity: [0.0, -9.80665, 0.0],
            samples_per_axis: 16,
            max_particles: 65536,
            memory_limit: 192 * 1024 * 1024,
            liquid_sound_speed: 20.0,
            wall_heat_exchange: 1.0,
        }
    }
}
