//! PyYAML safe-load compatibility adapter for HubertFA config loading.
//!
//! This module mirrors `inference/HubertFA/tools/config_utils.py::load_yaml`
//! for the fixture-bound PyYAML 6.0.3 SafeLoader contract. Rust uses
//! `saphyr-parser` only for YAML syntax events and source locations; PyYAML
//! resolver, constructor, merge, alias identity, and error projection behavior
//! is implemented here. Python remains the production runtime owner.

use saphyr_parser::{Event, Marker, Parser, ScalarStyle, Span, Tag};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::str::Utf8Error;

type RawRef = Rc<RefCell<RawNode>>;
type ValueRef = Rc<RefCell<ConstructedNode>>;
type LoadResult<T> = Result<T, Box<HfaPyyamlError>>;

/// Result of one compatibility load call.
#[derive(Debug, Clone)]
pub enum HfaPyyamlLoadResult {
    Ok(HfaPyyamlValue),
    Err(Box<HfaPyyamlError>),
}

impl HfaPyyamlLoadResult {
    /// Projects the result into the tagged JSON shape used by the parity
    /// fixtures.
    pub fn to_fixture_json(&self, temp_root: Option<&Path>) -> Value {
        match self {
            Self::Ok(value) => {
                json!({"ok": true, "value": value.to_fixture_json()})
            }
            Self::Err(error) => error.to_fixture_json(temp_root),
        }
    }
}

/// Loaded PyYAML-compatible value graph.
#[derive(Debug, Clone)]
pub struct HfaPyyamlValue {
    node: ValueRef,
}

impl HfaPyyamlValue {
    /// Projects the value into the tagged JSON shape used by the parity
    /// fixtures.
    pub fn to_fixture_json(&self) -> Value {
        Projector::default().project(&self.node)
    }
}

/// PyYAML-compatible load failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaPyyamlError {
    pub phase: &'static str,
    pub class_name: &'static str,
    pub message: String,
    pub context: Option<String>,
    pub problem: Option<String>,
    pub note: Option<String>,
    pub context_mark: Option<PyMark>,
    pub problem_mark: Option<PyMark>,
    pub extra: ErrorExtra,
}

impl HfaPyyamlError {
    fn to_fixture_json(&self, temp_root: Option<&Path>) -> Value {
        let mut error = Map::new();
        error.insert("phase".to_string(), json!(self.phase));
        error.insert("class".to_string(), json!(self.class_name));
        error.insert(
            "message".to_string(),
            json!(normalize_path(&self.message, temp_root)),
        );
        error.insert("context".to_string(), option_string(&self.context));
        error.insert("problem".to_string(), option_string(&self.problem));
        error.insert("note".to_string(), option_string(&self.note));
        error.insert(
            "context_mark".to_string(),
            mark_json(self.context_mark.as_ref(), temp_root),
        );
        error.insert(
            "problem_mark".to_string(),
            mark_json(self.problem_mark.as_ref(), temp_root),
        );

        match &self.extra {
            ErrorExtra::None => {}
            ErrorExtra::Io {
                errno,
                strerror,
                filename,
                filename2,
            } => {
                error.insert("errno".to_string(), json!(errno));
                error.insert("strerror".to_string(), json!(strerror));
                error.insert(
                    "filename".to_string(),
                    filename
                        .as_ref()
                        .map(|value| Value::String(normalize_path(value, temp_root)))
                        .unwrap_or(Value::Null),
                );
                error.insert(
                    "filename2".to_string(),
                    filename2
                        .as_ref()
                        .map(|value| Value::String(normalize_path(value, temp_root)))
                        .unwrap_or(Value::Null),
                );
            }
            ErrorExtra::Decode {
                encoding,
                reason,
                start,
                end,
                object_len,
                object_hex,
            } => {
                error.insert("encoding".to_string(), json!(encoding));
                error.insert("reason".to_string(), json!(reason));
                error.insert("start".to_string(), json!(start));
                error.insert("end".to_string(), json!(end));
                error.insert("object_len".to_string(), json!(object_len));
                error.insert("object_hex".to_string(), json!(object_hex));
            }
        }

        json!({"ok": false, "error": Value::Object(error)})
    }
}

impl fmt::Display for HfaPyyamlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HfaPyyamlError {}

fn boxed_error(error: HfaPyyamlError) -> Box<HfaPyyamlError> {
    Box::new(error)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorExtra {
    None,
    Io {
        errno: i32,
        strerror: String,
        filename: Option<String>,
        filename2: Option<String>,
    },
    Decode {
        encoding: &'static str,
        reason: String,
        start: usize,
        end: usize,
        object_len: usize,
        object_hex: String,
    },
}

/// Loads one UTF-8 YAML file with the PyYAML 6.0.3 SafeLoader compatibility
/// projection.
pub fn load_yaml_path(path: &Path) -> HfaPyyamlLoadResult {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => return HfaPyyamlLoadResult::Err(boxed_error(io_error(path, error))),
    };
    let content = match std::str::from_utf8(&bytes) {
        Ok(content) => content,
        Err(error) => return HfaPyyamlLoadResult::Err(boxed_error(decode_error(&bytes, error))),
    };
    load_yaml_str(content, &path.display().to_string())
}

/// Loads one YAML string with the PyYAML 6.0.3 SafeLoader compatibility
/// projection.
pub fn load_yaml_str(content: &str, source_name: &str) -> HfaPyyamlLoadResult {
    match load_yaml_str_inner(content, source_name) {
        Ok(value) => HfaPyyamlLoadResult::Ok(value),
        Err(error) => HfaPyyamlLoadResult::Err(error),
    }
}

fn load_yaml_str_inner(content: &str, source_name: &str) -> LoadResult<HfaPyyamlValue> {
    let raw = compose_single_document(content, source_name)?;
    let raw = raw.unwrap_or_else(|| scalar_node("~", source_name));
    let mut constructor = Constructor::new();
    let value = constructor.construct(&raw)?;
    Ok(HfaPyyamlValue { node: value })
}

fn compose_single_document(content: &str, source_name: &str) -> LoadResult<Option<RawRef>> {
    let mut builder = RawBuilder::new(content, source_name);
    for item in Parser::new_from_str(content) {
        let (event, span) =
            item.map_err(|error| boxed_error(map_parse_error(content, source_name, error)))?;
        builder.event(event, span)?;
    }
    builder.finish()
}

#[derive(Debug)]
struct RawNode {
    id: usize,
    tag: Option<String>,
    start: PyMark,
    kind: RawKind,
}

#[derive(Debug)]
enum RawKind {
    Scalar { value: String, style: ScalarStyle },
    Sequence(Vec<RawRef>),
    Mapping(Vec<(RawRef, RawRef)>),
}

struct RawBuilder<'a> {
    content: &'a str,
    source_name: &'a str,
    next_id: usize,
    anchors: HashMap<usize, RawRef>,
    anchor_marks: HashMap<String, PyMark>,
    stack: Vec<Frame>,
    current_document: Option<RawRef>,
    documents: Vec<RawRef>,
}

#[derive(Debug)]
struct Frame {
    node: RawRef,
    pending_key: Option<RawRef>,
}

impl<'a> RawBuilder<'a> {
    fn new(content: &'a str, source_name: &'a str) -> Self {
        Self {
            content,
            source_name,
            next_id: 1,
            anchors: HashMap::new(),
            anchor_marks: HashMap::new(),
            stack: Vec::new(),
            current_document: None,
            documents: Vec::new(),
        }
    }

    fn event(&mut self, event: Event<'_>, span: Span) -> LoadResult<()> {
        match event {
            Event::Nothing | Event::StreamStart | Event::StreamEnd => {}
            Event::DocumentStart(_) => {
                if !self.documents.is_empty() || self.current_document.is_some() {
                    let context_mark = self
                        .documents
                        .first()
                        .map(|node| node.borrow().start.clone());
                    let problem_mark = py_mark_from_marker(self.source_name, span.start);
                    return Err(boxed_error(composer_error(
                        Some("expected a single document in the stream"),
                        context_mark.as_ref(),
                        "but found another document",
                        &problem_mark,
                    )));
                }
                self.anchors.clear();
                self.anchor_marks.clear();
                self.current_document = None;
            }
            Event::DocumentEnd => {
                if let Some(node) = self.current_document.take() {
                    self.documents.push(node);
                }
            }
            Event::Scalar(value, style, anchor, tag) => {
                let node = self.new_node(
                    tag_to_string(tag.as_deref()),
                    mark_for_node(self.content, self.source_name, span, tag.as_deref()),
                    RawKind::Scalar {
                        value: value.to_string(),
                        style,
                    },
                );
                if anchor > 0 {
                    self.register_anchor_name(anchor, span)?;
                    self.anchors.insert(anchor, node.clone());
                }
                self.attach(node)?;
            }
            Event::SequenceStart(anchor, tag) => {
                let node = self.new_node(
                    tag_to_string(tag.as_deref()),
                    mark_for_node(self.content, self.source_name, span, tag.as_deref()),
                    RawKind::Sequence(Vec::new()),
                );
                if anchor > 0 {
                    self.register_anchor_name(anchor, span)?;
                    self.anchors.insert(anchor, node.clone());
                }
                self.stack.push(Frame {
                    node,
                    pending_key: None,
                });
            }
            Event::SequenceEnd => {
                let Some(frame) = self.stack.pop() else {
                    return Err(boxed_error(internal_constructor_error(
                        self.source_name,
                        span.start,
                        "unexpected sequence end",
                    )));
                };
                self.attach(frame.node)?;
            }
            Event::MappingStart(anchor, tag) => {
                let node = self.new_node(
                    tag_to_string(tag.as_deref()),
                    mark_for_node(self.content, self.source_name, span, tag.as_deref()),
                    RawKind::Mapping(Vec::new()),
                );
                if anchor > 0 {
                    self.register_anchor_name(anchor, span)?;
                    self.anchors.insert(anchor, node.clone());
                }
                self.stack.push(Frame {
                    node,
                    pending_key: None,
                });
            }
            Event::MappingEnd => {
                let Some(frame) = self.stack.pop() else {
                    return Err(boxed_error(internal_constructor_error(
                        self.source_name,
                        span.start,
                        "unexpected mapping end",
                    )));
                };
                self.attach(frame.node)?;
            }
            Event::Alias(anchor) => {
                let Some(node) = self.anchors.get(&anchor).cloned() else {
                    let mark = py_mark_from_marker(self.source_name, span.start);
                    return Err(boxed_error(composer_error(
                        None,
                        None,
                        "found undefined alias",
                        &mark,
                    )));
                };
                self.attach(node)?;
            }
        }
        Ok(())
    }

    fn register_anchor_name(&mut self, _anchor: usize, span: Span) -> LoadResult<()> {
        let Some((name, mark)) =
            anchor_name_before_node(self.content, self.source_name, span.start.index())
        else {
            return Ok(());
        };
        if let Some(first_mark) = self.anchor_marks.get(&name) {
            return Err(boxed_error(composer_error(
                Some(&format!(
                    "found duplicate anchor '{name}'; first occurrence"
                )),
                Some(first_mark),
                "second occurrence",
                &mark,
            )));
        }
        self.anchor_marks.insert(name, mark);
        Ok(())
    }

    fn finish(self) -> LoadResult<Option<RawRef>> {
        if !self.stack.is_empty() {
            let mark = self.stack.last().unwrap().node.borrow().start.clone();
            return Err(boxed_error(constructor_error(
                None,
                None,
                "unfinished YAML node",
                &mark,
            )));
        }
        Ok(self.documents.into_iter().next())
    }

    fn new_node(&mut self, tag: Option<String>, start: PyMark, kind: RawKind) -> RawRef {
        let id = self.next_id;
        self.next_id += 1;
        Rc::new(RefCell::new(RawNode {
            id,
            tag,
            start,
            kind,
        }))
    }

    fn attach(&mut self, node: RawRef) -> LoadResult<()> {
        if let Some(parent) = self.stack.last_mut() {
            let mut parent_node = parent.node.borrow_mut();
            match &mut parent_node.kind {
                RawKind::Sequence(items) => items.push(node),
                RawKind::Mapping(items) => {
                    if let Some(key) = parent.pending_key.take() {
                        items.push((key, node));
                    } else {
                        parent.pending_key = Some(node);
                    }
                }
                RawKind::Scalar { .. } => unreachable!(),
            }
        } else {
            self.current_document = Some(node);
        }
        Ok(())
    }
}

fn scalar_node(value: &str, source_name: &str) -> RawRef {
    Rc::new(RefCell::new(RawNode {
        id: 1,
        tag: None,
        start: PyMark::new(source_name, 0, 0, 0),
        kind: RawKind::Scalar {
            value: value.to_string(),
            style: ScalarStyle::Plain,
        },
    }))
}

struct Constructor {
    constructed: HashMap<usize, ValueRef>,
}

impl Constructor {
    fn new() -> Self {
        Self {
            constructed: HashMap::new(),
        }
    }

    fn construct(&mut self, raw: &RawRef) -> LoadResult<ValueRef> {
        let raw_id = raw.borrow().id;
        if let Some(value) = self.constructed.get(&raw_id) {
            return Ok(value.clone());
        }

        let tag = self.resolve_tag(raw)?;
        let raw_borrow = raw.borrow();
        match &raw_borrow.kind {
            RawKind::Scalar { value, .. } => self.construct_scalar(&raw_borrow, &tag, value),
            RawKind::Sequence(items) => self.construct_sequence(&raw_borrow, &tag, items),
            RawKind::Mapping(items) => self.construct_mapping_tag(&raw_borrow, &tag, items),
        }
    }

    fn resolve_tag(&self, raw: &RawRef) -> LoadResult<String> {
        let raw = raw.borrow();
        if let Some(tag) = &raw.tag {
            if tag != "!" {
                return Ok(tag.clone());
            }
        }
        Ok(match &raw.kind {
            RawKind::Scalar { value, style } => {
                if *style != ScalarStyle::Plain {
                    "tag:yaml.org,2002:str".to_string()
                } else {
                    resolve_plain_scalar(value)
                }
            }
            RawKind::Sequence(_) => "tag:yaml.org,2002:seq".to_string(),
            RawKind::Mapping(_) => "tag:yaml.org,2002:map".to_string(),
        })
    }

    fn construct_scalar(&mut self, raw: &RawNode, tag: &str, value: &str) -> LoadResult<ValueRef> {
        let kind = match tag {
            "tag:yaml.org,2002:null" => NodeKind::Null,
            "tag:yaml.org,2002:bool" => NodeKind::Bool(matches!(
                value.to_ascii_lowercase().as_str(),
                "yes" | "true" | "on"
            )),
            "tag:yaml.org,2002:int" => NodeKind::Int(construct_int(value)),
            "tag:yaml.org,2002:float" => NodeKind::Float(construct_float(value)),
            "tag:yaml.org,2002:binary" => {
                if let Some(message) = ascii_encode_error(value) {
                    return Err(boxed_error(constructor_error(
                        None,
                        None,
                        &format!("failed to convert base64 data into ascii: {message}"),
                        &raw.start,
                    )));
                }
                NodeKind::Bytes(decode_base64(value).map_err(|message| {
                    boxed_error(constructor_error(
                        None,
                        None,
                        &format!("failed to decode base64 data: {message}"),
                        &raw.start,
                    ))
                })?)
            }
            "tag:yaml.org,2002:timestamp" => construct_timestamp(value, &raw.start)?,
            "tag:yaml.org,2002:str" => NodeKind::String(value.to_string()),
            "tag:yaml.org,2002:value" => NodeKind::String(value.to_string()),
            unknown => {
                return Err(boxed_error(unknown_tag_error(unknown, &raw.start)));
            }
        };
        Ok(new_value_node(kind))
    }

    fn construct_sequence(
        &mut self,
        raw: &RawNode,
        tag: &str,
        items: &[RawRef],
    ) -> LoadResult<ValueRef> {
        match tag {
            "tag:yaml.org,2002:seq" => {
                let node = new_value_node(NodeKind::List(Vec::new()));
                self.constructed.insert(raw.id, node.clone());
                let mut children = Vec::with_capacity(items.len());
                for item in items {
                    children.push(self.construct(item)?);
                }
                node.borrow_mut().kind = NodeKind::List(children);
                Ok(node)
            }
            "tag:yaml.org,2002:omap" => self.construct_ordered_pairs(raw, items, "an ordered map"),
            "tag:yaml.org,2002:pairs" => self.construct_ordered_pairs(raw, items, "pairs"),
            unknown => Err(boxed_error(unknown_tag_error(unknown, &raw.start))),
        }
    }

    fn construct_ordered_pairs(
        &mut self,
        raw: &RawNode,
        items: &[RawRef],
        label: &'static str,
    ) -> LoadResult<ValueRef> {
        let node = new_value_node(NodeKind::List(Vec::new()));
        self.constructed.insert(raw.id, node.clone());
        let mut output = Vec::with_capacity(items.len());
        for item in items {
            let item_borrow = item.borrow();
            let RawKind::Mapping(entries) = &item_borrow.kind else {
                return Err(boxed_error(constructor_error(
                    Some(format!("while constructing {label}").as_str()),
                    Some(&raw.start),
                    "expected a mapping of length 1, but found scalar",
                    &item_borrow.start,
                )));
            };
            if entries.len() != 1 {
                return Err(boxed_error(constructor_error(
                    Some(format!("while constructing {label}").as_str()),
                    Some(&raw.start),
                    &format!(
                        "expected a single mapping item, but found {} items",
                        entries.len()
                    ),
                    &item_borrow.start,
                )));
            }
            let (key_raw, value_raw) = &entries[0];
            let key = self.construct(key_raw)?;
            let value = self.construct(value_raw)?;
            output.push(new_value_node(NodeKind::Tuple(vec![key, value])));
        }
        node.borrow_mut().kind = NodeKind::List(output);
        Ok(node)
    }

    fn construct_mapping_tag(
        &mut self,
        raw: &RawNode,
        tag: &str,
        items: &[(RawRef, RawRef)],
    ) -> LoadResult<ValueRef> {
        match tag {
            "tag:yaml.org,2002:map" => self.construct_mapping(raw, items),
            "tag:yaml.org,2002:set" => self.construct_set(raw, items),
            "tag:yaml.org,2002:omap" => Err(boxed_error(constructor_error(
                Some("while constructing an ordered map"),
                Some(&raw.start),
                "expected a sequence, but found mapping",
                &raw.start,
            ))),
            "tag:yaml.org,2002:pairs" => Err(boxed_error(constructor_error(
                Some("while constructing pairs"),
                Some(&raw.start),
                "expected a sequence, but found mapping",
                &raw.start,
            ))),
            unknown => Err(boxed_error(unknown_tag_error(unknown, &raw.start))),
        }
    }

    fn construct_mapping(
        &mut self,
        raw: &RawNode,
        items: &[(RawRef, RawRef)],
    ) -> LoadResult<ValueRef> {
        let node = new_value_node(NodeKind::Dict(Vec::new()));
        self.constructed.insert(raw.id, node.clone());
        let flattened = self.flatten_mapping(raw, items)?;
        let mut entries: Vec<(ValueRef, ValueRef)> = Vec::new();
        for (key_raw, value_raw) in flattened {
            let key = self.construct(&key_raw)?;
            if !is_hashable(&key) {
                let key_mark = key_raw.borrow().start.clone();
                return Err(boxed_error(constructor_error(
                    Some("while constructing a mapping"),
                    Some(&raw.start),
                    "found unhashable key",
                    &key_mark,
                )));
            }
            let value = self.construct(&value_raw)?;
            if let Some((_, existing_value)) = entries
                .iter_mut()
                .find(|(existing_key, _)| python_key_equal(existing_key, &key))
            {
                *existing_value = value;
            } else {
                entries.push((key, value));
            }
        }
        node.borrow_mut().kind = NodeKind::Dict(entries);
        Ok(node)
    }

    fn construct_set(&mut self, raw: &RawNode, items: &[(RawRef, RawRef)]) -> LoadResult<ValueRef> {
        let node = new_value_node(NodeKind::Set(Vec::new()));
        self.constructed.insert(raw.id, node.clone());
        let flattened = self.flatten_mapping(raw, items)?;
        let mut values = Vec::new();
        for (key_raw, _value_raw) in flattened {
            let key = self.construct(&key_raw)?;
            if !is_hashable(&key) {
                let key_mark = key_raw.borrow().start.clone();
                return Err(boxed_error(constructor_error(
                    Some("while constructing a mapping"),
                    Some(&raw.start),
                    "found unhashable key",
                    &key_mark,
                )));
            }
            if !values
                .iter()
                .any(|existing| python_key_equal(existing, &key))
            {
                values.push(key);
            }
        }
        node.borrow_mut().kind = NodeKind::Set(values);
        Ok(node)
    }

    fn flatten_mapping(
        &self,
        raw: &RawNode,
        items: &[(RawRef, RawRef)],
    ) -> LoadResult<Vec<(RawRef, RawRef)>> {
        let mut merge = Vec::new();
        let mut normal = Vec::new();
        for (key_raw, value_raw) in items {
            if self.resolve_tag(key_raw)? == "tag:yaml.org,2002:merge" {
                let value_borrow = value_raw.borrow();
                match &value_borrow.kind {
                    RawKind::Mapping(entries) => {
                        merge.extend(self.flatten_mapping(&value_borrow, entries)?);
                    }
                    RawKind::Sequence(nodes) => {
                        let mut submerge = Vec::new();
                        for node in nodes {
                            let node_borrow = node.borrow();
                            let RawKind::Mapping(entries) = &node_borrow.kind else {
                                return Err(boxed_error(constructor_error(
                                    Some("while constructing a mapping"),
                                    Some(&raw.start),
                                    "expected a mapping for merging, but found scalar",
                                    &node_borrow.start,
                                )));
                            };
                            submerge.push(self.flatten_mapping(&node_borrow, entries)?);
                        }
                        submerge.reverse();
                        for entries in submerge {
                            merge.extend(entries);
                        }
                    }
                    RawKind::Scalar { .. } => {
                        return Err(boxed_error(constructor_error(
                            Some("while constructing a mapping"),
                            Some(&raw.start),
                            "expected a mapping or list of mappings for merging, but found scalar",
                            &value_borrow.start,
                        )));
                    }
                }
            } else {
                normal.push((key_raw.clone(), value_raw.clone()));
            }
        }
        merge.extend(normal);
        Ok(merge)
    }
}

#[derive(Debug)]
struct ConstructedNode {
    kind: NodeKind,
}

#[derive(Debug, Clone)]
enum NodeKind {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Date(DateValue),
    DateTime(DateTimeValue),
    List(Vec<ValueRef>),
    Tuple(Vec<ValueRef>),
    Dict(Vec<(ValueRef, ValueRef)>),
    Set(Vec<ValueRef>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DateValue {
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DateTimeValue {
    date: DateValue,
    hour: u32,
    minute: u32,
    second: u32,
    microsecond: u32,
    tz_offset_seconds: Option<i32>,
}

fn new_value_node(kind: NodeKind) -> ValueRef {
    Rc::new(RefCell::new(ConstructedNode { kind }))
}

#[derive(Default)]
struct Projector {
    ids: HashMap<*const RefCell<ConstructedNode>, usize>,
    next_id: usize,
}

impl Projector {
    fn project(&mut self, node: &ValueRef) -> Value {
        match &node.borrow().kind {
            NodeKind::Null => json!({"tag": "null"}),
            NodeKind::Bool(value) => json!({"tag": "bool", "value": value}),
            NodeKind::Int(value) => json!({"tag": "int", "value": value.to_string()}),
            NodeKind::Float(value) => project_float(*value),
            NodeKind::String(value) => json!({"tag": "str", "value": value}),
            NodeKind::Bytes(value) => json!({
                "tag": "bytes",
                "hex": bytes_hex(value),
                "base64": encode_base64(value),
            }),
            NodeKind::Date(value) => json!({"tag": "date", "iso": value.iso()}),
            NodeKind::DateTime(value) => json!({
                "tag": "datetime",
                "iso": value.iso(),
                "tz_offset_seconds": value.tz_offset_seconds,
            }),
            NodeKind::List(items) => {
                if let Some(id) = self.existing_id(node) {
                    return json!({"tag": "ref", "id": id});
                }
                let id = self.assign_id(node);
                json!({
                    "tag": "list",
                    "id": id,
                    "items": items.iter().map(|item| self.project(item)).collect::<Vec<_>>(),
                })
            }
            NodeKind::Tuple(items) => json!({
                "tag": "tuple",
                "items": items.iter().map(|item| self.project(item)).collect::<Vec<_>>(),
            }),
            NodeKind::Dict(entries) => {
                if let Some(id) = self.existing_id(node) {
                    return json!({"tag": "ref", "id": id});
                }
                let id = self.assign_id(node);
                json!({
                    "tag": "dict",
                    "id": id,
                    "entries": entries
                        .iter()
                        .map(|(key, value)| json!({"key": self.project(key), "value": self.project(value)}))
                        .collect::<Vec<_>>(),
                })
            }
            NodeKind::Set(items) => {
                if let Some(id) = self.existing_id(node) {
                    return json!({"tag": "ref", "id": id});
                }
                let id = self.assign_id(node);
                let mut projected = items
                    .iter()
                    .map(|item| self.project(item))
                    .collect::<Vec<_>>();
                projected.sort_by_key(|item| serde_json::to_string(item).unwrap());
                json!({"tag": "set", "id": id, "items": projected})
            }
        }
    }

    fn existing_id(&self, node: &ValueRef) -> Option<usize> {
        self.ids.get(&Rc::as_ptr(node)).copied()
    }

    fn assign_id(&mut self, node: &ValueRef) -> usize {
        let id = if self.next_id == 0 { 1 } else { self.next_id };
        self.next_id = id + 1;
        self.ids.insert(Rc::as_ptr(node), id);
        id
    }
}

impl DateValue {
    fn iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl DateTimeValue {
    fn iso(&self) -> String {
        let mut text = format!(
            "{}T{:02}:{:02}:{:02}",
            self.date.iso(),
            self.hour,
            self.minute,
            self.second
        );
        if self.microsecond > 0 {
            text.push_str(&format!(".{:06}", self.microsecond));
        }
        if let Some(offset) = self.tz_offset_seconds {
            if offset == 0 {
                text.push_str("+00:00");
            } else {
                let sign = if offset < 0 { '-' } else { '+' };
                let abs = offset.abs();
                text.push_str(&format!("{sign}{:02}:{:02}", abs / 3600, (abs % 3600) / 60));
            }
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyMark {
    pub name: String,
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl PyMark {
    fn new(name: impl Into<String>, index: usize, line: usize, column: usize) -> Self {
        Self {
            name: name.into(),
            index,
            line,
            column,
        }
    }

    fn display(&self) -> String {
        format!(
            "  in \"{}\", line {}, column {}",
            self.name,
            self.line + 1,
            self.column + 1
        )
    }
}

fn py_mark_from_marker(source_name: &str, marker: Marker) -> PyMark {
    PyMark::new(
        source_name,
        marker.index(),
        marker.line().saturating_sub(1),
        marker.col(),
    )
}

fn mark_at_index(content: &str, source_name: &str, index: usize) -> PyMark {
    let mut line = 0usize;
    let mut column = 0usize;
    for (byte_index, ch) in content.char_indices() {
        if byte_index >= index {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    PyMark::new(source_name, index, line, column)
}

fn mark_for_node(content: &str, source_name: &str, span: Span, tag: Option<&Tag>) -> PyMark {
    if tag.is_some() {
        let index = find_tag_start(content, span.start.index()).unwrap_or(span.start.index());
        mark_at_index(content, source_name, index)
    } else {
        py_mark_from_marker(source_name, span.start)
    }
}

fn find_tag_start(content: &str, node_index: usize) -> Option<usize> {
    let prefix = &content[..node_index.min(content.len())];
    if let Some(bang_index) = prefix.rfind('!') {
        let token_start = prefix[..bang_index]
            .rfind(|ch: char| ch.is_whitespace() || matches!(ch, '[' | '{' | ',' | ':'))
            .map_or(0, |index| index + 1);
        return Some(token_start);
    }
    {
        let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
        content[line_start..]
            .find('!')
            .map(|index| line_start + index)
    }
}

fn mark_json(mark: Option<&PyMark>, temp_root: Option<&Path>) -> Value {
    let Some(mark) = mark else {
        return Value::Null;
    };
    json!({
        "name": normalize_path(&mark.name, temp_root),
        "index": mark.index,
        "line": mark.line,
        "column": mark.column,
        "line_1": mark.line + 1,
        "column_1": mark.column + 1,
        "pointer": Value::Null,
        "snippet": Value::Null,
    })
}

fn option_string(value: &Option<String>) -> Value {
    value
        .as_ref()
        .map(|text| Value::String(text.clone()))
        .unwrap_or(Value::Null)
}

fn normalize_path(value: &str, temp_root: Option<&Path>) -> String {
    temp_root.map_or_else(
        || value.to_string(),
        |root| value.replace(&root.display().to_string(), "<TMP>"),
    )
}

fn constructor_error(
    context: Option<&str>,
    context_mark: Option<&PyMark>,
    problem: &str,
    problem_mark: &PyMark,
) -> HfaPyyamlError {
    marked_error(
        "constructor",
        "yaml.constructor.ConstructorError",
        context,
        context_mark,
        problem,
        problem_mark,
    )
}

fn composer_error(
    context: Option<&str>,
    context_mark: Option<&PyMark>,
    problem: &str,
    problem_mark: &PyMark,
) -> HfaPyyamlError {
    marked_error(
        "composer",
        "yaml.composer.ComposerError",
        context,
        context_mark,
        problem,
        problem_mark,
    )
}

fn scanner_error(
    context: Option<&str>,
    context_mark: Option<&PyMark>,
    problem: &str,
    problem_mark: &PyMark,
) -> HfaPyyamlError {
    marked_error(
        "scanner",
        "yaml.scanner.ScannerError",
        context,
        context_mark,
        problem,
        problem_mark,
    )
}

fn parser_error(
    context: Option<&str>,
    context_mark: Option<&PyMark>,
    problem: &str,
    problem_mark: &PyMark,
) -> HfaPyyamlError {
    marked_error(
        "parser",
        "yaml.parser.ParserError",
        context,
        context_mark,
        problem,
        problem_mark,
    )
}

fn marked_error(
    phase: &'static str,
    class_name: &'static str,
    context: Option<&str>,
    context_mark: Option<&PyMark>,
    problem: &str,
    problem_mark: &PyMark,
) -> HfaPyyamlError {
    let mut lines = Vec::new();
    if let Some(context) = context {
        lines.push(context.to_string());
    }
    if let Some(context_mark) = context_mark {
        if context_mark.name != problem_mark.name
            || context_mark.line != problem_mark.line
            || context_mark.column != problem_mark.column
        {
            lines.push(context_mark.display());
        }
    }
    lines.push(problem.to_string());
    lines.push(problem_mark.display());
    HfaPyyamlError {
        phase,
        class_name,
        message: lines.join("\n"),
        context: context.map(str::to_string),
        problem: Some(problem.to_string()),
        note: None,
        context_mark: context_mark.cloned(),
        problem_mark: Some(problem_mark.clone()),
        extra: ErrorExtra::None,
    }
}

fn unknown_tag_error(tag: &str, mark: &PyMark) -> HfaPyyamlError {
    constructor_error(
        None,
        None,
        &format!("could not determine a constructor for the tag '{tag}'"),
        mark,
    )
}

fn internal_constructor_error(source_name: &str, marker: Marker, message: &str) -> HfaPyyamlError {
    let mark = py_mark_from_marker(source_name, marker);
    constructor_error(None, None, message, &mark)
}

fn value_error(message: impl Into<String>) -> HfaPyyamlError {
    HfaPyyamlError {
        phase: "python_value",
        class_name: "builtins.ValueError",
        message: message.into(),
        context: None,
        problem: None,
        note: None,
        context_mark: None,
        problem_mark: None,
        extra: ErrorExtra::None,
    }
}

fn io_error(path: &Path, error: std::io::Error) -> HfaPyyamlError {
    let errno = error.raw_os_error().unwrap_or(0);
    let strerror = match error.kind() {
        std::io::ErrorKind::NotFound => "No such file or directory".to_string(),
        std::io::ErrorKind::PermissionDenied => "Permission denied".to_string(),
        std::io::ErrorKind::IsADirectory => "Is a directory".to_string(),
        _ => error.to_string(),
    };
    let class_name = match error.kind() {
        std::io::ErrorKind::NotFound => "builtins.FileNotFoundError",
        std::io::ErrorKind::IsADirectory => "builtins.IsADirectoryError",
        std::io::ErrorKind::PermissionDenied => "builtins.PermissionError",
        _ => "builtins.OSError",
    };
    let filename = path.display().to_string();
    HfaPyyamlError {
        phase: "file_open",
        class_name,
        message: format!("[Errno {errno}] {strerror}: '{filename}'"),
        context: None,
        problem: None,
        note: None,
        context_mark: None,
        problem_mark: None,
        extra: ErrorExtra::Io {
            errno,
            strerror,
            filename: Some(filename),
            filename2: None,
        },
    }
}

fn decode_error(bytes: &[u8], error: Utf8Error) -> HfaPyyamlError {
    let start = error.valid_up_to();
    let end = error
        .error_len()
        .map_or(bytes.len(), |length| start + length);
    let offending = bytes.get(start).copied().unwrap_or(0);
    let reason = if error.error_len().is_none() {
        "unexpected end of data".to_string()
    } else if offending & 0b1100_0000 == 0b1000_0000 {
        "invalid continuation byte".to_string()
    } else {
        "invalid start byte".to_string()
    };
    let subject = if end - start == 1 {
        format!("byte 0x{offending:02x} in position {start}")
    } else {
        format!("bytes in position {start}-{}", end - 1)
    };
    HfaPyyamlError {
        phase: "utf8_decode",
        class_name: "builtins.UnicodeDecodeError",
        message: format!("'utf-8' codec can't decode {subject}: {reason}"),
        context: None,
        problem: None,
        note: None,
        context_mark: None,
        problem_mark: None,
        extra: ErrorExtra::Decode {
            encoding: "utf-8",
            reason,
            start,
            end,
            object_len: bytes.len(),
            object_hex: bytes_hex(&bytes[..bytes.len().min(32)]),
        },
    }
}

fn map_parse_error(
    content: &str,
    source_name: &str,
    error: saphyr_parser::ScanError,
) -> HfaPyyamlError {
    let info = error.info();
    if info.contains("unknown anchor") {
        let mark = py_mark_from_marker(source_name, *error.marker());
        let alias = alias_name_at(content, mark.index).unwrap_or_else(|| "missing".to_string());
        return composer_error(
            None,
            None,
            &format!("found undefined alias '{alias}'"),
            &mark,
        );
    }
    if info.contains("unknown escape character") {
        let context_mark = mark_at_index(content, source_name, content.find('"').unwrap_or(0));
        let problem_index = content
            .find("\\q")
            .map_or(context_mark.index, |index| index + 1);
        let problem_mark = mark_at_index(content, source_name, problem_index);
        return scanner_error(
            Some("while scanning a double-quoted scalar"),
            Some(&context_mark),
            "found unknown escape character 'q'",
            &problem_mark,
        );
    }
    if info.contains("unexpected end of stream") && info.contains("quoted scalar") {
        let context_mark = mark_at_index(content, source_name, content.find('"').unwrap_or(0));
        let problem_mark = mark_at_index(content, source_name, content.len());
        return scanner_error(
            Some("while scanning a quoted scalar"),
            Some(&context_mark),
            "found unexpected end of stream",
            &problem_mark,
        );
    }
    if info.contains("invalid escape sequence") {
        let context_mark = mark_at_index(content, source_name, content.find('!').unwrap_or(0));
        let problem_mark = mark_at_index(
            content,
            source_name,
            content.find("%Z").map_or(0, |index| index + 1),
        );
        return scanner_error(
            Some("while scanning a tag"),
            Some(&context_mark),
            "expected URI escape sequence of 2 hexadecimal numbers, but found 'Z'",
            &problem_mark,
        );
    }
    if info.contains("did not find expected node content") {
        let mark = py_mark_from_marker(source_name, *error.marker());
        return parser_error(
            Some("while parsing a flow node"),
            Some(&mark),
            "expected the node content, but found '<stream end>'",
            &mark,
        );
    }
    if info.contains("did not find expected key") {
        let context_mark = mark_at_index(content, source_name, 0);
        let problem_mark = py_mark_from_marker(source_name, *error.marker());
        return parser_error(
            Some("while parsing a block mapping"),
            Some(&context_mark),
            "expected <block end>, but found '<block mapping start>'",
            &problem_mark,
        );
    }
    if info.contains("handle wasn't declared") {
        let mark = py_mark_from_marker(source_name, *error.marker());
        return parser_error(
            Some("while parsing a node"),
            Some(&mark),
            "found undefined tag handle '!h!'",
            &mark,
        );
    }
    let mark = py_mark_from_marker(source_name, *error.marker());
    parser_error(None, None, info, &mark)
}

fn alias_name_at(content: &str, index: usize) -> Option<String> {
    let bytes = content.as_bytes();
    if bytes.get(index) != Some(&b'*') {
        return None;
    }
    let mut end = index + 1;
    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' {
            end += 1;
        } else {
            break;
        }
    }
    (end > index + 1).then(|| content[index + 1..end].to_string())
}

fn anchor_name_before_node(
    content: &str,
    source_name: &str,
    node_index: usize,
) -> Option<(String, PyMark)> {
    let search_end = node_index.min(content.len());
    let anchor_index = content[..search_end].rfind('&')?;
    let name_start = anchor_index + 1;
    let mut name_end = name_start;
    for (offset, ch) in content[name_start..].char_indices() {
        if !is_yaml_anchor_char(ch) {
            break;
        }
        name_end = name_start + offset + ch.len_utf8();
    }
    if name_end == name_start {
        return None;
    }
    Some((
        content[name_start..name_end].to_string(),
        mark_at_index(content, source_name, anchor_index),
    ))
}

fn is_yaml_anchor_char(ch: char) -> bool {
    !ch.is_whitespace() && !matches!(ch, '\0' | '[' | ']' | '{' | '}' | ',')
}

fn tag_to_string(tag: Option<&Tag>) -> Option<String> {
    let tag = tag?;
    if tag.handle.is_empty() && tag.suffix == "!" {
        return Some("!".to_string());
    }
    if tag.handle == "!" {
        return Some(format!("!{}", tag.suffix));
    }
    Some(format!("{}{}", tag.handle, tag.suffix))
}

fn resolve_plain_scalar(value: &str) -> String {
    if matches!(value, "~" | "null" | "Null" | "NULL" | "") {
        return "tag:yaml.org,2002:null".to_string();
    }
    if matches!(
        value,
        "yes"
            | "Yes"
            | "YES"
            | "no"
            | "No"
            | "NO"
            | "true"
            | "True"
            | "TRUE"
            | "false"
            | "False"
            | "FALSE"
            | "on"
            | "On"
            | "ON"
            | "off"
            | "Off"
            | "OFF"
    ) {
        return "tag:yaml.org,2002:bool".to_string();
    }
    if value == "<<" {
        return "tag:yaml.org,2002:merge".to_string();
    }
    if value == "=" {
        return "tag:yaml.org,2002:value".to_string();
    }
    if is_int_scalar(value) {
        return "tag:yaml.org,2002:int".to_string();
    }
    if is_float_scalar(value) {
        return "tag:yaml.org,2002:float".to_string();
    }
    if is_timestamp_scalar(value) {
        return "tag:yaml.org,2002:timestamp".to_string();
    }
    "tag:yaml.org,2002:str".to_string()
}

fn is_int_scalar(value: &str) -> bool {
    let rest = strip_sign(value).1;
    if rest == "0" {
        return true;
    }
    if let Some(binary) = rest.strip_prefix("0b") {
        return !binary.is_empty() && binary.chars().all(|ch| matches!(ch, '0' | '1' | '_'));
    }
    if let Some(hex) = rest.strip_prefix("0x") {
        return !hex.is_empty() && hex.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '_');
    }
    if rest.starts_with('0') && rest.len() > 1 {
        return rest.chars().all(|ch| matches!(ch, '0'..='7' | '_'));
    }
    if rest.contains(':') {
        let mut parts = rest.split(':');
        let first = parts.next().unwrap_or("");
        return !first.is_empty()
            && first.chars().all(|ch| ch.is_ascii_digit() || ch == '_')
            && parts.all(is_base60_part);
    }
    let mut chars = rest.chars();
    match chars.next() {
        Some('1'..='9') => chars.all(|ch| ch.is_ascii_digit() || ch == '_'),
        _ => false,
    }
}

fn is_base60_part(part: &str) -> bool {
    let cleaned = part.replace('_', "");
    if cleaned.is_empty() || cleaned.len() > 2 || !cleaned.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    cleaned.parse::<u32>().is_ok_and(|value| value <= 59)
}

fn is_float_scalar(value: &str) -> bool {
    let (sign, rest) = strip_sign(value);
    if rest.eq_ignore_ascii_case(".inf") {
        return true;
    }
    if sign.is_none() && matches!(rest, ".nan" | ".NaN" | ".NAN") {
        return true;
    }
    if rest.contains(':') && rest.contains('.') {
        let mut parts = rest.split(':');
        let first = parts.next().unwrap_or("");
        return !first.is_empty()
            && first.chars().all(|ch| ch.is_ascii_digit() || ch == '_')
            && parts.all(|part| {
                if let Some((whole, fraction)) = part.split_once('.') {
                    is_base60_part(whole)
                        && !fraction.is_empty()
                        && fraction.chars().all(|ch| ch.is_ascii_digit() || ch == '_')
                } else {
                    is_base60_part(part)
                }
            });
    }
    let Some((before_dot, after_dot)) = rest.split_once('.') else {
        return false;
    };
    let before_ok = before_dot.is_empty()
        || before_dot
            .chars()
            .all(|ch| ch.is_ascii_digit() || ch == '_');
    let (fraction, exponent) = split_exponent(after_dot);
    let fraction_ok = fraction.chars().all(|ch| ch.is_ascii_digit() || ch == '_');
    let has_digit = before_dot.chars().any(|ch| ch.is_ascii_digit())
        || fraction.chars().any(|ch| ch.is_ascii_digit());
    before_ok && fraction_ok && has_digit && exponent.is_none_or(valid_float_exponent)
}

fn split_exponent(value: &str) -> (&str, Option<&str>) {
    if let Some(index) = value.find(['e', 'E']) {
        (&value[..index], Some(&value[index + 1..]))
    } else {
        (value, None)
    }
}

fn valid_float_exponent(value: &str) -> bool {
    let Some(rest) = value.strip_prefix(['+', '-']) else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())
}

fn is_timestamp_scalar(value: &str) -> bool {
    if parse_date_only(value).is_some() {
        return true;
    }
    parse_datetime_parts(value).is_some()
}

fn strip_sign(value: &str) -> (Option<char>, &str) {
    match value.as_bytes().first() {
        Some(b'+') => (Some('+'), &value[1..]),
        Some(b'-') => (Some('-'), &value[1..]),
        _ => (None, value),
    }
}

fn construct_int(value: &str) -> i128 {
    let cleaned = value.replace('_', "");
    let (sign, rest) = strip_sign(&cleaned);
    let sign = if sign == Some('-') { -1 } else { 1 };
    let value = if rest == "0" {
        0
    } else if let Some(binary) = rest.strip_prefix("0b") {
        i128::from_str_radix(binary, 2).unwrap()
    } else if let Some(hex) = rest.strip_prefix("0x") {
        i128::from_str_radix(hex, 16).unwrap()
    } else if rest.starts_with('0') && rest.len() > 1 {
        i128::from_str_radix(rest, 8).unwrap()
    } else if rest.contains(':') {
        let mut base = 1i128;
        let mut total = 0i128;
        for part in rest.split(':').rev() {
            total += part.parse::<i128>().unwrap() * base;
            base *= 60;
        }
        total
    } else {
        rest.parse::<i128>().unwrap()
    };
    sign * value
}

fn construct_float(value: &str) -> f64 {
    let cleaned = value.replace('_', "").to_ascii_lowercase();
    let (sign_char, rest) = strip_sign(&cleaned);
    let sign = if sign_char == Some('-') { -1.0 } else { 1.0 };
    if rest == ".inf" {
        return sign * f64::INFINITY;
    }
    if rest == ".nan" {
        return f64::from_bits(0xfff8_0000_0000_0000);
    }
    if rest.contains(':') {
        let mut base = 1.0;
        let mut total = 0.0;
        for part in rest.split(':').rev() {
            total += part.parse::<f64>().unwrap() * base;
            base *= 60.0;
        }
        sign * total
    } else {
        sign * rest.parse::<f64>().unwrap()
    }
}

fn construct_timestamp(value: &str, mark: &PyMark) -> LoadResult<NodeKind> {
    if let Some(date) = parse_date_only(value) {
        return validate_date(date.year, date.month, date.day)
            .map(NodeKind::Date)
            .map_err(|message| boxed_error(value_error(message)));
    }
    let Some(mut datetime) = parse_datetime_parts(value) else {
        return Ok(NodeKind::String(value.to_string()));
    };
    datetime.date = validate_date(datetime.date.year, datetime.date.month, datetime.date.day)
        .map_err(|message| boxed_error(value_error(message)))?;
    validate_time(datetime.hour, datetime.minute, datetime.second)
        .map_err(|message| boxed_error(value_error(message)))?;
    let _ = mark;
    Ok(NodeKind::DateTime(datetime))
}

fn parse_date_only(value: &str) -> Option<DateValue> {
    let mut parts = value.split('-');
    let year = parts.next()?;
    let month = parts.next()?;
    let day = parts.next()?;
    if parts.next().is_some()
        || year.len() != 4
        || month.len() != 2
        || day.len() != 2
        || !year.chars().all(|ch| ch.is_ascii_digit())
        || !month.chars().all(|ch| ch.is_ascii_digit())
        || !day.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    Some(DateValue {
        year: year.parse().ok()?,
        month: month.parse().ok()?,
        day: day.parse().ok()?,
    })
}

fn parse_datetime_parts(value: &str) -> Option<DateTimeValue> {
    let separator = value.find(['T', 't']).or_else(|| value.find([' ', '\t']))?;
    let date_text = &value[..separator];
    let time_text = value[separator + 1..].trim_start();
    let mut date_parts = date_text.split('-');
    let year_text = date_parts.next()?;
    let month_text = date_parts.next()?;
    let day_text = date_parts.next()?;
    if date_parts.next().is_some()
        || year_text.len() != 4
        || !(1..=2).contains(&month_text.len())
        || !(1..=2).contains(&day_text.len())
        || !year_text.chars().all(|ch| ch.is_ascii_digit())
        || !month_text.chars().all(|ch| ch.is_ascii_digit())
        || !day_text.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    let date = DateValue {
        year: year_text.parse().ok()?,
        month: month_text.parse().ok()?,
        day: day_text.parse().ok()?,
    };

    let (main_time, tz_offset_seconds) = split_timezone_suffix(time_text)?;

    let mut time_parts = main_time.split(':');
    let hour_text = time_parts.next()?;
    let minute_text = time_parts.next()?;
    let second_and_fraction = time_parts.next()?;
    if time_parts.next().is_some() {
        return None;
    }
    let (second_text, fraction_text) = second_and_fraction
        .split_once('.')
        .map_or((second_and_fraction, ""), |(second, fraction)| {
            (second, fraction)
        });
    if !(1..=2).contains(&hour_text.len())
        || minute_text.len() != 2
        || second_text.len() != 2
        || !hour_text.chars().all(|ch| ch.is_ascii_digit())
        || !minute_text.chars().all(|ch| ch.is_ascii_digit())
        || !second_text.chars().all(|ch| ch.is_ascii_digit())
        || !fraction_text.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    let hour = hour_text.parse().ok()?;
    let minute = minute_text.parse().ok()?;
    let second = second_text.parse().ok()?;
    let mut fraction = fraction_text.chars().take(6).collect::<String>();
    while !fraction.is_empty() && fraction.len() < 6 {
        fraction.push('0');
    }
    let microsecond = if fraction.is_empty() {
        0
    } else {
        fraction.parse().ok()?
    };
    Some(DateTimeValue {
        date,
        hour,
        minute,
        second,
        microsecond,
        tz_offset_seconds,
    })
}

fn split_timezone_suffix(value: &str) -> Option<(&str, Option<i32>)> {
    let value = value.trim_end();
    if let Some(prefix) = value.strip_suffix('Z') {
        return Some((prefix.trim_end(), Some(0)));
    }
    for (index, ch) in value.char_indices().rev() {
        if ch != '+' && ch != '-' {
            continue;
        }
        if index == 0 {
            return None;
        }
        let prefix = value[..index].trim_end();
        let offset = &value[index + ch.len_utf8()..];
        let sign = if ch == '-' { -1 } else { 1 };
        let mut parts = offset.split(':');
        let hour_text = parts.next()?;
        let minute_text = parts.next();
        if parts.next().is_some()
            || !(1..=2).contains(&hour_text.len())
            || !hour_text.chars().all(|ch| ch.is_ascii_digit())
            || minute_text.is_some_and(|minute| {
                minute.len() != 2 || !minute.chars().all(|ch| ch.is_ascii_digit())
            })
        {
            return None;
        }
        let hour: i32 = hour_text.parse().ok()?;
        let minute: i32 = minute_text.unwrap_or("0").parse().ok()?;
        return Some((prefix, Some(sign * (hour * 3600 + minute * 60))));
    }
    Some((value, None))
}

fn validate_date(year: i32, month: u32, day: u32) -> Result<DateValue, String> {
    if !(1..=12).contains(&month) {
        return Err("month must be in 1..12".to_string());
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => unreachable!(),
    };
    if day < 1 || day > max_day {
        return Err("day is out of range for month".to_string());
    }
    Ok(DateValue { year, month, day })
}

fn validate_time(hour: u32, minute: u32, second: u32) -> Result<(), String> {
    if hour > 23 {
        return Err("hour must be in 0..23".to_string());
    }
    if minute > 59 {
        return Err("minute must be in 0..59".to_string());
    }
    if second > 59 {
        return Err("second must be in 0..59".to_string());
    }
    Ok(())
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    let cleaned = value
        .bytes()
        .filter(|byte| {
            byte.is_ascii_alphanumeric() || *byte == b'+' || *byte == b'/' || *byte == b'='
        })
        .collect::<Vec<_>>();
    let data_count = cleaned.iter().filter(|byte| **byte != b'=').count();
    if data_count % 4 == 1 {
        return Err(format!(
            "Invalid base64-encoded string: number of data characters ({data_count}) cannot be 1 more than a multiple of 4"
        ));
    }

    let mut bits = 0u32;
    let mut bit_count = 0u32;
    let mut output = Vec::new();
    for byte in cleaned {
        if byte == b'=' {
            break;
        }
        let Some(value) = base64_value(byte) else {
            continue;
        };
        bits = (bits << 6) | u32::from(value);
        bit_count += 6;
        while bit_count >= 8 {
            bit_count -= 8;
            output.push(((bits >> bit_count) & 0xff) as u8);
        }
    }
    Ok(output)
}

fn ascii_encode_error(value: &str) -> Option<String> {
    let mut start = None;
    let mut end = 0usize;
    let mut first = None;
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii() {
            if start.is_some() {
                break;
            }
            continue;
        }
        if start.is_none() {
            start = Some(index);
            first = Some(ch);
        }
        end = index + 1;
    }
    let start = start?;
    let last = end - 1;
    let subject = if start == last {
        format!(
            "character {} in position {start}",
            python_non_ascii_char_repr(first.unwrap())
        )
    } else {
        format!("characters in position {start}-{last}")
    };
    Some(format!(
        "'ascii' codec can't encode {subject}: ordinal not in range(128)"
    ))
}

fn python_non_ascii_char_repr(ch: char) -> String {
    let value = ch as u32;
    if value <= 0xff {
        format!("'\\x{value:02x}'")
    } else if value <= 0xffff {
        format!("'\\u{value:04x}'")
    } else {
        format!("'\\U{value:08x}'")
    }
}

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        output.push(TABLE[(b0 >> 2) as usize] as char);
        output.push(TABLE[(((b0 & 0b11) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[(((b1 & 0b1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(b2 & 0b111111) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

fn bytes_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn project_float(value: f64) -> Value {
    let sign = if value.is_sign_negative() { -1 } else { 1 };
    if value.is_nan() {
        return json!({"tag": "float", "kind": "nan", "repr": "nan", "hex": "nan", "sign": sign});
    }
    if value.is_infinite() {
        return json!({
            "tag": "float",
            "kind": "inf",
            "repr": if sign < 0 { "-inf" } else { "inf" },
            "hex": if sign < 0 { "-inf" } else { "inf" },
            "sign": sign,
        });
    }
    json!({
        "tag": "float",
        "kind": "finite",
        "repr": python_float_repr(value),
        "hex": python_float_hex(value),
        "sign": sign,
    })
}

fn python_float_repr(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0".to_string()
        } else {
            "0.0".to_string()
        };
    }
    let mut text = format!("{value:?}");
    if !text.contains('.') && !text.contains('e') {
        text.push_str(".0");
    }
    text
}

fn python_float_hex(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0x0.0p+0".to_string()
        } else {
            "0x0.0p+0".to_string()
        };
    }
    let bits = value.to_bits();
    let sign = if bits >> 63 == 1 { "-" } else { "" };
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    if exponent_bits == 0 {
        return format!("{sign}0x0.{fraction:013x}p-1022");
    }
    let exponent = exponent_bits - 1023;
    format!("{sign}0x1.{fraction:013x}p{exponent:+}")
}

fn is_hashable(value: &ValueRef) -> bool {
    match &value.borrow().kind {
        NodeKind::List(_) | NodeKind::Dict(_) | NodeKind::Set(_) => false,
        NodeKind::Tuple(items) => items.iter().all(is_hashable),
        _ => true,
    }
}

fn python_key_equal(left: &ValueRef, right: &ValueRef) -> bool {
    match (&left.borrow().kind, &right.borrow().kind) {
        (NodeKind::Bool(left), NodeKind::Bool(right)) => left == right,
        (NodeKind::Bool(left), NodeKind::Int(right))
        | (NodeKind::Int(right), NodeKind::Bool(left)) => i128::from(*left) == *right,
        (NodeKind::Bool(left), NodeKind::Float(right))
        | (NodeKind::Float(right), NodeKind::Bool(left)) => {
            right.is_finite() && *right == f64::from(u8::from(*left))
        }
        (NodeKind::Int(left), NodeKind::Int(right)) => left == right,
        (NodeKind::Int(left), NodeKind::Float(right))
        | (NodeKind::Float(right), NodeKind::Int(left)) => {
            right.is_finite() && *right == *left as f64
        }
        (NodeKind::Float(left), NodeKind::Float(right)) => {
            !left.is_nan() && !right.is_nan() && left == right
        }
        (NodeKind::Null, NodeKind::Null) => true,
        (NodeKind::String(left), NodeKind::String(right)) => left == right,
        (NodeKind::Bytes(left), NodeKind::Bytes(right)) => left == right,
        (NodeKind::Date(left), NodeKind::Date(right)) => left == right,
        (NodeKind::DateTime(left), NodeKind::DateTime(right)) => left == right,
        (NodeKind::Tuple(left), NodeKind::Tuple(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| python_key_equal(left, right))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_pyyaml_safe_load_contract.jsonl");
    static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            loop {
                let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir()
                    .join(format!("v2m-hfa-pyyaml-{}-{sequence}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("failed to create test directory: {error}"),
                }
            }
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn prepare_case_path(case: &Value, temp_root: &Path) -> PathBuf {
        let path = temp_root.join("case.yaml");
        match case.get("kind").and_then(Value::as_str).unwrap_or("file") {
            "file" => {
                fs::write(&path, case.get("content").and_then(Value::as_str).unwrap()).unwrap()
            }
            "bytes" => fs::write(
                &path,
                parse_hex(case.get("bytes_hex").unwrap().as_str().unwrap()),
            )
            .unwrap(),
            "directory" => fs::create_dir(&path).unwrap(),
            "missing" => {}
            other => panic!("unsupported fixture kind {other:?}"),
        }
        path
    }

    fn parse_hex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks(2)
            .map(|pair| {
                let text = std::str::from_utf8(pair).unwrap();
                u8::from_str_radix(text, 16).unwrap()
            })
            .collect()
    }

    fn run_case(case: &Value) -> Value {
        let directory = TestDirectory::new();
        let path = prepare_case_path(case, directory.path());
        let repeat = case.get("repeat").and_then(Value::as_u64).unwrap_or(1);
        let calls = (0..repeat)
            .map(|_| load_yaml_path(&path).to_fixture_json(Some(directory.path())))
            .collect::<Vec<_>>();
        json!({"calls": calls})
    }

    #[test]
    fn hfa_pyyaml_safe_load_contract_fixture_parity() {
        for line in FIXTURES.lines().filter(|line| !line.is_empty()) {
            let case: Value = serde_json::from_str(line).unwrap();
            let actual = run_case(&case);
            let expected = case.get("expect").unwrap();
            assert_eq!(
                &actual,
                expected,
                "{}",
                case.get("case_id").and_then(Value::as_str).unwrap()
            );
        }
    }
}
