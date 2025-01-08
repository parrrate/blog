use std::process::{Command, Output};

use chrono::{FixedOffset, TimeZone, Utc};
use clap::Parser;
use handlebars::Handlebars;
use miette::IntoDiagnostic;
use serde::Serialize;

#[derive(Parser)]
struct Cli {
    title: String,
}

#[derive(Serialize)]
struct Data<'a> {
    title: String,
    date: String,
    author: &'a str,
}

fn main() -> miette::Result<()> {
    let Cli { title } = Cli::parse();
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("blog", "src/00000000-0000-blog-template.md.handlebars")
        .into_diagnostic()?;
    let now = FixedOffset::east_opt(3 * 3600)
        .ok_or_else(|| miette::miette!("invalid TZ"))?
        .from_utc_datetime(&Utc::now().naive_utc());
    let date = now.format("%Y%m%d-%H%M");
    let slug = slug::slugify(title.as_str());
    let path = format!("src/pages/posts/{date}-{slug}.md");
    let date = now.to_rfc3339();
    let Output { status, stdout, .. } = Command::new("git")
        .arg("config")
        .arg("user.name")
        .output()
        .into_diagnostic()?;
    if !status.success() {
        miette::bail!("`git config user.name` failed: {status}");
    }
    let author = String::from_utf8(stdout).into_diagnostic()?;
    let author = author.trim();
    let data = Data {
        title,
        date,
        author,
    };
    let contents = handlebars.render("blog", &data).into_diagnostic()?;
    std::fs::write(path, contents).into_diagnostic()?;
    Ok(())
}
