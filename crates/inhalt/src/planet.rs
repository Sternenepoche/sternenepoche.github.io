//! Seeded spherical CPU materials and orbital thumbnails, independent of GPU or browser.
use kern::{Gut, Zone};
use std::{fs, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ResourceOverlay {
    pub resource: Gut,
    /// Must come from the player's observation, never the global world state.
    pub profile_known: bool,
    /// The observed engine factor. Bands remain decorative, not gameplay deposits.
    pub factor: f64,
}

#[derive(Clone, Debug)]
pub struct PlanetOptions {
    pub seed: u32,
    pub zone: Zone,
    pub rotation: f64,
    pub overlay: Option<ResourceOverlay>,
}

impl Default for PlanetOptions {
    fn default() -> Self {
        Self {
            seed: 42,
            zone: Zone::Leben,
            rotation: 0.0,
            overlay: None,
        }
    }
}

pub struct SurfaceMaps {
    pub albedo: RgbaImage,
    pub heightmap: RgbaImage,
    pub roughness: RgbaImage,
    pub clouds: RgbaImage,
}

fn mix(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}
fn hash(x: i32, y: i32, z: i32, seed: u32) -> f64 {
    let mut h = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (z as u32).wrapping_mul(2147483647)
        ^ seed;
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    (h ^ (h >> 16)) as f64 / u32::MAX as f64
}
fn noise(x: f64, y: f64, z: f64, seed: u32) -> f64 {
    let (xx, yy, zz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let (u, v, w) = (
        smooth(x - x.floor()),
        smooth(y - y.floor()),
        smooth(z - z.floor()),
    );
    let xy = |dz| {
        mix(
            mix(
                hash(xx, yy, zz + dz, seed),
                hash(xx + 1, yy, zz + dz, seed),
                u,
            ),
            mix(
                hash(xx, yy + 1, zz + dz, seed),
                hash(xx + 1, yy + 1, zz + dz, seed),
                u,
            ),
            v,
        )
    };
    mix(xy(0), xy(1), w)
}
fn terrain(x: f64, y: f64, z: f64, seed: u32) -> f64 {
    let (mut n, mut weight, mut scale) = (0.0, 0.55, 2.6);
    for i in 0..5 {
        n += weight
            * noise(
                x * scale + 12.0,
                y * scale + 12.0,
                z * scale + 12.0,
                seed.wrapping_add(i * 103),
            );
        weight *= 0.5;
        scale *= 2.0;
    }
    n / 1.065625
}
struct Surface {
    albedo: [f64; 3],
    height: f64,
    roughness: f64,
    clouds: f64,
}
fn surface(x: f64, y: f64, z: f64, o: &PlanetOptions) -> Surface {
    let colors = match o.zone {
        Zone::Leben => [
            [12., 43., 73.],
            [29., 83., 112.],
            [62., 105., 70.],
            [141., 128., 89.],
            [210., 220., 219.],
        ],
        Zone::Glut => [
            [47., 32., 30.],
            [85., 46., 32.],
            [140., 76., 40.],
            [178., 121., 70.],
            [221., 173., 104.],
        ],
        Zone::Frost => [
            [23., 46., 67.],
            [48., 77., 99.],
            [108., 147., 163.],
            [169., 195., 201.],
            [224., 235., 233.],
        ],
    };
    let n = terrain(x, y, z, o.seed);
    let stops = [0.30, 0.46, 0.51, 0.67, 0.82];
    let mut i = 0;
    while i < 3 && n > stops[i + 1] {
        i += 1;
    }
    let blend = smooth(((n - stops[i]) / (stops[i + 1] - stops[i])).clamp(0.0, 1.0));
    let mut color = std::array::from_fn(|j| mix(colors[i][j], colors[i + 1][j], blend));
    if o.zone == Zone::Leben && y.abs() > 0.86 + n * 0.12 {
        color = colors[4];
    }
    let grain = noise(
        x * 115. + 12.,
        y * 115. + 12.,
        z * 115. + 12.,
        o.seed.wrapping_add(211),
    );
    let ridges = noise(
        x * 41. + 12.,
        y * 41. + 12.,
        z * 41. + 12.,
        o.seed.wrapping_add(487),
    );
    let relief = 0.68 + n * 0.30 + grain * 0.18 + ridges * 0.20;
    if o.zone == Zone::Glut
        && n > 0.64
        && noise(x * 29., y * 29., z * 29., o.seed.wrapping_add(88)) > 0.66
    {
        color = [229., 88., 28.];
    }
    Surface {
        albedo: color.map(|c| (c * relief).min(255.)),
        height: n,
        roughness: if o.zone == Zone::Leben && n < 0.49 {
            0.22
        } else {
            0.82
        },
        clouds: if o.zone == Zone::Leben {
            (terrain(x + 2., y + 1., z - 1., o.seed.wrapping_add(1234)) - 0.52)
                .max(0.0)
                .mul_add(3., 0.)
                .min(0.8)
        } else {
            0.
        },
    }
}
fn byte(v: f64) -> u8 {
    v.clamp(0., 255.).round() as u8
}
fn empty(width: u32, height: u32) -> RgbaImage {
    RgbaImage {
        width,
        height,
        pixels: vec![0; (width * height * 4) as usize],
    }
}
fn resource_color(resource: Gut) -> Option<[f64; 3]> {
    match resource {
        Gut::Erz => Some([246., 159., 78.]),
        Gut::Kristall => Some([89., 207., 245.]),
        Gut::Deuterium => Some([147., 143., 255.]),
        Gut::Nahrung => Some([125., 230., 115.]),
        Gut::Xenokristall => Some([231., 120., 253.]),
        _ => None,
    }
}
pub fn resource_visible(o: &PlanetOptions) -> bool {
    o.overlay.as_ref().is_some_and(|r| {
        r.profile_known
            && r.factor.is_finite()
            && r.factor > 0.
            && resource_color(r.resource).is_some()
    })
}

/// Four equirectangular maps. Exact duplicated seam columns and single-color poles.
/// No resource overlay is ever baked into these physical material channels.
pub fn surface_maps(width: u32, o: &PlanetOptions) -> Result<SurfaceMaps, String> {
    if !(4..=4096).contains(&width) || width % 2 != 0 {
        return Err("Even map width 4..4096 required".into());
    }
    let height = width / 2;
    let mut maps = SurfaceMaps {
        albedo: empty(width, height),
        heightmap: empty(width, height),
        roughness: empty(width, height),
        clouds: empty(width, height),
    };
    for py in 0..height {
        for px in 0..width {
            let lat = std::f64::consts::PI * (py as f64 / (height - 1) as f64 - 0.5);
            let lon = if px == width - 1 {
                0.
            } else {
                std::f64::consts::TAU * px as f64 / (width - 1) as f64
            };
            let c = if py == 0 || py == height - 1 {
                0.
            } else {
                lat.cos()
            };
            let s = surface(lon.sin() * c, lat.sin(), lon.cos() * c, o);
            let i = ((py * width + px) * 4) as usize;
            maps.albedo.pixels[i..i + 4].copy_from_slice(&[
                byte(s.albedo[0]),
                byte(s.albedo[1]),
                byte(s.albedo[2]),
                255,
            ]);
            for (map, value) in [
                (&mut maps.heightmap, s.height),
                (&mut maps.roughness, s.roughness),
                (&mut maps.clouds, s.clouds),
            ] {
                let v = byte(value * 255.);
                map.pixels[i..i + 4].copy_from_slice(&[v, v, v, 255]);
            }
        }
    }
    Ok(maps)
}

/// A lit planet with transparent background. Only observed resource profiles enable bands.
pub fn render_orbit(size: u32, o: &PlanetOptions) -> Result<RgbaImage, String> {
    if !(4..=4096).contains(&size) || !o.rotation.is_finite() {
        return Err("Size 4..4096 and finite rotation required".into());
    }
    let mut out = empty(size, size);
    let radius = size as f64 * 0.435;
    let (ct, st) = (o.rotation.cos(), o.rotation.sin());
    let overlay = o.overlay.as_ref().filter(|_| resource_visible(o));
    for py in 0..size {
        for px in 0..size {
            let x = (px as f64 - size as f64 / 2.) / radius;
            let y = (py as f64 - size as f64 / 2.) / radius;
            let r2 = x * x + y * y;
            let i = ((py * size + px) * 4) as usize;
            if r2 > 1.09 {
                continue;
            }
            if r2 > 1. {
                out.pixels[i..i + 4].copy_from_slice(&[
                    70,
                    137,
                    195,
                    byte(255. * (1.09 - r2) / 0.09 * 0.33),
                ]);
                continue;
            }
            let z = (1. - r2).sqrt();
            let (tx, tz) = (x * ct + z * st, z * ct - x * st);
            let s = surface(tx, y, tz, o);
            let mut color = s.albedo.map(|c| mix(c, 232., s.clouds));
            let shade = 0.10 + 0.90 * (-x * 0.55 - y * 0.4 + z * 0.63).max(0.);
            if let Some(r) = overlay {
                let band = noise(
                    tx * 7. + 9.,
                    y * 7. + 9.,
                    tz * 7. + 9.,
                    o.seed.wrapping_add(r.resource.name().len() as u32 * 701),
                );
                let alpha = if band > 0.64 {
                    ((band - 0.64) * 5. * r.factor.min(2.)).min(0.8)
                } else {
                    0.
                };
                let tint = resource_color(r.resource).unwrap();
                color = std::array::from_fn(|j| mix(color[j], tint[j], alpha));
            }
            let rim = (1. - z).powi(4) * 0.35;
            let glow = [43., 91., 145.];
            out.pixels[i..i + 4].copy_from_slice(&[
                byte(color[0] * shade + glow[0] * rim),
                byte(color[1] * shade + glow[1] * rim),
                byte(color[2] * shade + glow[2] * rim),
                255,
            ]);
        }
    }
    Ok(out)
}

impl RgbaImage {
    /// Standards-compliant PNG using uncompressed zlib blocks: no external codec/runtime.
    pub fn png(&self) -> Result<Vec<u8>, String> {
        if self.width == 0
            || self.height == 0
            || self.width as u64 * self.height as u64 * 4 != self.pixels.len() as u64
        {
            return Err("Invalid RGBA dimensions".into());
        }
        let mut rows = Vec::with_capacity(self.pixels.len() + self.height as usize);
        for row in self.pixels.chunks_exact(self.width as usize * 4) {
            rows.push(0);
            rows.extend_from_slice(row);
        }
        let mut zlib = vec![0x78, 0x01];
        let blocks = rows.chunks(65535);
        let total = blocks.len();
        for (i, block) in blocks.enumerate() {
            zlib.push(if i + 1 == total { 1 } else { 0 });
            let n = block.len() as u16;
            zlib.extend_from_slice(&n.to_le_bytes());
            zlib.extend_from_slice(&(!n).to_le_bytes());
            zlib.extend_from_slice(block);
        }
        let (mut a, mut b) = (1u32, 0u32);
        for &v in &rows {
            a = (a + v as u32) % 65521;
            b = (b + a) % 65521;
        }
        zlib.extend_from_slice(&((b << 16) | a).to_be_bytes());
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::new();
        header.extend_from_slice(&self.width.to_be_bytes());
        header.extend_from_slice(&self.height.to_be_bytes());
        header.extend_from_slice(&[8, 6, 0, 0, 0]);
        chunk(&mut png, b"IHDR", &header);
        chunk(&mut png, b"IDAT", &zlib);
        chunk(&mut png, b"IEND", &[]);
        Ok(png)
    }
    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<(), String> {
        fs::write(path, self.png()?).map_err(|e| e.to_string())
    }
}
fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc = 0xffff_ffffu32;
    for &v in kind.iter().chain(data) {
        crc ^= v as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 == 1 { 0xedb88320 } else { 0 };
        }
    }
    out.extend_from_slice(&(!crc).to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_and_distinct() {
        let o = PlanetOptions::default();
        let a = render_orbit(48, &o).unwrap();
        assert_eq!(a, render_orbit(48, &o).unwrap());
        assert_ne!(
            a,
            render_orbit(48, &PlanetOptions { seed: 43, ..o }).unwrap()
        );
        assert_eq!(&a.pixels[..4], &[0, 0, 0, 0]);
    }
    #[test]
    fn maps_have_exact_seam_and_poles() {
        for zone in Zone::ALLE {
            let m = surface_maps(
                64,
                &PlanetOptions {
                    zone,
                    ..Default::default()
                },
            )
            .unwrap();
            for img in [&m.albedo, &m.heightmap, &m.roughness, &m.clouds] {
                for y in 0..img.height {
                    let i = (y * img.width * 4) as usize;
                    assert_eq!(
                        &img.pixels[i..i + 4],
                        &img.pixels[i + (img.width as usize - 1) * 4..i + img.width as usize * 4]
                    );
                }
                for y in [0, img.height - 1] {
                    let i = (y * img.width * 4) as usize;
                    for x in 1..img.width as usize {
                        assert_eq!(&img.pixels[i..i + 4], &img.pixels[i + x * 4..i + x * 4 + 4]);
                    }
                }
            }
        }
    }
    #[test]
    fn hidden_resource_cannot_change_pixels() {
        let o = PlanetOptions::default();
        let base = render_orbit(64, &o).unwrap();
        let mut opts = PlanetOptions {
            overlay: Some(ResourceOverlay {
                resource: Gut::Xenokristall,
                profile_known: false,
                factor: 2.,
            }),
            ..o
        };
        assert_eq!(base, render_orbit(64, &opts).unwrap());
        opts.overlay.as_mut().unwrap().profile_known = true;
        assert_ne!(base, render_orbit(64, &opts).unwrap());
        opts.overlay.as_mut().unwrap().factor = f64::NAN;
        assert_eq!(base, render_orbit(64, &opts).unwrap());
    }
    #[test]
    fn overlay_never_enters_material_maps() {
        let o = PlanetOptions::default();
        let a = surface_maps(32, &o).unwrap();
        let b = surface_maps(
            32,
            &PlanetOptions {
                overlay: Some(ResourceOverlay {
                    resource: Gut::Erz,
                    profile_known: true,
                    factor: 2.,
                }),
                ..o
            },
        )
        .unwrap();
        assert_eq!(a.albedo, b.albedo);
        assert_eq!(a.heightmap, b.heightmap);
    }
    #[test]
    fn rejects_invalid_input() {
        assert!(surface_maps(5, &Default::default()).is_err());
        assert!(render_orbit(4097, &Default::default()).is_err());
        assert!(render_orbit(
            32,
            &PlanetOptions {
                rotation: f64::NAN,
                ..Default::default()
            }
        )
        .is_err());
    }
}
