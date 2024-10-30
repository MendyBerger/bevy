use crate::io::{
    AssetReader, AssetReaderError, EmptyPathStream, PathStream, Reader,
};
use bevy_log::error;
use bevy_utils::BoxedFuture;
use std::path::{Path, PathBuf};

/// Reader implementation for loading assets via HTTP in WASM.
pub struct WasiAssetReader {
    // root_path: PathBuf,
}

impl WasiAssetReader {
    pub fn new() -> Self {
        Self {

        }
    }
}

// const WASI_HTTP_PREFIX: &str = "QQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQ";
const WASI_HTTP_PREFIX: [u8; 12] = [255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0];

impl AssetReader for WasiAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        log::info!("AssetReader::read()");
        Box::pin(async move {
            // let path = self.root_path.join(path);
            // self.fetch_bytes(path).await
            // todo!()
            let g = Box::new(&WASI_HTTP_PREFIX[..]) as Box<(dyn futures_io::AsyncRead + std::marker::Send + Sync + Unpin)>;
            Ok(g)
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        log::info!("AssetReader::read_meta()");
        Box::pin(async move {
            // let meta_path = get_meta_path(&self.root_path.join(path));
            // Ok(self.fetch_bytes(meta_path).await?)
            // todo!()
            let g = Box::new(&WASI_HTTP_PREFIX[..]) as Box<(dyn futures_io::AsyncRead + std::marker::Send + Sync + Unpin)>;
            Ok(g)
        })
    }

    fn read_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        let stream: Box<PathStream> = Box::new(EmptyPathStream);
        error!("Reading directories is not supported with the WasiAssetReader");
        Box::pin(async move { Ok(stream) })
    }

    fn is_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> BoxedFuture<'a, std::result::Result<bool, AssetReaderError>> {
        error!("Reading directories is not supported with the WasiAssetReader");
        Box::pin(async move { Ok(false) })
    }
}
