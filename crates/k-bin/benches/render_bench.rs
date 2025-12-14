use criterion::{Criterion, criterion_group, criterion_main};
use tera::{Context, Tera};

fn bench_render_simple(c: &mut Criterion) {
    let mut tera = Tera::default();
    tera.add_raw_template("simple", "Hello {{ name }}! Color is {{ colors.primary }}")
        .unwrap();

    let mut ctx = Context::new();
    ctx.insert("name", "User");
    ctx.insert("colors", &serde_json::json!({"primary": "#ff0000"}));

    c.bench_function("render simple template", |b| {
        b.iter(|| tera.render("simple", &ctx).unwrap())
    });
}

fn bench_render_complex(c: &mut Criterion) {
    let mut tera = Tera::default();
    // A more realistic template with conditionals and loops
    let complex_template = r#"
# Kitchn Theme
Background: {{ colors.bg }}
Foreground: {{ colors.fg }}
{% if colors.accent %}Accent: {{ colors.accent }}{% endif %}
## Terminal Colors
{% for color in ansi_colors %}
- {{ color.name }}: {{ color.value }}
{% endfor %}
"#;
    tera.add_raw_template("complex", complex_template).unwrap();

    let mut ctx = Context::new();
    ctx.insert(
        "colors",
        &serde_json::json!({
            "bg": "#282a36",
            "fg": "#f8f8f2",
            "accent": "#bd93f9"
        }),
    );
    ctx.insert(
        "ansi_colors",
        &serde_json::json!([
            {"name": "red", "value": "#ff5555"},
            {"name": "green", "value": "#50fa7b"},
            {"name": "yellow", "value": "#f1fa8c"},
            {"name": "blue", "value": "#6272a4"},
            {"name": "magenta", "value": "#ff79c6"},
            {"name": "cyan", "value": "#8be9fd"}
        ]),
    );

    c.bench_function("render complex template", |b| {
        b.iter(|| tera.render("complex", &ctx).unwrap())
    });
}

criterion_group!(benches, bench_render_simple, bench_render_complex);
criterion_main!(benches);
