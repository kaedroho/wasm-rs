#[derive(Debug, PartialEq)]
enum NodeElement {
    Text(String),
    Node(Node),
}

#[derive(Debug, Default, PartialEq)]
struct Node {
    elements: Vec<NodeElement>,
}

impl Node {
    fn new() -> Node {
        Node::default()
    }
}

#[derive(Debug, Default)]
pub struct SExpressionParser {
    parsing_text_element: Option<String>,
    in_comment: bool,
    in_string_literal: bool,
    stack: Vec<Node>,
    root_nodes: Vec<Node>,
}

impl SExpressionParser {
    pub fn new() -> SExpressionParser {
        SExpressionParser::default()
    }

    fn finish_parsing_text_element(&mut self) {
        if let Some(text_element) = self.parsing_text_element.take() {
            match self.stack.last_mut() {
                Some(node) => node.elements.push(NodeElement::Text(text_element)),
                None => panic!("Text element found before opening bracket"),
            };
        }
    }

    pub fn feed_char(&mut self, c: char) {
        if self.in_comment {
            if c == '\n' {
                self.in_comment = false;
            } else {
                return
            }
        }

        if self.in_string_literal {
            if let Some(ref mut text_element) = self.parsing_text_element {
                text_element.push(c);
            }

            if c == '"' {
                self.in_string_literal = false;
            }

            return
        }

        match c {
            '(' => {  // Node start
                // Make new node
                self.stack.push(Node::new());
            }
            ')' => {  // Node end
                // Finish text element if we're parsing one
                self.finish_parsing_text_element();

                // Pop the node from the stack
                let node = match self.stack.pop() {
                    Some(node) => node,
                    None => panic!("Close bracket found without matching open bracket"),
                };

                // Append it to the parent node
                match self.stack.last_mut() {
                    Some(last) => last.elements.push(NodeElement::Node(node)),
                    None => self.root_nodes.push(node),
                }
            }
            ';' => {  // Comment
                self.in_comment = true;
            }
            '"' => {  // Open quote
                self.in_string_literal = true;

                match self.parsing_text_element {
                    Some(ref mut text_element) => text_element.push(c),
                    None => {
                        let mut text_element = String::new();
                        text_element.push(c);
                        self.parsing_text_element = Some(text_element);
                    }
                }
            }
            ' ' | '\t' | '\n' => {  // Whitespace
                // Finish text element if we're parsing one
                self.finish_parsing_text_element();
            }
            _ => {  // Text
                match self.parsing_text_element {
                    Some(ref mut text_element) => text_element.push(c),
                    None => {
                        let mut text_element = String::new();
                        text_element.push(c);
                        self.parsing_text_element = Some(text_element);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
fn parse(string: &str) -> Vec<Node> {
    let mut parser = SExpressionParser::new();

    for c in string.chars() {
        parser.feed_char(c);
    }

    // Parser stack must always be empty after parsing
    assert_eq!(parser.stack.len(), 0);

    parser.root_nodes
}

#[test]
fn test_simple_expressions() {
    let test = "(test)";

    assert_eq!(parse(test), vec![
        Node {
            elements: vec![
                NodeElement::Text("test".to_owned()),
            ]
        }
    ]);
}

#[test]
fn test_text_elements() {
    let test = "(test hello $world 123.4 こんにちは)";

    assert_eq!(parse(test), vec![
        Node {
            elements: vec![
                NodeElement::Text("test".to_owned()),
                NodeElement::Text("hello".to_owned()),
                NodeElement::Text("$world".to_owned()),
                NodeElement::Text("123.4".to_owned()),
                NodeElement::Text("こんにちは".to_owned()),
            ]
        }
    ]);
}

#[test]
fn test_line_comments() {
    let test = ";; Hello!\n(test ;; blah\n)";

    assert_eq!(parse(test), vec![
        Node {
            elements: vec![
                NodeElement::Text("test".to_owned()),
            ]
        }
    ]);
}

#[test]
fn test_string_literals() {
    let test = "(test hello \"$world 123.4 こんにちは\")";

    assert_eq!(parse(test), vec![
        Node {
            elements: vec![
                NodeElement::Text("test".to_owned()),
                NodeElement::Text("hello".to_owned()),
                NodeElement::Text("\"$world 123.4 こんにちは\"".to_owned()),
            ]
        }
    ]);
}

#[test]
fn test_sub_nodes() {
    let test = "(test (hello (world) (hi there)))";

    assert_eq!(parse(test), vec![
        Node {
            elements: vec![
                NodeElement::Text("test".to_owned()),
                NodeElement::Node(Node {
                    elements: vec![
                        NodeElement::Text("hello".to_owned()),
                        NodeElement::Node(Node {
                            elements: vec![
                                NodeElement::Text("world".to_owned()),
                            ]
                        }),
                        NodeElement::Node(Node {
                            elements: vec![
                                NodeElement::Text("hi".to_owned()),
                                NodeElement::Text("there".to_owned()),
                            ]
                        }),
                    ]
                }),
            ]
        }
    ]);
}