use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::{
    Component,
    Path,
    PathBuf,
};
use std::sync::Arc;

use fig_os_shim::Context;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::warn;
use wry::http::header::CONTENT_TYPE;
use wry::http::{
    Request,
    Response,
    StatusCode,
};

use super::util::{
    res_400,
    res_404,
};
use crate::webview::WindowId;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecIndex {
    completions: Vec<String>,
    #[serde(default)]
    diff_versioned_completions: Vec<String>,
}

/// User specs first so they can override bundled ones.
fn spec_dirs(ctx: &Context) -> Vec<PathBuf> {
    let mut dirs = vec![];
    if let Ok(dir) = fig_util::directories::autocomplete_specs_dir() {
        dirs.push(dir);
    }
    if let Ok(dir) = fig_util::directories::resources_path_ctx(ctx) {
        dirs.push(dir.join("specs"));
    }
    dirs
}

/// Index from `index.json`, or from top level `*.js` files for user dirs without one.
fn read_index(dir: &Path) -> Option<SpecIndex> {
    if let Ok(bytes) = std::fs::read(dir.join("index.json")) {
        match serde_json::from_slice(&bytes) {
            Ok(index) => return Some(index),
            Err(err) => warn!(%err, "invalid spec index in {}", dir.display()),
        }
    }

    let completions = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.strip_suffix(".js").map(str::to_owned))
        .collect();

    Some(SpecIndex {
        completions,
        diff_versioned_completions: vec![],
    })
}

fn merged_index(dirs: &[PathBuf]) -> SpecIndex {
    let mut completions = BTreeSet::new();
    let mut diff_versioned_completions = BTreeSet::new();
    for index in dirs.iter().filter_map(|dir| read_index(dir)) {
        completions.extend(index.completions);
        diff_versioned_completions.extend(index.diff_versioned_completions);
    }
    SpecIndex {
        completions: completions.into_iter().collect(),
        diff_versioned_completions: diff_versioned_completions.into_iter().collect(),
    }
}

fn res_ok(body: Vec<u8>, mime: &str) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, mime)
        .body(body.into())?)
}

// handle `spec://localhost/spec.js`
pub async fn handle(
    ctx: Arc<Context>,
    request: Request<Vec<u8>>,
    _: WindowId,
) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let dirs = spec_dirs(&ctx);
    let path = request.uri().path();

    if path == "/index.json" {
        return res_ok(serde_json::to_vec(&merged_index(&dirs))?, mime::APPLICATION_JSON.as_ref());
    }

    let relative = Path::new(path.strip_prefix('/').unwrap_or(path));
    if !relative.components().all(|c| matches!(c, Component::Normal(_))) {
        return res_400();
    }

    let mime = match relative.extension().and_then(|ext| ext.to_str()) {
        Some("js") => mime::APPLICATION_JAVASCRIPT_UTF_8.as_ref(),
        Some("json") => mime::APPLICATION_JSON.as_ref(),
        Some("png") => mime::IMAGE_PNG.as_ref(),
        Some("svg") => mime::IMAGE_SVG.as_ref(),
        _ => mime::APPLICATION_OCTET_STREAM.as_ref(),
    };

    for dir in &dirs {
        if let Ok(bytes) = std::fs::read(dir.join(relative)) {
            return res_ok(bytes, mime);
        }
    }

    res_404()
}
