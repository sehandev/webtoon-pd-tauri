use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use tauri::{AppHandle, Manager};

use crate::{
    domain::{RenderManifest, RenderedTile, ToolConfig, MAX_TILE_HEIGHT, MAX_TILE_WIDTH},
    error::{AppError, AppResult},
};

#[derive(Debug, Clone)]
struct ToolCommand {
    program: PathBuf,
    prefix_args: Vec<OsString>,
    display_name: String,
}

pub fn render_source(
    app: &AppHandle,
    tools: &ToolConfig,
    source_path: &Path,
    output_dir: &Path,
) -> AppResult<RenderManifest> {
    std::fs::create_dir_all(output_dir)?;
    let manifest_path = output_dir.join("manifest.json");
    let renderer = resolve_renderer_tool(app, tools.renderer_bin.as_deref())?;

    let mut args = renderer.prefix_args.clone();
    args.extend([
        OsString::from("--input"),
        source_path.as_os_str().to_os_string(),
        OsString::from("--output-dir"),
        output_dir.as_os_str().to_os_string(),
        OsString::from("--max-width"),
        OsString::from(MAX_TILE_WIDTH.to_string()),
        OsString::from("--tile-height"),
        OsString::from(MAX_TILE_HEIGHT.to_string()),
        OsString::from("--manifest"),
        manifest_path.as_os_str().to_os_string(),
    ]);

    run_tool(&renderer, args)?;
    let manifest_json = std::fs::read_to_string(manifest_path)?;
    let manifest: RenderManifest = serde_json::from_str(&manifest_json)?;
    ensure_render_manifest_paths(&manifest)?;
    Ok(manifest)
}

pub fn encode_avif_tiles(
    app: &AppHandle,
    tools: &ToolConfig,
    rendered_tiles: &[RenderedTile],
    output_dir: &Path,
) -> AppResult<Vec<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;
    let avifenc = resolve_plain_tool(
        app,
        tools.avifenc_bin.as_deref(),
        "WEBTOON_PD_AVIFENC_BIN",
        "avifenc",
    )?;

    let mut outputs = Vec::with_capacity(rendered_tiles.len());
    for tile in rendered_tiles {
        let output_path = output_dir.join(format!("tile_{:04}.avif", tile.index));
        let args = vec![
            OsString::from("-l"),
            OsString::from("-s"),
            OsString::from("0"),
            OsString::from("-j"),
            OsString::from("all"),
            OsString::from("-y"),
            OsString::from("444"),
            OsString::from("-r"),
            OsString::from("full"),
            OsString::from("--"),
            tile.path.as_os_str().to_os_string(),
            output_path.as_os_str().to_os_string(),
        ];
        run_tool(&avifenc, args)?;
        outputs.push(output_path);
    }

    Ok(outputs)
}

fn resolve_renderer_tool(app: &AppHandle, override_path: Option<&str>) -> AppResult<ToolCommand> {
    if let Some(path) = normalize_override_path(override_path) {
        return renderer_invocation(path);
    }

    if let Ok(path) = std::env::var("WEBTOON_PD_IMAGE_RENDERER_BIN") {
        return renderer_invocation(PathBuf::from(path));
    }

    for candidate in bundled_tool_candidates(app, "image-renderer") {
        if candidate.exists() {
            return renderer_invocation(candidate);
        }
    }

    renderer_invocation(PathBuf::from("image-renderer"))
}

fn resolve_plain_tool(
    app: &AppHandle,
    override_path: Option<&str>,
    env_name: &str,
    base_name: &str,
) -> AppResult<ToolCommand> {
    if let Some(path) = normalize_override_path(override_path) {
        return Ok(ToolCommand {
            program: path,
            prefix_args: Vec::new(),
            display_name: base_name.to_string(),
        });
    }

    if let Ok(path) = std::env::var(env_name) {
        return Ok(ToolCommand {
            program: PathBuf::from(path),
            prefix_args: Vec::new(),
            display_name: base_name.to_string(),
        });
    }

    for candidate in bundled_tool_candidates(app, base_name) {
        if candidate.exists() {
            return Ok(ToolCommand {
                program: candidate,
                prefix_args: Vec::new(),
                display_name: base_name.to_string(),
            });
        }
    }

    Ok(ToolCommand {
        program: PathBuf::from(base_name),
        prefix_args: Vec::new(),
        display_name: base_name.to_string(),
    })
}

fn renderer_invocation(path: PathBuf) -> AppResult<ToolCommand> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("py"))
    {
        return Ok(ToolCommand {
            program: PathBuf::from("python3"),
            prefix_args: vec![path.into_os_string()],
            display_name: "image-renderer.py".to_string(),
        });
    }

    Ok(ToolCommand {
        program: path,
        prefix_args: Vec::new(),
        display_name: "image-renderer".to_string(),
    })
}

fn normalize_override_path(path: Option<&str>) -> Option<PathBuf> {
    path.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    })
}

fn bundled_tool_candidates(app: &AppHandle, base_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        let binary_dir = resource_dir.join("binaries");
        candidates.push(binary_dir.join(platform_binary_name(base_name)));
        candidates.push(binary_dir.join(base_name));
    }
    candidates
}

fn platform_binary_name(base_name: &str) -> String {
    let extension = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };
    format!("{base_name}-{}{extension}", platform_target_triple())
}

fn platform_target_triple() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        _ => "unknown",
    }
}

fn run_tool(tool: &ToolCommand, args: Vec<OsString>) -> AppResult<()> {
    let output = Command::new(&tool.program).args(args).output();
    let output = match output {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::ToolNotFound(tool.display_name.clone()));
        }
        Err(error) => return Err(AppError::Io(error)),
    };

    if output.status.success() {
        return Ok(());
    }

    Err(AppError::ToolFailed {
        tool: tool.display_name.clone(),
        status: output.status.to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn ensure_render_manifest_paths(manifest: &RenderManifest) -> AppResult<()> {
    for tile in &manifest.tiles {
        if !tile.path.exists() {
            return Err(AppError::ToolFailed {
                tool: "image-renderer".to_string(),
                status: "missing output".to_string(),
                stderr: format!("rendered tile does not exist: {}", tile.path.display()),
            });
        }
    }
    Ok(())
}
