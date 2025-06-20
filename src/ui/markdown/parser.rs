use crate::ui::markdown::{MarkdownResult};
use pulldown_cmark::{Event, Parser, Tag, CowStr, CodeBlockKind, HeadingLevel};

/// Internal representation of markdown elements
#[derive(Debug, Clone)]
pub enum MarkdownElement {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        spans: Vec<TextSpan>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    List {
        items: Vec<MarkdownElement>,
    },
    Quote {
        content: String,
    },
    HorizontalRule,
}

/// Text span with styling information
#[derive(Debug, Clone)]
pub enum TextSpan {
    Plain(String),
    Strong(String),
    Emphasis(String),
    Code(String),
    Link {
        text: String,
        url: String,
    },
}

/// Markdown parser using pulldown-cmark
pub struct MarkdownParser {
    options: pulldown_cmark::Options,
}

impl MarkdownParser {
    /// Create a new parser with default options
    pub fn new() -> MarkdownResult<Self> {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        options.insert(pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION);
        
        Ok(Self { options })
    }
    
    /// Create parser with custom options
    pub fn with_options(options: pulldown_cmark::Options) -> Self {
        Self { options }
    }
    
    /// Parse markdown text into our internal representation
    pub fn parse(&self, markdown: &str) -> MarkdownResult<Vec<MarkdownElement>> {
        let parser = Parser::new_ext(markdown, self.options);
        let mut elements = Vec::new();
        let _event_stack: Vec<Event> = Vec::new();
        
        // Collect all events first for easier processing
        let all_events: Vec<Event> = parser.collect();
        
        let mut i = 0;
        while i < all_events.len() {
            match &all_events[i] {
                Event::Start(tag) => {
                    let (element, consumed) = self.parse_element(&all_events[i..], tag)?;
                    if let Some(element) = element {
                        elements.push(element);
                    }
                    i += consumed;
                }
                Event::Rule => {
                    elements.push(MarkdownElement::HorizontalRule);
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        Ok(elements)
    }
    
    /// Parse a single markdown element from event stream
    fn parse_element(&self, events: &[Event], start_tag: &Tag) -> MarkdownResult<(Option<MarkdownElement>, usize)> {
        match start_tag {
            Tag::Heading(level, _, _) => {
                self.parse_heading(events, *level)
            }
            Tag::Paragraph => {
                self.parse_paragraph(events)
            }
            Tag::CodeBlock(kind) => {
                self.parse_code_block(events, kind)
            }
            Tag::List(_) => {
                self.parse_list(events)
            }
            Tag::BlockQuote => {
                self.parse_quote(events)
            }
            _ => {
                // Skip unsupported elements for now
                let consumed = self.find_matching_end(events, start_tag)?;
                Ok((None, consumed))
            }
        }
    }
    
    /// Parse heading element
    fn parse_heading(&self, events: &[Event], level: HeadingLevel) -> MarkdownResult<(Option<MarkdownElement>, usize)> {
        let mut text = String::new();
        let mut consumed = 1; // Start tag
        
        for (i, event) in events.iter().skip(1).enumerate() {
            consumed = i + 2; // +1 for skip(1), +1 for 0-based indexing
            
            match event {
                Event::Text(cow_str) => {
                    text.push_str(cow_str);
                }
                Event::End(Tag::Heading(..)) => {
                    break;
                }
                _ => {
                    // Handle other inline elements like emphasis, strong, etc.
                    if let Event::Start(Tag::Strong) = event {
                        // For simplicity, we'll just add the text content
                        // A more sophisticated parser would preserve formatting
                    }
                }
            }
        }
        
        let level_num = match level {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        };
        
        let element = MarkdownElement::Heading {
            level: level_num,
            text,
        };
        
        Ok((Some(element), consumed))
    }
    
    /// Parse paragraph element  
    fn parse_paragraph(&self, events: &[Event]) -> MarkdownResult<(Option<MarkdownElement>, usize)> {
        let mut spans = Vec::new();
        let mut consumed = 1; // Start tag
        let mut i = 1;
        
        while i < events.len() {
            match &events[i] {
                Event::Text(cow_str) => {
                    spans.push(TextSpan::Plain(cow_str.to_string()));
                    i += 1;
                }
                Event::Start(Tag::Strong) => {
                    let (span, span_consumed) = self.parse_strong_span(&events[i..])?;
                    spans.push(span);
                    i += span_consumed;
                }
                Event::Start(Tag::Emphasis) => {
                    let (span, span_consumed) = self.parse_emphasis_span(&events[i..])?;
                    spans.push(span);
                    i += span_consumed;
                }
                Event::Code(cow_str) => {
                    spans.push(TextSpan::Code(cow_str.to_string()));
                    i += 1;
                }
                Event::Start(Tag::Link(_, dest_url, _)) => {
                    let (span, span_consumed) = self.parse_link_span(&events[i..], dest_url)?;
                    spans.push(span);
                    i += span_consumed;
                }
                Event::End(Tag::Paragraph) => {
                    consumed = i + 1;
                    break;
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        let element = MarkdownElement::Paragraph { spans };
        Ok((Some(element), consumed))
    }
    
    /// Parse code block
    fn parse_code_block(&self, events: &[Event], kind: &CodeBlockKind) -> MarkdownResult<(Option<MarkdownElement>, usize)> {
        let language = match kind {
            CodeBlockKind::Fenced(cow_str) if !cow_str.is_empty() => Some(cow_str.to_string()),
            _ => None,
        };
        
        let mut code = String::new();
        let mut consumed = 1;
        
        for (i, event) in events.iter().skip(1).enumerate() {
            consumed = i + 2;
            
            match event {
                Event::Text(cow_str) => {
                    code.push_str(cow_str);
                }
                Event::End(Tag::CodeBlock(_)) => {
                    break;
                }
                _ => {}
            }
        }
        
        let element = MarkdownElement::CodeBlock { language, code };
        Ok((Some(element), consumed))
    }
    
    /// Parse list (simplified - doesn't handle nested lists perfectly)
    fn parse_list(&self, events: &[Event]) -> MarkdownResult<(Option<MarkdownElement>, usize)> {
        let mut items = Vec::new();
        let mut consumed = 1;
        let mut i = 1;
        
        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::Item) => {
                    // Find the content of this list item
                    let mut item_events = Vec::new();
                    let mut depth = 1;
                    let mut j = i + 1;
                    
                    while j < events.len() && depth > 0 {
                        match &events[j] {
                            Event::Start(Tag::Item) => depth += 1,
                            Event::End(Tag::Item) => depth -= 1,
                            _ => {}
                        }
                        if depth > 0 {
                            item_events.push(events[j].clone());
                        }
                        j += 1;
                    }
                    
                    // Parse the item content as a paragraph
                    if !item_events.is_empty() {
                        let item_content = self.extract_text_from_events(&item_events);
                        items.push(MarkdownElement::Paragraph {
                            spans: vec![TextSpan::Plain(item_content)],
                        });
                    }
                    
                    i = j;
                }
                Event::End(Tag::List(_)) => {
                    consumed = i + 1;
                    break;
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        let element = MarkdownElement::List { items };
        Ok((Some(element), consumed))
    }
    
    /// Parse block quote
    fn parse_quote(&self, events: &[Event]) -> MarkdownResult<(Option<MarkdownElement>, usize)> {
        let mut content = String::new();
        let mut consumed = 1;
        
        for (i, event) in events.iter().skip(1).enumerate() {
            consumed = i + 2;
            
            match event {
                Event::Text(cow_str) => {
                    content.push_str(cow_str);
                }
                Event::SoftBreak | Event::HardBreak => {
                    content.push('\n');
                }
                Event::End(Tag::BlockQuote) => {
                    break;
                }
                _ => {}
            }
        }
        
        let element = MarkdownElement::Quote { content };
        Ok((Some(element), consumed))
    }
    
    /// Parse strong/bold span
    fn parse_strong_span(&self, events: &[Event]) -> MarkdownResult<(TextSpan, usize)> {
        let mut text = String::new();
        let mut consumed = 1;
        
        for (i, event) in events.iter().skip(1).enumerate() {
            consumed = i + 2;
            
            match event {
                Event::Text(cow_str) => {
                    text.push_str(cow_str);
                }
                Event::End(Tag::Strong) => {
                    break;
                }
                _ => {}
            }
        }
        
        Ok((TextSpan::Strong(text), consumed))
    }
    
    /// Parse emphasis/italic span
    fn parse_emphasis_span(&self, events: &[Event]) -> MarkdownResult<(TextSpan, usize)> {
        let mut text = String::new();
        let mut consumed = 1;
        
        for (i, event) in events.iter().skip(1).enumerate() {
            consumed = i + 2;
            
            match event {
                Event::Text(cow_str) => {
                    text.push_str(cow_str);
                }
                Event::End(Tag::Emphasis) => {
                    break;
                }
                _ => {}
            }
        }
        
        Ok((TextSpan::Emphasis(text), consumed))
    }
    
    /// Parse link span
    fn parse_link_span(&self, events: &[Event], dest_url: &CowStr) -> MarkdownResult<(TextSpan, usize)> {
        let mut text = String::new();
        let mut consumed = 1;
        
        for (i, event) in events.iter().skip(1).enumerate() {
            consumed = i + 2;
            
            match event {
                Event::Text(cow_str) => {
                    text.push_str(cow_str);
                }
                Event::End(Tag::Link(_, _, _)) => {
                    break;
                }
                _ => {}
            }
        }
        
        Ok((TextSpan::Link {
            text,
            url: dest_url.to_string(),
        }, consumed))
    }
    
    /// Find the matching end tag for a start tag
    fn find_matching_end(&self, events: &[Event], start_tag: &Tag) -> MarkdownResult<usize> {
        let mut depth = 1;
        let mut consumed = 1;
        
        for (i, event) in events.iter().skip(1).enumerate() {
            consumed = i + 2;
            
            match event {
                Event::Start(tag) if std::mem::discriminant(tag) == std::mem::discriminant(start_tag) => {
                    depth += 1;
                }
                Event::End(_) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        
        Ok(consumed)
    }
    
    /// Extract plain text from a sequence of events
    fn extract_text_from_events(&self, events: &[Event]) -> String {
        let mut text = String::new();
        
        for event in events {
            match event {
                Event::Text(cow_str) => text.push_str(cow_str),
                Event::SoftBreak => text.push(' '),
                Event::HardBreak => text.push('\n'),
                _ => {}
            }
        }
        
        text
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new().unwrap()
    }
}