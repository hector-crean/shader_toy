use pdbtbx::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_backbone() -> Result<(), PDBError> {
        let (mut pdb, _errors) = pdbtbx::open(
            "assets/AF-H2NHM8-F1-model_v4.cif",
            pdbtbx::StrictnessLevel::Loose,
        )
        .unwrap();
        // You can loop over all atoms within 3.5 Aͦ of a specific atom
        // Note: The `locate_within_distance` method takes a squared distance
        let tree = pdb.create_atom_rtree();

        Ok(())
    }
}
