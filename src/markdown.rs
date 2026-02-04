use pulldown_cmark::{ Alignment, BlockQuoteKind, CodeBlockKind, Event, LinkType, Tag, TagEnd };
use pulldown_cmark_escape::{ escape_href, escape_html, escape_html_body_text, FmtWriter, StrWrite };
use crate::tree_sitter_html::*;

static LANG_DB: crate::tree_sitter_html::LangDb = crate::tree_sitter_html::LangDb::new();

pub struct Markdown<'a>(pub &'a str);

impl<'a> std::fmt::Display for Markdown<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parser = pulldown_cmark::Parser::new_ext(self.0, pulldown_cmark::Options::ENABLE_TABLES | pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        HtmlWriter::new(self.0, parser, FmtWriter(f)).run()
    }
}

// INFO Fork of pulldown_cmark::HtmlWriter with new code processing and minification

enum TableState {
    Head,
    Body,
}

struct HtmlWriter<'a, I, W> {
    text:   &'a str,
    iter:   I,
    writer: W,

    in_non_writing_block: bool,

    table_state:      TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    numbers:          std::collections::HashMap<pulldown_cmark::CowStr<'a>, usize>,

    code_lang:  Option<Lang>,
    code_start: Option<* const u8>,
    code_len:   usize,
}

impl<'a, I, W> HtmlWriter<'a, I, W>
where
    I: Iterator<Item = Event<'a>>,
    W: StrWrite,
{
    fn new(text: &'a str, iter: I, writer: W) -> Self {
        Self {
            text,
            iter,
            writer,
            in_non_writing_block: false,
            table_state:          TableState::Head,
            table_alignments:     Vec::new(),
            table_cell_index:     0,
            numbers:              std::collections::HashMap::new(),
            code_lang:            None,
            code_start:           None,
            code_len:             0,
        }
    }

    fn run(mut self) -> Result<(), W::Error> {
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(tag) => self.start_tag(tag)?,
                Event::End(tag)    => self.end_tag(tag)?,
                Event::Text(text) => if !self.in_non_writing_block {
                    match &self.code_lang {
                        Some(_) if self.text.as_ptr() <= text.as_ptr() && self.text.as_ptr() as usize + self.text.len() >= text.as_ptr() as usize + text.len() => {
                            if self.code_start.is_none() {
                                self.code_start = Some(text.as_ptr());
                            }
                            self.code_len += text.len();
                        }
                        Some(_) => unreachable!(),
                        None => escape_html_body_text(&mut self.writer, &text)?,
                    }
                },
                Event::Code(text) => {
                    self.writer.write_str("<code>")?;
                    escape_html_body_text(&mut self.writer, &text)?;
                    self.writer.write_str("</code>")?;
                },
                Event::InlineMath(text) => {
                    self.writer.write_str(r#"<span class="math math-inline">"#)?;
                    escape_html(&mut self.writer, &text)?;
                    self.writer.write_str("</span>")?;
                },
                Event::DisplayMath(text) => {
                    self.writer.write_str(r#"<span class="math math-display">"#)?;
                    escape_html(&mut self.writer, &text)?;
                    self.writer.write_str("</span>")?;
                },
                Event::Html(html) | Event::InlineHtml(html) => self.writer.write_str(&html)?,
                Event::SoftBreak => self.writer.write_str("\n")?,
                Event::HardBreak => self.writer.write_str("<br />")?,
                Event::Rule      => self.writer.write_str("<hr />")?,
                Event::FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.writer.write_str("<sup class=\"footnote-reference\"><a href=\"#")?;
                    escape_html(&mut self.writer, &name)?;
                    self.writer.write_str("\">")?;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "{}", number)?;
                    self.writer.write_str("</a></sup>")?;
                },
                Event::TaskListMarker(true)  => self.writer.write_str("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>")?,
                Event::TaskListMarker(false) => self.writer.write_str("<input disabled=\"\" type=\"checkbox\"/>")?,
            }
        }
        Ok(())
    }

    fn start_tag(&mut self, tag: Tag<'a>) -> Result<(), W::Error> {
        match tag {
            Tag::HtmlBlock => Ok(()),
            Tag::Paragraph => self.writer.write_str("<p>"),
            Tag::Heading {level, id, classes, attrs } => {
                self.writer.write_str("<")?;
                write!(&mut self.writer, "{}", level)?;
                if let Some(id) = id {
                    self.writer.write_str(" id=\"")?;
                    escape_html(&mut self.writer, &id)?;
                    self.writer.write_str("\"")?;
                }
                let mut classes = classes.iter();
                if let Some(class) = classes.next() {
                    self.writer.write_str(" class=\"")?;
                    escape_html(&mut self.writer, class)?;
                    for class in classes {
                        self.writer.write_str(" ")?;
                        escape_html(&mut self.writer, class)?;
                    }
                    self.writer.write_str("\"")?;
                }
                for (attr, value) in attrs {
                    self.writer.write_str(" ")?;
                    escape_html(&mut self.writer, &attr)?;
                    if let Some(val) = value {
                        self.writer.write_str("=\"")?;
                        escape_html(&mut self.writer, &val)?;
                        self.writer.write_str("\"")?;
                    } else {
                        self.writer.write_str("=\"\"")?;
                    }
                }
                self.writer.write_str(">")
            },
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.writer.write_str("<table>")
            },
            Tag::TableHead => {
                self.table_state = TableState::Head;
                self.table_cell_index = 0;
                self.writer.write_str("<thead><tr>")
            },
            Tag::TableRow => {
                self.table_cell_index = 0;
                self.writer.write_str("<tr>")
            },
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => self.writer.write_str("<th")?,
                    TableState::Body => self.writer.write_str("<td")?,
                }

                match self.table_alignments.get(self.table_cell_index) {
                    Some(&Alignment::Left)   => self.writer.write_str(" style=\"text-align: left\">"),
                    Some(&Alignment::Center) => self.writer.write_str(" style=\"text-align: center\">"),
                    Some(&Alignment::Right)  => self.writer.write_str(" style=\"text-align: right\">"),
                    _ => self.writer.write_str(">"),
                }
            },
            Tag::BlockQuote(kind) => {
                let class_str = match kind {
                    None => "",
                    Some(kind) => match kind {
                        BlockQuoteKind::Note      => " class=\"markdown-alert-note\"",
                        BlockQuoteKind::Tip       => " class=\"markdown-alert-tip\"",
                        BlockQuoteKind::Important => " class=\"markdown-alert-important\"",
                        BlockQuoteKind::Warning   => " class=\"markdown-alert-warning\"",
                        BlockQuoteKind::Caution   => " class=\"markdown-alert-caution\"",
                    },
                };
                self.writer.write_str(&format!("<blockquote{}>", class_str))
            },
            Tag::CodeBlock(info) => {
                if let CodeBlockKind::Fenced(info) = info {
                    self.code_lang = info.split(' ').next().and_then(|s| Lang::form_str(s));
                }
                self.writer.write_str("<pre><code>")
            },
            Tag::List(Some(1)) => self.writer.write_str("<ol>"),
            Tag::List(Some(start)) => {
                self.writer.write_str("<ol start=\"")?;
                write!(&mut self.writer, "{}", start)?;
                self.writer.write_str("\">")
            },
            Tag::List(None)               => self.writer.write_str("<ul>"),
            Tag::Item                     => self.writer.write_str("<li>"),
            Tag::DefinitionList           => self.writer.write_str("<dl>"),
            Tag::DefinitionListTitle      => self.writer.write_str("<dt>"),
            Tag::DefinitionListDefinition => self.writer.write_str("<dd>"),
            Tag::Subscript                => self.writer.write_str("<sub>"),
            Tag::Superscript              => self.writer.write_str("<sup>"),
            Tag::Emphasis                 => self.writer.write_str("<em>"),
            Tag::Strong                   => self.writer.write_str("<strong>"),
            Tag::Strikethrough            => self.writer.write_str("<del>"),
            Tag::Link { link_type: LinkType::Email, dest_url, title, id: _ } => {
                self.writer.write_str("<a href=\"mailto:")?;
                escape_href(&mut self.writer, &dest_url)?;
                if !title.is_empty() {
                    self.writer.write_str("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.writer.write_str("\">")
            },
            Tag::Link { link_type: _, dest_url, title, id: _ } => {
                self.writer.write_str("<a href=\"")?;
                escape_href(&mut self.writer, &dest_url)?;
                if !title.is_empty() {
                    self.writer.write_str("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.writer.write_str("\">")
            },
            Tag::Image { link_type: _, dest_url, title, id: _ } => {
                self.writer.write_str("<img src=\"")?;
                escape_href(&mut self.writer, &dest_url)?;
                self.writer.write_str("\" alt=\"")?;
                self.raw_text()?;
                if !title.is_empty() {
                    self.writer.write_str("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.writer.write_str("\" />")
            },
            Tag::FootnoteDefinition(name) => {
                self.writer.write_str("<div class=\"footnote-definition\" id=\"")?;
                escape_html(&mut self.writer, &name)?;
                self.writer.write_str("\"><sup class=\"footnote-definition-label\">")?;
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                write!(&mut self.writer, "{}", number)?;
                self.writer.write_str("</sup>")
            },
            Tag::MetadataBlock(_) => {
                self.in_non_writing_block = true;
                Ok(())
            },
        }
    }

    fn end_tag(&mut self, tag: TagEnd) -> Result<(), W::Error> {
        match tag {
            TagEnd::HtmlBlock                    => (),
            TagEnd::Paragraph                    => self.writer.write_str("</p>")?,
            TagEnd::Heading(level) => write!(&mut self.writer, "</{}>", level)?,
            TagEnd::Table                        => self.writer.write_str("</tbody></table>")?,
            TagEnd::TableHead => {
                self.writer.write_str("</tr></thead><tbody>")?;
                self.table_state = TableState::Body;
            },
            TagEnd::TableRow => self.writer.write_str("</tr>")?,
            TagEnd::TableCell => {
                match self.table_state {
                    TableState::Head => self.writer.write_str("</th>")?,
                    TableState::Body => self.writer.write_str("</td>")?,
                }
                self.table_cell_index += 1;
            }
            TagEnd::CodeBlock => {
                if let Some(lang) = self.code_lang.take() {
                    if let Some(ptr) = self.code_start.take() {
                        let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, self.code_len)) };
                        LANG_DB.html(s, lang, &mut self.writer)?;
                        self.code_start = None;
                    }

                    self.code_len = 0;
                }
                self.writer.write_str("</code></pre>")?
            },
            TagEnd::BlockQuote(_)            => self.writer.write_str("</blockquote>")?,
            TagEnd::List(true)               => self.writer.write_str("</ol>")?,
            TagEnd::List(false)              => self.writer.write_str("</ul>")?,
            TagEnd::Item                     => self.writer.write_str("</li>")?,
            TagEnd::DefinitionList           => self.writer.write_str("</dl>")?,
            TagEnd::DefinitionListTitle      => self.writer.write_str("</dt>")?,
            TagEnd::DefinitionListDefinition => self.writer.write_str("</dd>")?,
            TagEnd::Emphasis                 => self.writer.write_str("</em>")?,
            TagEnd::Superscript              => self.writer.write_str("</sup>")?,
            TagEnd::Subscript                => self.writer.write_str("</sub>")?,
            TagEnd::Strong                   => self.writer.write_str("</strong>")?,
            TagEnd::Strikethrough            => self.writer.write_str("</del>")?,
            TagEnd::Link                     => self.writer.write_str("</a>")?,
            TagEnd::Image                    => (), // INFO shouldn't happen, handled in start
            TagEnd::FootnoteDefinition       => self.writer.write_str("</div>")?,
            TagEnd::MetadataBlock(_)         => self.in_non_writing_block = false,
        }
        Ok(())
    }

    fn raw_text(&mut self) -> Result<(), W::Error> {
        let mut nest = 0;
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(_) => nest += 1,
                Event::End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                },
                Event::Html(_) => (),
                Event::InlineHtml(text) | Event::Code(text) | Event::Text(text) => escape_html(&mut self.writer, &text)?,
                Event::InlineMath(text) => {
                    self.writer.write_str("$")?;
                    escape_html(&mut self.writer, &text)?;
                    self.writer.write_str("$")?;
                },
                Event::DisplayMath(text) => {
                    self.writer.write_str("$$")?;
                    escape_html(&mut self.writer, &text)?;
                    self.writer.write_str("$$")?;
                },
                Event::SoftBreak | Event::HardBreak | Event::Rule => self.writer.write_str(" ")?,
                Event::FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "[{}]", number)?;
                },
                Event::TaskListMarker(true)  => self.writer.write_str("[x]")?,
                Event::TaskListMarker(false) => self.writer.write_str("[ ]")?,
            }
        }
        Ok(())
    }
}