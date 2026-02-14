use std::path::PathBuf;
use std::io::BufRead as _;
use clap::Parser as _;
use rayon::prelude::*;

mod trim_offset;
mod article;
mod markdown;
mod tree_sitter_html;

macro_rules! die {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about = "Simple Static Site Generator")]
struct Config {
    /// The name of blog, displayed in page titles and headers
    #[arg(short, long, default_value = "My Blog")]
    sitename: String,

    /// Path to the directory containing source content,
    /// expected a flat structure with "ascii_alphanumeric_lowercase.md" files
    #[arg(short, long, default_value = "content")]
    content: PathBuf,

    /// Directory where the generated HTML files will be saved
    #[arg(short, long, default_value = "public")]
    output: PathBuf,

    /// Path to the directory with static assets,
    /// expected "head.html", "header.html", "footer.html" and intro.md
    #[arg(short, long, default_value = "assets")]
    assets: PathBuf,

    /// Optional list of specific files to process,
    /// if empty, all files in the content directory will be processed
    #[arg(short, long, num_args = 1..)]
    files: Option<Vec<PathBuf>>,
}

#[minificator::template]
#[derive(askama::Template)]
#[template(path = "templates/article.html")]
pub struct ArticleTemplate<'a> {
    pub sitename: &'a str,
    pub title:    &'a str,
    pub head:     &'a str,
    pub header:   &'a str,
    pub footer:   &'a str,
    pub ts:       article::Datetime,
    pub content:  markdown::Markdown<'a>,
}

#[minificator::template]
#[derive(askama::Template)]
#[template(path = "templates/index.html")]
pub struct IndexTemplate<'a> {
    pub sitename: &'a str,
    pub head:     &'a str,
    pub header:   &'a str,
    pub footer:   &'a str,
    pub articles: &'a[&'a (crate::article::Article, String)],
    pub intro:    markdown::Markdown<'a>,
}

fn main() {
    let time = std::time::Instant::now();
    let cfg = Config::parse();

    println!("{} v{}\nUse '--help' for more information.", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    if cfg.content == cfg.output {
        die!("Error: input dir eq output dir");
    }

    let head   = load_asset(&cfg.assets, "head.html", true);
    let header = load_asset(&cfg.assets, "header.html", true);
    let footer = load_asset(&cfg.assets, "footer.html", true);
    let intro  = load_asset(&cfg.assets, "intro.md", false);

    std::fs::create_dir_all(&cfg.output).unwrap_or_else(|e| die!("Error: can't open output directory {:?} - {e}", cfg.output));

    let articles = load_dir(&cfg.content)
        .unwrap_or_else(|e| die!("Error: can't process content {:?} - {e}", cfg.content))
        .into_par_iter()
        .map(|(name, src)| {
            let data = load_article(&src, cfg.files.as_deref()).unwrap_or_else(|e| die!("Error: can't read {src:?} - {e}"));
            let article = article::Article::new(data).unwrap_or_else(|e| die!("Error: can't parse {src:?} - {e}"));

            let path = cfg.output.join(&name);
            match article.body() {
                Some(body) => write(&path, ArticleTemplate {
                    sitename: &cfg.sitename,
                    title:    article.name(),
                    head:     &head,
                    header:   &header,
                    footer:   &footer,
                    ts:       article.ts,
                    content:  markdown::Markdown(body),
                }),
                None => println!("Info: ignored {path:?}"),
            }

            (article, name)
        })
        .collect::<Vec<_>>();

    if articles.len() == 0 {
        die!("Error: no articles found");
    }

    let mut sorted = articles.iter().collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|v| std::cmp::Reverse(v.0.ts));

    write(&cfg.output.join("index.html"), IndexTemplate {
        sitename: &cfg.sitename,
        head:     &head, 
        header:   &header,
        footer:   &footer,
        articles: sorted.as_slice(),
        intro:    markdown::Markdown(&intro),
    });

    println!(
        "Done in {:.2} seconds: {} indexed, {} parsed, index generated",
        time.elapsed().as_secs_f64(),
        articles.len(),
        articles.iter().filter(|v| v.0.body().is_some()).count(),
    );
}

fn load_asset(dir: &PathBuf, path: &str, html: bool) -> String {
    fn load_html(src: PathBuf) -> Result<String, std::io::Error> {
        let file = std::fs::File::open(src)?;
        let mut s = String::with_capacity(file.metadata().map(|v| v.len() as usize).unwrap_or_default());
        for line in std::io::BufReader::new(file).lines() {
            s.push_str(line?.trim());
        }
        Ok(s)
    }

    let src = dir.join(path);
    match if html { load_html(src) } else { std::fs::read_to_string(src) } {
        Ok(s) => { println!("Info: \"{path}\" loaded"); s },
        Err(e) => { println!("Warning: \"{path}\" - {e}. Empty value is used"); String::new() },
    }
}

fn load_dir(path: &PathBuf) -> Result<Vec<(String, PathBuf)>, std::io::Error> {
    let mut result = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let src = entry?.path();
        if 
            src.is_file() &&
            src.extension().map_or(false, |ext| ext == "md") &&
            let Some(dst) = src.file_stem() &&
            let Some(dst) = dst.to_str() &&
            dst.len() > 0 &&
            dst.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_')) &&
            dst != "index"
        {
            result.push((format!("{dst}.html"), src));
        } else {
            println!("Warning: {src:?} - isn't \"ascii_alphanumeric_lowercase.md\", ignored");
        }
    }

    Ok(result)
}

fn load_article(path: &PathBuf, filter: Option<&[PathBuf]>) -> Result<String, std::io::Error> {
    let file = std::fs::File::open(&path)?;
    let mut s = match filter.map_or(false, |v| v.iter().any(|f| f == path)) {
        true => {
            let mut line = String::with_capacity(512);
            std::io::BufReader::new(file).read_line(&mut line)?;
            Ok(line)
        },
        false => std::fs::read_to_string(&path),
    }?;
    s.retain(|c| c != '\r');
    Ok(s)
}

fn write<T: askama::Template>(dst: &PathBuf, t: T) {
    match inner(dst, t) {
        Ok(_) => println!("Info: successfully generated {dst:?}"),
        Err(e) => die!("Error: can't write to {dst:?} - {e}"),
    }

    fn inner<T: askama::Template>(dst: &PathBuf, t: T) -> Result<(), std::io::Error> {
        askama::Template::write_into(&t, &mut std::io::BufWriter::new(std::fs::File::create(&dst)?))
    }
}