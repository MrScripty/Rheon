//! User-triggered export. Never overwrites an existing artifact; failed new
//! output is removed, and success requires a completed flush.
use eframe::egui;
use std::{
    fs::OpenOptions,
    io::{self, BufWriter, Write},
    path::Path,
};
pub struct ExportFile {
    path: String,
    feedback: String,
}
impl ExportFile {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.into(),
            feedback: String::new(),
        }
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        write: impl FnOnce(&mut BufWriter<std::fs::File>) -> io::Result<()>,
    ) {
        ui.label("Export path (new file)");
        ui.add(egui::TextEdit::singleline(&mut self.path).desired_width(f32::INFINITY));
        if ui.button(label).clicked() {
            self.feedback = match create_export(Path::new(&self.path), write) {
                Ok(()) => format!("Saved {}", self.path),
                Err(e) => format!("Export failed: {e}"),
            };
        }
        if !self.feedback.is_empty() {
            ui.label(&self.feedback);
        }
    }
}
fn create_export(
    path: &Path,
    write: impl FnOnce(&mut BufWriter<std::fs::File>) -> io::Result<()>,
) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut out = BufWriter::new(file);
    let result = write(&mut out).and_then(|()| out.flush());
    drop(out);
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exports_refuse_overwrite_and_remove_only_failed_new_output() {
        let dir = std::env::temp_dir().join(format!("rheon-vtk-ui-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("data.vti");
        create_export(&path, |out| out.write_all(b"accepted")).unwrap();
        assert!(create_export(&path, |out| out.write_all(b"replacement")).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"accepted");
        let failed = dir.join("failed.vti");
        assert!(
            create_export(&failed, |out| {
                out.write_all(b"partial")?;
                Err(io::Error::other("injected writer failure"))
            })
            .is_err()
        );
        assert!(!failed.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
