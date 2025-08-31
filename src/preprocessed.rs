use geojson::GeoJson;

use crate::tile::Tile;
use crate::types::VtFeatures;
use crate::{clip, convert, geojson_to_feature_collection, wrap, InternalTile, TileOptions};

#[derive(Clone)]
pub struct PreprocessedGeoJSON {
    features: VtFeatures,
    max_zoom: u8,
    tile_options: TileOptions,
}

impl PreprocessedGeoJSON {
    pub fn new(geojson: &GeoJson, max_zoom: u8, tile_options: &TileOptions) -> Self {
        let collection = geojson_to_feature_collection(geojson);

        // project once at max zoom precision
        let z2 = 1u32 << max_zoom;
        let tolerance = (tile_options.tolerance / tile_options.extent as f64) / z2 as f64;
        let converted = convert(&collection, tolerance, false);

        let features = wrap(
            &converted,
            tile_options.buffer as f64 / tile_options.extent as f64,
            tile_options.line_metrics,
        );

        Self {
            features,
            max_zoom,
            tile_options: tile_options.clone(),
        }
    }

    pub fn generate_tile(&self, z: u8, x: u32, y: u32) -> Tile {
        if z > self.max_zoom {
            panic!(
                "Requested zoom {} higher than max_zoom {}",
                z, self.max_zoom
            );
        }

        let z2 = 1u32 << z;
        let tolerance = (self.tile_options.tolerance / self.tile_options.extent as f64) / z2 as f64;
        let p = self.tile_options.buffer as f64 / self.tile_options.extent as f64;

        let left = clip::<0>(
            &self.features,
            (x as f64 - p) / z2 as f64,
            (x as f64 + 1. + p) / z2 as f64,
            -1.,
            2.,
            self.tile_options.line_metrics,
        );

        let clipped_features = clip::<1>(
            &left,
            (y as f64 - p) / z2 as f64,
            (y as f64 + 1. + p) / z2 as f64,
            -1.,
            2.,
            self.tile_options.line_metrics,
        );

        InternalTile::new(
            &clipped_features,
            z,
            x,
            y,
            self.tile_options.extent,
            tolerance,
            self.tile_options.line_metrics,
        )
        .tile
    }
}
