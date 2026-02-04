use tree_sitter_highlight::{ HighlightConfiguration, Highlighter, HighlightEvent };

use tree_sitter_asm        as asm;
use tree_sitter_bash       as bash;
use tree_sitter_c          as c;
use tree_sitter_cpp        as cpp;
use tree_sitter_css        as css;
use tree_sitter_c_sharp    as c_sharp;
use tree_sitter_elixir     as elixir;
use tree_sitter_fsharp     as fsharp;
use tree_sitter_java       as java;
use tree_sitter_javascript as js;
use tree_sitter_julia      as julia;
use tree_sitter_html       as html;
use tree_sitter_kotlin_sg  as kotlin;
use tree_sitter_lua        as lua;
use tree_sitter_go         as go;
use tree_sitter_ocaml      as ocaml;
use tree_sitter_pascal     as pascal;
use tree_sitter_php        as php;
use tree_sitter_powershell as pwsh;
use tree_sitter_python     as python;
use tree_sitter_ruby       as ruby;
use tree_sitter_rust       as rust;
use tree_sitter_scala      as scala;
use tree_sitter_sequel     as sql;
use tree_sitter_swift      as swift;
use tree_sitter_typescript as ts;
use tree_sitter_xml        as xml;
use tree_sitter_zig        as zig;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Lang {
    Asm    = 0,
    Bash   = 1,
    C      = 2,
    Cpp    = 3,
    Css    = 4,
    Csharp = 5,
    Elixir = 6,
    Fsharp = 7,
    Java   = 8,
    Js     = 9,
    Julia  = 10,
    Html   = 11,
    Kotlin = 12,
    Lua    = 13,
    Go     = 14,
    Ocaml  = 15,
    Pascal = 16,
    Php    = 17,
    Pwsh   = 18,
    Python = 19,
    Ruby   = 20,
    Rust   = 21,
    Scala  = 22,
    Sql    = 23,
    Swift  = 24,
    Ts     = 25,
    Xml    = 26,
    Zig    = 27,
}

struct Dataset<'a> {
    name:       &'a str,
    f:          fn() -> tree_sitter::Language,
    highlights: &'a str,
    injection:  &'a str,
    locals:     &'a str,
}

impl Lang {
    pub fn form_str(s: &str) -> Option<Self> {
        let s = s.trim();

        if s.len() == 0 || s.len() > 16 || !s.is_ascii() {
            return None;
        }

        let mut v = [0u8; 16];

        s.as_bytes().iter().enumerate().for_each(|(i, b)| v[i] = *b);

        match &v[..s.len()] {
            b"asm" | b"assembly" | b"nasm" | b"fasm" => Some(Lang::Asm),
            b"bash" | b"sh" | b"shell"               => Some(Lang::Bash),
            b"c"                                     => Some(Lang::C),
            b"cpp" | b"c++" | b"cc" | b"cxx"         => Some(Lang::Cpp),
            b"css"                                   => Some(Lang::Css),
            b"csharp" | b"cs" | b"c#"                => Some(Lang::Csharp),
            b"elixir" | b"ex" | b"exs"               => Some(Lang::Elixir),
            b"fsharp" | b"fs" | b"f#"                => Some(Lang::Fsharp),
            b"java"                                  => Some(Lang::Java),
            b"js" | b"javascript" | b"node"          => Some(Lang::Js),
            b"julia" | b"jl"                         => Some(Lang::Julia),
            b"html" | b"htm"                         => Some(Lang::Html),
            b"kotlin" | b"kt" | b"kts"               => Some(Lang::Kotlin),
            b"lua"                                   => Some(Lang::Lua),
            b"go" | b"golang"                        => Some(Lang::Go),
            b"ocaml" | b"ml"                         => Some(Lang::Ocaml),
            b"pascal" | b"delphi" | b"pas"           => Some(Lang::Pascal),
            b"php"                                   => Some(Lang::Php),
            b"pwsh" | b"powershell" | b"ps1"         => Some(Lang::Pwsh),
            b"python" | b"py"                        => Some(Lang::Python),
            b"ruby" | b"rb"                          => Some(Lang::Ruby),
            b"rust" | b"rs"                          => Some(Lang::Rust),
            b"scala" | b"sc"                         => Some(Lang::Scala),
            b"sql"                                   => Some(Lang::Sql),
            b"swift"                                 => Some(Lang::Swift),
            b"ts" | b"typescript"                    => Some(Lang::Ts),
            b"xml" | b"rss" | b"svg"                 => Some(Lang::Xml),
            b"zig"                                   => Some(Lang::Zig),
            b"text" | _                              => None,
        }
    }

    fn dataset(self) -> Dataset<'static> {
        match self {
            Lang::Asm    => Dataset { name:"asm",    f: || asm::LANGUAGE.into(),           highlights: asm::HIGHLIGHTS_QUERY,                     injection: "",                       locals: "" },
            Lang::Bash   => Dataset { name:"bash",   f: || bash::LANGUAGE.into(),          highlights: bash::HIGHLIGHT_QUERY,                     injection: "",                       locals: "" },
            Lang::C      => Dataset { name:"c",      f: || c::LANGUAGE.into(),             highlights: c::HIGHLIGHT_QUERY,                        injection: "",                       locals: "" },
            Lang::Cpp    => Dataset { name:"cpp",    f: || cpp::LANGUAGE.into(),           highlights: CPP_HIGHLIGHTNING,                         injection: "",                       locals: "" },
            Lang::Css    => Dataset { name:"css",    f: || css::LANGUAGE.into(),           highlights: css::HIGHLIGHTS_QUERY,                     injection: "",                       locals: "" },
            Lang::Csharp => Dataset { name:"csharp", f: || c_sharp::LANGUAGE.into(),       highlights: include_str!("../highlights/c_sharp.scm"), injection: "",                       locals: "" },
            Lang::Elixir => Dataset { name:"elixir", f: || elixir::LANGUAGE.into(),        highlights: elixir::HIGHLIGHTS_QUERY,                  injection: elixir::INJECTIONS_QUERY, locals: "" },
            Lang::Fsharp => Dataset { name:"fsharp", f: || fsharp::LANGUAGE_FSHARP.into(), highlights: fsharp::HIGHLIGHTS_QUERY,                  injection: fsharp::INJECTIONS_QUERY, locals: fsharp::LOCALS_QUERY },
            Lang::Java   => Dataset { name:"java",   f: || java::LANGUAGE.into(),          highlights: java::HIGHLIGHTS_QUERY,                    injection: "",                       locals: "" },
            Lang::Js     => Dataset { name:"js",     f: || js::LANGUAGE.into(),            highlights: js::HIGHLIGHT_QUERY,                       injection: js::INJECTIONS_QUERY,     locals: js::LOCALS_QUERY },
            Lang::Julia  => Dataset { name:"julia",  f: || julia::LANGUAGE.into(),         highlights: include_str!("../highlights/julia.scm"),   injection: "",                       locals: "" },
            Lang::Html   => Dataset { name:"html",   f: || html::LANGUAGE.into(),          highlights: html::HIGHLIGHTS_QUERY,                    injection: html::INJECTIONS_QUERY,   locals: "" },
            Lang::Kotlin => Dataset { name:"kotlin", f: || kotlin::LANGUAGE.into(),        highlights: kotlin::HIGHLIGHTS_QUERY,                  injection: "",                       locals: "" },
            Lang::Lua    => Dataset { name:"lua",    f: || lua::LANGUAGE.into(),           highlights: lua::HIGHLIGHTS_QUERY,                     injection: lua::INJECTIONS_QUERY,    locals: lua::LOCALS_QUERY },
            Lang::Go     => Dataset { name:"go",     f: || go::LANGUAGE.into(),            highlights: go::HIGHLIGHTS_QUERY,                      injection: "",                       locals: "" },
            Lang::Ocaml  => Dataset { name:"ocaml",  f: || ocaml::LANGUAGE_OCAML.into(),   highlights: ocaml::HIGHLIGHTS_QUERY,                   injection: "",                       locals: ocaml::LOCALS_QUERY },
            Lang::Pascal => Dataset { name:"pascal", f: || pascal::LANGUAGE.into(),        highlights: include_str!("../highlights/pascal.scm"),  injection: "",                       locals: "" },
            Lang::Php    => Dataset { name:"php",    f: || php::LANGUAGE_PHP_ONLY.into(),  highlights: php::HIGHLIGHTS_QUERY,                     injection: php::INJECTIONS_QUERY,    locals: "" },
            Lang::Pwsh   => Dataset { name:"pwsh",   f: || pwsh::LANGUAGE.into(),          highlights: include_str!("../highlights/pwsh.scm"),    injection: "",                       locals: "" },
            Lang::Python => Dataset { name:"python", f: || python::LANGUAGE.into(),        highlights: python::HIGHLIGHTS_QUERY,                  injection: "",                       locals: "" },
            Lang::Ruby   => Dataset { name:"ruby",   f: || ruby::LANGUAGE.into(),          highlights: ruby::HIGHLIGHTS_QUERY,                    injection: "",                       locals: "" },
            Lang::Rust   => Dataset { name:"rust",   f: || rust::LANGUAGE.into(),          highlights: rust::HIGHLIGHTS_QUERY,                    injection: rust::INJECTIONS_QUERY,   locals: "" },
            Lang::Scala  => Dataset { name:"scala",  f: || scala::LANGUAGE.into(),         highlights: scala::HIGHLIGHTS_QUERY,                   injection: "",                       locals: scala::LOCALS_QUERY },
            Lang::Sql    => Dataset { name:"sql",    f: || sql::LANGUAGE.into(),           highlights: sql::HIGHLIGHTS_QUERY,                     injection: "",                       locals: "" },
            Lang::Swift  => Dataset { name:"swift",  f: || swift::LANGUAGE.into(),         highlights: swift::HIGHLIGHTS_QUERY,                   injection: swift::INJECTIONS_QUERY,  locals: swift::LOCALS_QUERY },
            Lang::Ts     => Dataset { name:"ts",     f: || ts::LANGUAGE_TYPESCRIPT.into(), highlights: TS_HIGHLIGHTNING,                          injection: "",                       locals: ts::LOCALS_QUERY },
            Lang::Xml    => Dataset { name:"xml",    f: || xml::LANGUAGE_XML.into(),       highlights: xml::XML_HIGHLIGHT_QUERY,                  injection: "",                       locals: "" },
            Lang::Zig    => Dataset { name:"zig",    f: || zig::LANGUAGE.into(),           highlights: zig::HIGHLIGHTS_QUERY,                     injection: zig::INJECTIONS_QUERY,    locals: "" },
        }
    }
}

pub struct LangDb([std::sync::OnceLock<HighlightConfiguration>; 28]);

const CPP_HIGHLIGHTNING: &str = const_format::concatcp!(c::HIGHLIGHT_QUERY, "\n", cpp::HIGHLIGHT_QUERY);
const TS_HIGHLIGHTNING: &str = const_format::concatcp!(js::HIGHLIGHT_QUERY, "\n", ts::HIGHLIGHTS_QUERY);

const ENTITIES: &[&str] = &[
    "comment",
    "comment.documentation",
    "tag",
    "keyword",
    "keyword.directive",
    "keyword.function",
    "keyword.return",
    "keyword.conditional",
    "function",
    "function.method",
    "function.call",
    "function.builtin",
    "string",
    "string.special",
    "type",
    "type.builtin",
    "number",
    "number.float",
    "property",
    "variable",
    "variable.builtin",
    "label",
    "constant",
    "constant.builtin",
    "operator",
    "attribute",
    "module",
    // "punctuation.bracket",
    // "punctuation.delimiter",
];

const CSS: &[&str] = &[
    "comm",
    "comm",
    "kw",
    "kw",
    "kw",
    "kw",
    "kw",
    "kw",
    "fn",
    "fn",
    "fn",
    "fn",
    "str",
    "str",
    "ty",
    "ty",
    "num",
    "num",
    "prop",
    "var",
    "var",
    "var",
    "const",
    "const",
    "op",
    "attr",
    "mod",
    // "punctuation",
    // "punctuation",
];

const _: () = assert!(ENTITIES.len() == CSS.len());

impl LangDb {
    pub const fn new() -> Self {
        unsafe {
            let mut a: [std::mem::MaybeUninit<std::sync::OnceLock<HighlightConfiguration>>; 28] = std::mem::MaybeUninit::uninit().assume_init();
            
            let mut i = 0;
            while i < 28 {
                a[i] = std::mem::MaybeUninit::new(std::sync::OnceLock::new());
                i += 1;
            }
            
            Self(std::mem::transmute(a))
        }
    }

    pub fn html<W: pulldown_cmark_escape::StrWrite>(&self, s: &str, lang: Lang, mut w: W) -> Result<(), W::Error> {
        let cfg = self.0[lang as usize].get_or_init(|| {
            let v = lang.dataset();
            let mut cfg = HighlightConfiguration::new((v.f)(), v.name, v.highlights, v.injection, v.locals)
                .unwrap_or_else(|e| panic!("Can't load {lang:?}: {e:?}"));
            cfg.configure(ENTITIES);
            cfg
        });

        let mut highlighter = Highlighter::new();

        for v in highlighter.highlight(&cfg, s.as_bytes(), None, |_| None).unwrap() {
            match v.unwrap() {
                HighlightEvent::Source { start, end } => {
                    pulldown_cmark_escape::escape_html(&mut w, std::str::from_utf8(&s.as_bytes()[start..end]).unwrap())?;
                }
                HighlightEvent::HighlightStart(h) => {
                    if let Some(name) = CSS.get(h.0) {
                        write!(w, "<span class=\"{name}\">")?;
                    }
                }
                HighlightEvent::HighlightEnd => {
                    write!(w, "</span>")?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SET: &[(Lang, &str)] = &[
        (Lang::Asm,    "mov rax, 1\nadd rax, 2"),
        (Lang::Bash,   "echo \"Hi!\"\nsum=$((1 + 2))"),
        (Lang::C,      "printf(\"Hi!\");\nint sum = 1 + 2;"),
        (Lang::Cpp,    "std::cout << \"Hi!\" << std::endl;\nint sum = 1 + 2;"),
        (Lang::Css,    ".class { color: red; }\n.sum { width: 3px; }"),
        (Lang::Csharp, "Console.WriteLine(\"Hi!\");\nint sum = 1 + 2;"),
        (Lang::Elixir, "IO.puts \"Hi!\"sum = 1 + 2"),
        (Lang::Fsharp, "printfn \"Hi!\"let sum = 1 + 2"),
        (Lang::Java,   "System.out.println(\"Hi!\");\nint sum = 1 + 2;"),
        (Lang::Js,     "console.log(\"Hi!\");\nconst sum = 1 + 2;"),
        (Lang::Julia,  "println(\"Hi!\")\nsum = 1 + 2"),
        (Lang::Html,   "<p>Hi!</p>\n<span class='num'>3</span>"),
        (Lang::Kotlin, "println(\"Hi!\")\nval sum = 1 + 2"),
        (Lang::Lua,    "print(\"Hi!\")\nlocal sum = 1 + 2"),
        (Lang::Go,     "fmt.Println(\"Hi!\")\nsum := 1 + 2"),
        (Lang::Ocaml,  "print_endline \"Hi!\";;\nlet sum = 1 + 2;;"),
        (Lang::Pascal, "WriteLn('Hi!');\nsum := 1 + 2;"),
        (Lang::Php,    "echo \"Hi!\";\n$sum = 1 + 2;"),
        (Lang::Pwsh,   "Write-Host \"Hi!\"$sum = 1 + 2"),
        (Lang::Python, "print(\"Hi!\")\nsum = 1 + 2"),
        (Lang::Ruby,   "puts \"Hi!\"\nsum = 1 + 2"),
        (Lang::Rust,   "println!(\"Hi!\");\nlet sum = 1 + 2;"),
        (Lang::Scala,  "println(\"Hi!\")\nval sum = 1 + 2"),
        (Lang::Sql,    "SELECT \"Hi!\";\nSELECT 1 + 2 AS sum;"),
        (Lang::Swift,  "print(\"Hi!\")\nlet sum = 1 + 2"),
        (Lang::Ts,     "console.log('Hi!');\nconst sum: number = 1 + 2;"),
        (Lang::Xml,    "<message>Hi!</message>\n<sum>3</sum>"),
        (Lang::Zig,    "std.debug.print(\"Hi!\", .{});\nconst sum = 1 + 2;"),
    ];

    #[test]
    fn test() {
        let db = LangDb::new();

        let report = SET.iter()
            .map(|(lang, input)| {
                let mut s = String::with_capacity(input.len() * 8);
                match db.html(input, *lang, &mut s) {
                    Ok(_) if s.contains("<span") => Ok((lang, input, s)),
                    Ok(_) => Err((lang, input, format!("Highlight error:\n{s}"))),
                    Err(e) => Err((lang, input, format!("Critical error:\n{e:?}"))),
                }
            })
            .filter_map(|v| v.err())
            // .filter_map(|v| v.ok())
            .map(|v| format!("================\n\nLang:\n{:?}\n\nInput:\n{}\n\nResult:\n{}\n\n", v.0, v.1, v.2))
            .collect::<String>();

        if !report.is_empty() {
            panic!("{report}================\n");
        }
    }
}