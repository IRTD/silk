use std::{
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    path::Path,
};

use crate::{
    handler::{Handler, HandlerFunc, Service},
    http::Method,
    param::Param,
    router::Response,
};

pub type PathVariables = HashMap<String, String>;

#[derive(Default, Debug, PartialEq)]
pub struct HttpNodeTree {
    root: HttpNode,
}

impl HttpNodeTree {
    pub fn new() -> Self {
        HttpNodeTree {
            root: HttpNode::default(),
        }
    }

    pub fn add_service(
        &mut self,
        path: impl ToString,
        services: ServiceCollection,
    ) -> Result<(), SegmentParseError> {
        let mut current_node = &mut self.root;
        for segment in Segment::parse_path(path) {
            let segment = segment?;
            if current_node.segment == segment {
                break;
            }

            let idx = current_node.get_leave_idx_or_add(segment);
            current_node = current_node.leaves.get_mut(idx).unwrap();
        }

        current_node.methods = services;
        Ok(())
    }

    pub fn get_node(
        &self,
        path: impl ToString,
    ) -> Option<Result<(&HttpNode, PathVariables), SegmentParseError>> {
        let mut current_node = &self.root;
        let mut path_vars = PathVariables::new();
        let iter = Segment::parse_path(path);
        match iter.first()? {
            Ok(seg) => {
                if &current_node.segment == seg {
                    return Some(Ok((current_node, path_vars)));
                }
            }
            Err(e) => return Some(Err(e.to_owned())),
        }

        for seg in iter {
            let seg = match seg {
                Ok(s) => s,
                Err(e) => return Some(Err(e)),
            };
            current_node = current_node.get_leave_path_vars(seg, &mut path_vars)?;
        }

        Some(Ok((current_node, path_vars)))
    }
}

#[derive(Debug, PartialEq)]
pub struct HttpNode {
    pub(crate) segment: Segment,
    pub(crate) leaves: VecDeque<HttpNode>,
    pub(crate) methods: ServiceCollection,
}

impl Default for HttpNode {
    fn default() -> Self {
        HttpNode {
            segment: Segment::Static("/".to_string()),
            leaves: VecDeque::new(),
            methods: ServiceCollection::default(),
        }
    }
}

impl HttpNode {
    pub fn new(segment: Segment, methods: ServiceCollection) -> Self {
        HttpNode {
            segment,
            leaves: VecDeque::new(),
            methods,
        }
    }

    pub fn get_leave(&self, segment: &Segment) -> Option<&HttpNode> {
        self.leaves.iter().find(|&seg| &seg.segment == segment)
    }

    pub fn get_leave_path_vars(
        &self,
        segment: Segment,
        path_vars: &mut PathVariables,
    ) -> Option<&HttpNode> {
        for leave in &self.leaves {
            if leave.segment.is_static() && leave.segment == segment {
                return Some(leave);
            } else if let Segment::Pattern { reference } = &leave.segment {
                path_vars.insert(reference.clone(), segment.get_string()[1..].to_string());
                return Some(leave);
            }
        }

        None
    }

    pub fn get_leave_idx_or_add(&mut self, segment: Segment) -> usize {
        for (idx, leave) in &mut self.leaves.iter().enumerate() {
            if leave.segment != segment {
                continue;
            }
            return idx;
        }
        let segment_is_static = segment.is_static();
        let node = HttpNode::new(segment, ServiceCollection::default());
        if segment_is_static {
            self.leaves.push_front(node);
        } else {
            self.leaves.push_back(node);
        }
        self.leaves.len() - 1
    }
}

#[derive(Default)]
pub struct ServiceCollection {
    get: Option<Box<dyn Service>>,
    post: Option<Box<dyn Service>>,
}

impl std::fmt::Debug for ServiceCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceCollection")
            .field("get", &self.get.is_some())
            .field("post", &self.post.is_some())
            .finish()
    }
}

impl PartialEq for ServiceCollection {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl ServiceCollection {
    pub fn set_get<F, P>(mut self, service: F) -> Self
    where
        F: Handler<P> + Send + Sync + 'static,
        P: Param,
    {
        self.get = Some(Box::new(HandlerFunc::<_, P>::new(service)));
        self
    }

    pub fn set_post<F, P>(mut self, service: F) -> Self
    where
        F: Handler<P> + Send + Sync + 'static,
        P: Param,
    {
        self.post = Some(Box::new(HandlerFunc::<_, P>::new(service)));
        self
    }

    pub fn get(&self) -> Option<&Box<dyn Service + 'static>> {
        self.get.as_ref()
    }

    pub fn post(&self) -> Option<&Box<dyn Service + 'static>> {
        self.post.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Segment {
    Static(String),
    Pattern { reference: String },
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SegmentParseError {
    #[error("Segment begins with {0}, should begin with '/'")]
    InvalidStart(char),
    #[error("Empty Input")]
    EmptyInput,
    #[error("Expected '}}' at end")]
    ExpectedClosingCurly,
    #[error("Pattern missing identifier, found '{{}}'")]
    EmptyPattern,
}

impl Segment {
    pub fn parse(s: String) -> Result<Self, SegmentParseError> {
        if !s.starts_with('/') {
            return Err(SegmentParseError::InvalidStart(
                s.chars().next().ok_or(SegmentParseError::EmptyInput)?,
            ));
        }
        let mut seg_string = String::with_capacity(s.len());
        let mut is_pattern = false;
        let mut chars = s.chars();
        loop {
            let ch = chars.next();
            if is_pattern {
                match ch {
                    Some('}') => {
                        if seg_string.is_empty() {
                            return Err(SegmentParseError::EmptyPattern);
                        }
                        return Ok(Segment::Pattern {
                            reference: seg_string,
                        });
                    }
                    Some(c) => seg_string.push(c),
                    None => return Err(SegmentParseError::ExpectedClosingCurly),
                }
            } else {
                match ch {
                    Some('{') => {
                        seg_string.clear();
                        is_pattern = true;
                    }
                    Some(c) => seg_string.push(c),
                    None => return Ok(Segment::Static(seg_string)),
                }
            }
        }
    }

    pub fn get_string(&self) -> &String {
        match self {
            Segment::Static(s) => s,
            Segment::Pattern { reference } => reference,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Segment::Static(_) => true,
            Segment::Pattern { reference } => false,
        }
    }

    pub fn parse_path(input: impl ToString) -> Vec<Result<Segment, SegmentParseError>> {
        parse_raw_segs(input)
            .into_iter()
            .map(|seg| Segment::parse(seg))
            .collect()
    }
}

fn parse_raw_segs(input: impl ToString) -> Vec<String> {
    let mut segs = Vec::new();
    let mut current = String::new();
    for ch in input.to_string().chars() {
        match (ch, current.is_empty()) {
            ('/', true) => {
                current.push(ch);
            }
            ('/', false) => {
                segs.push(std::mem::take(&mut current));
                current.push(ch);
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        segs.push(current);
    }

    segs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_correct_static_segment() {
        let input = "/home".to_string();
        let expected = Segment::Static("/home".to_string());
        assert_eq!(Ok(expected), Segment::parse(input));
    }

    #[test]
    fn parses_correct_pattern_segment() {
        let input = "/{name}".to_string();
        let expected = Segment::Pattern {
            reference: String::from("name"),
        };
        assert_eq!(Ok(expected), Segment::parse(input));
    }

    #[test]
    fn segment_parse_path() {
        let input = "/home/{name}/hello";
        let expected = vec![
            Segment::Static("/home".to_string()),
            Segment::Pattern {
                reference: String::from("name"),
            },
            Segment::Static("/hello".to_string()),
        ];
        let iter = Segment::parse_path(input);
        for (parsed_seg, expected_seg) in iter.into_iter().zip(expected) {
            assert_eq!(Ok(expected_seg), parsed_seg);
        }
    }

    #[test]
    fn segment_parse_path_single_slash() {
        let input = "/";
        let expected = Segment::Static("/".to_string());
        let mut iter = Segment::parse_path(input);
        assert_eq!(1, iter.len());
        assert_eq!(Ok(expected), iter.remove(0));
    }

    #[test]
    fn tree_adds_non_existent() {
        let path = "/home/user";
        let expected = HttpNodeTree {
            root: HttpNode {
                segment: Segment::Static("/".to_string()),
                leaves: vec![HttpNode {
                    segment: Segment::Static("/home".to_string()),
                    leaves: vec![HttpNode {
                        segment: Segment::Static("/user".to_string()),
                        leaves: VecDeque::new(),
                        methods: ServiceCollection::default(),
                    }]
                    .into(),
                    methods: ServiceCollection::default(),
                }]
                .into(),
                methods: ServiceCollection::default(),
            },
        };
        let mut tree = HttpNodeTree::new();
        let res = tree.add_service(path, ServiceCollection::default());
        assert_eq!(Ok(()), res);
        assert_eq!(expected, tree);
    }

    #[test]
    fn tree_extend_path() {
        let initial_path = "/home";
        let follow_up_path = "/home/user";
        let expected = HttpNodeTree {
            root: HttpNode {
                segment: Segment::Static("/".to_string()),
                leaves: vec![HttpNode {
                    segment: Segment::Static("/home".to_string()),
                    leaves: vec![HttpNode {
                        segment: Segment::Static("/user".to_string()),
                        leaves: VecDeque::new(),
                        methods: ServiceCollection::default(),
                    }]
                    .into(),
                    methods: ServiceCollection::default(),
                }]
                .into(),
                methods: ServiceCollection::default(),
            },
        };
        let mut tree = HttpNodeTree::new();
        assert!(
            tree.add_service(initial_path, ServiceCollection::default())
                .is_ok()
        );

        assert!(
            tree.add_service(follow_up_path, ServiceCollection::default())
                .is_ok()
        );

        assert_eq!(expected, tree);
    }

    #[test]
    fn tree_get_node_static() {
        let input = "/home/user";
        let mut tree = HttpNodeTree::new();
        assert!(
            tree.add_service(input, ServiceCollection::default())
                .is_ok()
        );
        let expected = HttpNode {
            segment: Segment::Static("/user".to_string()),
            leaves: VecDeque::new(),
            methods: ServiceCollection::default(),
        };
        let (node, vars) = tree.get_node(input).unwrap().unwrap();
        assert_eq!(HashMap::new(), vars);
        assert_eq!(&expected, node);
    }

    #[test]
    fn tree_get_node_pattern() {
        let register_input = "/home/{user}";
        let incoming_input = "/home/Steve";
        let mut tree = HttpNodeTree::new();
        assert!(
            tree.add_service(register_input, ServiceCollection::default())
                .is_ok()
        );
        let expected = HttpNode {
            segment: Segment::Pattern {
                reference: String::from("user"),
            },
            leaves: VecDeque::new(),
            methods: ServiceCollection::default(),
        };
        let mut expected_path_vars = PathVariables::new();
        expected_path_vars.insert("user".to_string(), "Steve".to_string());
        let res = tree.get_node(incoming_input);

        assert_eq!(Some(Ok((&expected, expected_path_vars))), res);
    }

    #[test]
    fn tree_prefer_static_path_over_pattern() {
        let register_input_pattern = "/home/{user}";
        let register_input_static = "/home/valia";
        let incoming_pattern = "/home/steve";
        let mut tree = HttpNodeTree::new();
        assert!(
            tree.add_service(register_input_pattern, ServiceCollection::default())
                .is_ok()
        );
        assert!(
            tree.add_service(register_input_static, ServiceCollection::default())
                .is_ok()
        );
        let expected = HttpNode {
            segment: Segment::Static("/valia".to_string()),
            leaves: VecDeque::new(),
            methods: ServiceCollection::default(),
        };
        let expected_path_vars = PathVariables::new();

        let res = tree.get_node(register_input_static);
        assert_eq!(Some(Ok((&expected, expected_path_vars))), res);

        let expected = HttpNode {
            segment: Segment::Pattern {
                reference: String::from("user"),
            },
            leaves: VecDeque::new(),
            methods: ServiceCollection::default(),
        };
        let mut expected_path_vars = PathVariables::new();
        expected_path_vars.insert("user".to_string(), "steve".to_string());

        let res = tree.get_node(incoming_pattern);
        assert_eq!(Some(Ok((&expected, expected_path_vars))), res);
    }

    #[test]
    fn tree_return_root() {
        let dummy_register = "/home";
        let mut tree = HttpNodeTree::new();
        assert!(
            tree.add_service(dummy_register, ServiceCollection::default())
                .is_ok()
        );

        let mut expected = HttpNode::default();
        expected.get_leave_idx_or_add(Segment::Static("/home".to_string()));
        let expected_path_vars = PathVariables::new();

        assert_eq!(
            Some(Ok((&expected, expected_path_vars))),
            tree.get_node("/")
        );
    }
}
