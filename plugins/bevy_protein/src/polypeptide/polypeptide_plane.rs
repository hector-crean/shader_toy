/**
* Work out the bezier etc. through the polypeptide backbone

*/
use bevy::math::Vec3;
use bevy_geometry::TangentSpace;
use pdbtbx::*;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PolypeptidePlane {
    pub r1: Residue,
    pub r2: Residue,
    pub r3: Residue,
    pub tangent_space: TangentSpace,
    pub width: f32,
}

impl PolypeptidePlane {
    pub fn new(
        r1: Residue,
        r2: Residue,
        r3: Residue,
        position: Vec3,
        normal: Vec3,
        tangent: Vec3,
        binormal: Vec3,
        width: f32,
    ) -> Self {
        Self {
            r1,
            r2,
            r3,
            tangent_space: TangentSpace::new(position, normal, tangent, binormal),
            width,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PolypeptidePlaneError {
    #[error("the Atom `{0}` is not present")]
    AtomNotPresent(String),
}

impl TryFrom<(Residue, Residue, Residue)> for PolypeptidePlane {
    type Error = PolypeptidePlaneError;

    fn try_from((r1, r2, r3): (Residue, Residue, Residue)) -> Result<Self, Self::Error> {
        let ca1 = r1
            .atoms()
            .find(|atom| atom.name() == "CA")
            .ok_or(PolypeptidePlaneError::AtomNotPresent("CA".to_string()))?;
        let ca2 = r2
            .atoms()
            .find(|atom| atom.name() == "CA")
            .ok_or(PolypeptidePlaneError::AtomNotPresent("CA".to_string()))?;
        let o1 = r1
            .atoms()
            .find(|atom| atom.name() == "O")
            .ok_or(PolypeptidePlaneError::AtomNotPresent("O".to_string()))?;

        let (x1, y1, z1) = ca1.pos();
        let ca1_position = Vec3::new(x1 as f32, y1 as f32, z1 as f32);

        let (x2, y2, z2) = ca2.pos();
        let ca2_position = Vec3::new(x2 as f32, y2 as f32, z2 as f32);

        let (x3, y3, z3) = o1.pos();
        let o_position = Vec3::new(x3 as f32, y3 as f32, z3 as f32);

        //tangent
        let a = (ca2_position - ca1_position).normalize();

        let width = (o_position - ca1_position).length();
        let b = (o_position - ca1_position).normalize();

        //normal
        let c = a.cross(b).normalize();

        //binormal
        let d = c.cross(a).normalize();

        //plane_centre
        let p = 0.5 * (ca1_position + ca2_position);

        let polypeptide_plane = Self::new(r1, r2, r3, p, c, a, d, width);

        Ok(polypeptide_plane)
    }
}
