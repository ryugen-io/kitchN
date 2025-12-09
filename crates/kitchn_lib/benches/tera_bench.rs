use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;
use tera::{Context, Error, Tera, Value};

fn benchmark_tera_render(c: &mut Criterion) {
    let mut tera = Tera::default();
    // Register the custom filter we use in production
    tera.register_filter(
        "hex_to_rgb",
        |value: &Value, _args: &HashMap<String, Value>| -> Result<Value, Error> {
            // approximate the filter logic for bench
            let _ = value.as_str().unwrap_or("#000000");
            Ok(tera::to_value(vec![0, 0, 0]).unwrap())
        },
    );

    let template_content = "
    window {
        background: {{ colors.bg }};
        foreground: {{ colors.fg }};
        border: 1px solid {{ colors.primary | hex_to_rgb }};
    }
    ";

    let mut ctx = Context::new();
    ctx.insert(
        "colors",
        &serde_json::json!({
            "bg": "#282a36",
            "fg": "#f8f8f2",
            "primary": "#bd93f9"
        }),
    );

    c.bench_function("tera_render_template", |b| {
        b.iter(|| {
            let _ = tera.render_str(black_box(template_content), black_box(&ctx));
        })
    });
}

criterion_group!(benches, benchmark_tera_render);
criterion_main!(benches);
