use std::cmp::Ordering;

use html_editor::{operation::*, parse, Element, Node};
use syntect::parsing::SyntaxSet;

use crate::generate_html_from_code;

const HTML_TYPES: &str = "span,pre,li,ul,ol,a,div,h1,h2,h3,h4,h5,h6,section,code";

pub trait ElementExtra {
    fn container(nodes: Vec<Node>) -> Self;
    fn get_attirbute(&self, attr: &str) -> Option<&String>;
    fn get_id(&self) -> Option<&String>;
    fn split_id(&self) -> Option<(i32, i32, Vec<&str>)>;
    fn are_childern_code(&self) -> Option<(&str, &String)>;
    fn code_children(&mut self, syntaxt_set: &SyntaxSet);
    fn seperate_html_elements(&mut self) -> &mut Self;
    fn sort_html_elements(&mut self) -> Self;
    fn to_html(self) -> Vec<String>;
}

impl ElementExtra for Element {
    fn get_attirbute(&self, attr: &str) -> Option<&String> {
        self.attrs
            .iter()
            .filter(|(head, _)| head == attr)
            .map(|(_, value)| value)
            .nth(0)
    }

    #[inline(always)]
    fn get_id(&self) -> Option<&String> {
        self.get_attirbute("id")
    }

    fn split_id(&self) -> Option<(i32, i32, Vec<&str>)> {
        let Some(id) = self.get_id() else {
            return None;
        };
        let [ps, family, ..] = id.split('@').collect::<Vec<_>>()[..] else {
            return None;
        };
        let [generation, index, ..] = ps
            .split(':')
            .flat_map(|x| x.parse::<i32>())
            .collect::<Vec<_>>()[..]
        else {
            return None;
        };
        let family = family.split(':').collect::<Vec<_>>();
        Some((generation, index, family))
    }

    fn are_childern_code(&self) -> Option<(&str, &String)> {
        if self.name != "pre" {
            return None;
        }
        let [one_node] = &self.children[..] else {
            return None;
        };
        let element = match one_node {
            Node::Element(element) => element,
            _ => return None,
        };
        if element.name != "code" {
            return None;
        }
        let Some(language) = element
            .get_attirbute("class")
            .and_then(|x| x.split('-').last())
        else {
            return None;
        };
        let [Node::Text(code)] = &element.children[..] else {
            return None;
        };
        Some((language, code))
    }

    fn code_children(&mut self, syntax_set: &SyntaxSet) {
        let Some((language, code)) = self.are_childern_code() else {
            return;
        };
        let attrs = self
            .attrs
            .iter()
            .filter(|x| x.0 != "class")
            .chain(&vec![(String::from("class"), String::from("code"))])
            .cloned()
            .collect::<Vec<_>>();

        let Ok(code) = generate_html_from_code(code, language, syntax_set) else {
            return;
        };
        let children = parse(&code).unwrap_or_default();
        self.children = children;
        self.attrs = attrs;
    }

    fn seperate_html_elements(&mut self) -> &mut Self {
        let mapping = |element: Element| {
            if element
                .children
                .iter()
                .any(|node| matches!(node, Node::Element(_)))
            {
                Element {
                    children: vec![],
                    ..element
                }
                .into_node()
            } else {
                element.into_node()
            }
        };
        self.children = self
            .query_all(&Selector::from(HTML_TYPES))
            .into_iter()
            .cloned()
            .map(mapping)
            .collect::<Vec<_>>();
        self
    }

    fn sort_html_elements(&mut self) -> Self {
        let mut children = self.children.clone();
        children.sort_by_key(|node| {
            node.as_element()
                .map(|element| element.split_id())
                .flatten()
                .map(|(x, y, z)| (z.len(),x, y))
        });
        self.children = children;
        self.to_owned()
    }

    fn to_html(self) -> Vec<String> {
        self.children
            .into_iter()
            .map(|x| x.html())
            .collect::<Vec<_>>()
    }

    fn container(nodes: Vec<Node>) -> Self {
        Element::new("FakeElement", vec![], nodes)
    }
}
