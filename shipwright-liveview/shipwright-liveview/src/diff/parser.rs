//! HTML parser for the diffing algorithm
//! 
//! This module provides a simple HTML parser that creates a tree structure
//! suitable for diffing operations.

use std::collections::HashMap;

/// A node in the HTML tree
#[derive(Debug, Clone, PartialEq)]
pub enum HtmlNode {
    Element(Element),
    Text(Text),
    Comment(Comment),
}

/// An HTML element
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<HtmlNode>,
    pub path: Vec<usize>,
    pub self_closing: bool,
}

/// A text node
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub content: String,
    pub path: Vec<usize>,
}

/// A comment node
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub content: String,
    pub path: Vec<usize>,
}

impl HtmlNode {
    /// Get the path to this node
    pub fn get_path(&self) -> &[usize] {
        match self {
            HtmlNode::Element(elem) => &elem.path,
            HtmlNode::Text(text) => &text.path,
            HtmlNode::Comment(comment) => &comment.path,
        }
    }
    
    /// Convert the node back to HTML
    pub fn to_html(&self) -> String {
        match self {
            HtmlNode::Element(elem) => {
                let mut html = format!("<{}", elem.tag_name);
                
                // Add attributes
                for (name, value) in &elem.attributes {
                    html.push_str(&format!(" {}=\"{}\"", name, escape_html(value)));
                }
                
                if elem.self_closing {
                    html.push_str(" />");
                } else {
                    html.push('>');
                    
                    // Add children
                    for child in &elem.children {
                        html.push_str(&child.to_html());
                    }
                    
                    html.push_str(&format!("</{}>", elem.tag_name));
                }
                
                html
            }
            HtmlNode::Text(text) => escape_html(&text.content),
            HtmlNode::Comment(comment) => format!("<!--{}-->", comment.content),
        }
    }
}

/// Parse HTML string into a tree structure
pub fn parse_html(html: &str) -> Result<HtmlNode, super::morphdom::DiffError> {
    let mut parser = Parser::new(html);
    parser.parse().ok_or_else(|| super::morphdom::DiffError::ParseError("Failed to parse HTML".to_string()))
}

/// Simple HTML parser
struct Parser {
    input: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }
    
    fn parse(&mut self) -> Option<HtmlNode> {
        self.skip_whitespace();
        self.parse_node(&[])
    }
    
    fn parse_node(&mut self, path: &[usize]) -> Option<HtmlNode> {
        if self.pos >= self.input.len() {
            return None;
        }
        
        if self.peek() == Some('<') {
            if self.peek_ahead(1) == Some('!') && self.peek_ahead(2) == Some('-') && self.peek_ahead(3) == Some('-') {
                self.parse_comment(path)
            } else if self.peek_ahead(1) == Some('/') {
                // Closing tag
                None
            } else {
                self.parse_element(path)
            }
        } else {
            self.parse_text(path)
        }
    }
    
    fn parse_element(&mut self, path: &[usize]) -> Option<HtmlNode> {
        self.consume('<')?;
        
        // Parse tag name
        let tag_name = self.parse_tag_name()?;
        
        // Parse attributes
        let mut attributes = HashMap::new();
        self.skip_whitespace();
        
        while self.peek() != Some('>') && self.peek() != Some('/') {
            let (name, value) = self.parse_attribute()?;
            attributes.insert(name, value);
            self.skip_whitespace();
        }
        
        // Check for self-closing tag
        let self_closing = self.peek() == Some('/');
        if self_closing {
            self.consume('/')?;
        }
        self.consume('>')?;
        
        // Parse children for non-self-closing tags
        let mut children = Vec::new();
        if !self_closing && !is_void_element(&tag_name) {
            let mut child_index = 0;
            loop {
                self.skip_whitespace();
                
                // Check for closing tag
                if self.peek() == Some('<') && self.peek_ahead(1) == Some('/') {
                    self.consume('<')?;
                    self.consume('/')?;
                    let closing_tag = self.parse_tag_name()?;
                    if closing_tag != tag_name {
                        return None; // Mismatched tags
                    }
                    self.skip_whitespace();
                    self.consume('>')?;
                    break;
                }
                
                // Parse child
                let mut child_path = path.to_vec();
                child_path.push(child_index);
                
                if let Some(child) = self.parse_node(&child_path) {
                    children.push(child);
                    child_index += 1;
                } else {
                    break;
                }
            }
        }
        
        Some(HtmlNode::Element(Element {
            tag_name,
            attributes,
            children,
            path: path.to_vec(),
            self_closing,
        }))
    }
    
    fn parse_text(&mut self, path: &[usize]) -> Option<HtmlNode> {
        let mut content = String::new();
        
        while self.pos < self.input.len() && self.peek() != Some('<') {
            content.push(self.input[self.pos]);
            self.pos += 1;
        }
        
        if content.is_empty() {
            None
        } else {
            Some(HtmlNode::Text(Text {
                content,
                path: path.to_vec(),
            }))
        }
    }
    
    fn parse_comment(&mut self, path: &[usize]) -> Option<HtmlNode> {
        self.consume('<')?;
        self.consume('!')?;
        self.consume('-')?;
        self.consume('-')?;
        
        let mut content = String::new();
        
        while self.pos < self.input.len() {
            if self.peek() == Some('-') && self.peek_ahead(1) == Some('-') && self.peek_ahead(2) == Some('>') {
                self.consume('-')?;
                self.consume('-')?;
                self.consume('>')?;
                break;
            }
            content.push(self.input[self.pos]);
            self.pos += 1;
        }
        
        Some(HtmlNode::Comment(Comment {
            content,
            path: path.to_vec(),
        }))
    }
    
    fn parse_tag_name(&mut self) -> Option<String> {
        let mut name = String::new();
        
        while self.pos < self.input.len() {
            let ch = self.peek()?;
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                name.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        
        if name.is_empty() {
            None
        } else {
            Some(name.to_lowercase())
        }
    }
    
    fn parse_attribute(&mut self) -> Option<(String, String)> {
        // Parse attribute name
        let name = self.parse_attribute_name()?;
        
        self.skip_whitespace();
        
        // Check for value
        if self.peek() == Some('=') {
            self.consume('=')?;
            self.skip_whitespace();
            
            // Parse value
            let value = if self.peek() == Some('"') || self.peek() == Some('\'') {
                self.parse_quoted_value()?
            } else {
                self.parse_unquoted_value()?
            };
            
            Some((name, value))
        } else {
            // Boolean attribute
            Some((name, String::new()))
        }
    }
    
    fn parse_attribute_name(&mut self) -> Option<String> {
        let mut name = String::new();
        
        while self.pos < self.input.len() {
            let ch = self.peek()?;
            if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == ':' {
                name.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    }
    
    fn parse_quoted_value(&mut self) -> Option<String> {
        let quote = self.peek()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        
        self.consume(quote)?;
        let mut value = String::new();
        
        while self.pos < self.input.len() {
            let ch = self.peek()?;
            if ch == quote {
                self.consume(quote)?;
                break;
            }
            value.push(ch);
            self.pos += 1;
        }
        
        Some(value)
    }
    
    fn parse_unquoted_value(&mut self) -> Option<String> {
        let mut value = String::new();
        
        while self.pos < self.input.len() {
            let ch = self.peek()?;
            if ch.is_whitespace() || ch == '>' || ch == '/' {
                break;
            }
            value.push(ch);
            self.pos += 1;
        }
        
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    
    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }
    
    fn consume(&mut self, expected: char) -> Option<()> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }
    
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
}

/// Check if an element is a void element (self-closing)
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" |
        "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.chars()
        .map(|ch| match ch {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => ch.to_string(),
        })
        .collect()
}