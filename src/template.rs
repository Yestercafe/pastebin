use minijinja::Environment;
use std::path::Path;

pub fn render_from_dir(
    templates_dir: &Path,
    name: &str,
    ctx: impl serde::Serialize,
) -> Result<String, minijinja::Error> {
    let path = templates_dir.join(name);
    let source = std::fs::read_to_string(&path).map_err(|_| minijinja::Error::new(
        minijinja::ErrorKind::InvalidOperation,
        "template not found",
    ))?;
    let mut env = Environment::new();
    env.add_template(name, source.as_str())?;
    let tmpl = env.get_template(name)?;
    tmpl.render(ctx)
}
