use std::convert::Infallible;
use std::io::Cursor;
use std::str::{from_utf8, Utf8Error};

use bevy::asset::saver::{AssetSaver, SavedAsset};
use bevy::asset::transformer::{AssetTransformer, TransformedAsset};
use bevy::asset::AsyncWriteExt;
use bevy::utils::thiserror;
use bevy::{
    asset::{
        io::{Reader, Writer},
        ron, AssetLoader, AsyncReadExt, LoadContext,
    },
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use pdbtbx::{open_mmcif_raw, open_pdb_raw, PDBError, TransformationMatrix, PDB};

use crate::polypeptide_plane::PolypeptidePlane;
use crate::polypeptide_planes::PolypeptidePlanes;

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct ProteinAsset {
    pub pdb: PDB,
    pub polypeptide_planes: PolypeptidePlanes,
}

#[derive(Default)]
pub struct ProteinAssetLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ProteinAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse Pdb: {error_log:?}")]
    PdbError { error_log: Vec<PDBError> },
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
}

#[derive(Default, Serialize, Deserialize)]
pub struct ProteinAssetSettings {}

impl AssetLoader for ProteinAssetLoader {
    type Asset = ProteinAsset;
    type Settings = ();
    type Error = ProteinAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // // Create a Cursor<Vec<u8>>
            // let cursor = Cursor::new(bytes);
            // // Create a BufReader from the Cursor
            // let buf_reader = std::io::BufReader::new(cursor);

            let str = from_utf8(&bytes)?;

            let (mut pdb, _) = open_mmcif_raw(str, pdbtbx::StrictnessLevel::Loose)
                .map_err(|error_log| ProteinAssetLoaderError::PdbError { error_log })?;

            let ((x1, y1, z1), (x2, y2, z2)) = pdb.bounding_box();

            let centre = (0.5 * (x1 + x2), 0.5 * (y1 + y2), 0.5 * (z1 + z2));

            info!("centre of protein at: {:?}", &centre);

            pdb.apply_transformation(&TransformationMatrix::translation(
                -centre.0, -centre.1, -centre.2,
            ));

            let mut polypeptide_planes = Vec::<PolypeptidePlane>::new();

            let residues: Vec<_> = pdb.residues().collect();

            let residue_triptych_iter = residues.windows(3);

            for residue_triptych in residue_triptych_iter {
                match *residue_triptych {
                    [r1, r2, r3] => {
                        match PolypeptidePlane::try_from((r1.clone(), r2.clone(), r3.clone())) {
                            Ok(polypeptide_plane) => {
                                // info!("{:?}", &polypeptide_plane);
                                polypeptide_planes.push(polypeptide_plane);
                            }
                            Err(err) => {
                                info!("{:?}", err)
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }

            Ok(ProteinAsset {
                pdb,
                polypeptide_planes: PolypeptidePlanes::new(polypeptide_planes),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["cif"]
    }
}
