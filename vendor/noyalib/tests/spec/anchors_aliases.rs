// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

// YAML spec: Anchors and aliases

use std::collections::HashMap;

use noyalib::from_str;

#[test]
fn anchor_and_alias_scalar() {
    let m: HashMap<String, String> = from_str("first: &anchor Foo\nsecond: *anchor\n").unwrap();
    assert_eq!(m["first"], "Foo");
    assert_eq!(m["second"], "Foo");
}

#[test]
fn anchor_override() {
    let m: HashMap<String, String> =
        from_str("first: &a Foo\nsecond: *a\nthird: &a Bar\nfourth: *a\n").unwrap();
    assert_eq!(m["first"], "Foo");
    assert_eq!(m["second"], "Foo");
    assert_eq!(m["third"], "Bar");
    assert_eq!(m["fourth"], "Bar");
}

#[test]
fn anchor_on_sequence() {
    let m: HashMap<String, Vec<i64>> =
        from_str("original: &items\n  - 1\n  - 2\ncopy: *items\n").unwrap();
    assert_eq!(m["original"], vec![1, 2]);
    assert_eq!(m["copy"], vec![1, 2]);
}

#[test]
fn anchor_on_mapping() {
    use noyalib::Value;
    let v: Value = from_str(
        "defaults: &defaults\n  color: red\n  size: large\nitem:\n  <<: *defaults\n  name: widget\n",
    )
    .unwrap();
    let item = v.get("item").unwrap();
    assert_eq!(item.get("name").unwrap().as_str(), Some("widget"));
    assert_eq!(item.get("color").unwrap().as_str(), Some("red"));
    assert_eq!(item.get("size").unwrap().as_str(), Some("large"));
}

#[test]
fn merge_key_basic() {
    use noyalib::Value;
    let v: Value =
        from_str("base: &base\n  a: 1\n  b: 2\nderived:\n  <<: *base\n  c: 3\n").unwrap();
    let derived = v.get("derived").unwrap();
    assert_eq!(derived.get("a").unwrap().as_i64(), Some(1));
    assert_eq!(derived.get("b").unwrap().as_i64(), Some(2));
    assert_eq!(derived.get("c").unwrap().as_i64(), Some(3));
}

#[test]
fn merge_key_override() {
    use noyalib::Value;
    let v: Value =
        from_str("base: &base\n  x: 1\n  y: 2\nderived:\n  <<: *base\n  x: 10\n").unwrap();
    let derived = v.get("derived").unwrap();
    // Direct key takes precedence over merge
    assert_eq!(derived.get("x").unwrap().as_i64(), Some(10));
    assert_eq!(derived.get("y").unwrap().as_i64(), Some(2));
}

#[test]
fn merge_key_multiple_sources() {
    use noyalib::Value;
    let v: Value = from_str("a: &a\n  x: 1\nb: &b\n  y: 2\nc:\n  <<: [*a, *b]\n  z: 3\n").unwrap();
    let c = v.get("c").unwrap();
    assert_eq!(c.get("x").unwrap().as_i64(), Some(1));
    assert_eq!(c.get("y").unwrap().as_i64(), Some(2));
    assert_eq!(c.get("z").unwrap().as_i64(), Some(3));
}

#[test]
fn anchor_in_flow_sequence() {
    let v: Vec<Vec<i64>> = from_str("- &a [1, 2]\n- *a\n").unwrap();
    assert_eq!(v[0], vec![1, 2]);
    assert_eq!(v[1], vec![1, 2]);
}

// yaml-test-suite Y2GN — `:` is a valid character inside an anchor name.
// Anchor names terminate only at whitespace and flow indicators per
// YAML 1.2.2 §6.9.2 (ns-anchor-char = ns-char - c-flow-indicator).
#[test]
fn anchor_name_may_contain_colon() {
    use std::collections::HashMap;
    let m: HashMap<String, String> = from_str("---\nkey: &an:chor value\n").unwrap();
    assert_eq!(m["key"], "value");
}
