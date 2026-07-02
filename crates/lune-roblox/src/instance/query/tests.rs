use rbx_dom_weak::types::{
    Attributes as DomAttributes, Tags as DomTags, Variant as DomValue, Vector3 as DomVector3,
};

use crate::instance::{Instance, query::query_descendants};

fn child(parent: Instance, class: &str, name: &str) -> Instance {
    let inst = Instance::new_orphaned(class);
    inst.set_name(name);
    inst.set_parent(Some(parent));
    inst
}

fn names(instances: &[Instance]) -> Vec<String> {
    instances.iter().map(Instance::get_name).collect()
}

fn run(root: Instance, selector: &str) -> Vec<String> {
    names(&query_descendants(root, selector).unwrap())
}

// Builds:
//   root (Folder)
//   |- ModelA (Model)        [tag Apple, attr Variety=Fuji, Flavor=4, OnFire=true, Vec=Vector3]
//   |  |- RedTree (Part)     [CanCollide=false, tag Fruit]
//   |  '- MeshA (MeshPart)   [tag SwordPart]
//   |- FolderB (Folder)
//   |  '- ModelC (Model)
//   |     '- PartC (Part)    [default CanCollide=true]
//   '- PartD (Part)          [attr OnFire=true]
fn build_tree() -> Instance {
    let root = Instance::new_orphaned("Folder");
    root.set_name("QDRoot");

    let model_a = child(root, "Model", "ModelA");
    model_a.set_property(
        "Tags",
        DomValue::Tags(DomTags::from(vec!["Apple".to_string()])),
    );
    let mut attrs = DomAttributes::new();
    attrs.insert("Variety".into(), DomValue::String("Fuji".into()));
    attrs.insert("Flavor".into(), DomValue::Float64(4.0));
    attrs.insert("OnFire".into(), DomValue::Bool(true));
    attrs.insert(
        "Vec".into(),
        DomValue::Vector3(DomVector3::new(1.0, 2.0, 3.0)),
    );
    model_a.set_property("Attributes", DomValue::Attributes(attrs));

    let red_tree = child(model_a, "Part", "RedTree");
    red_tree.set_property("CanCollide", DomValue::Bool(false));
    red_tree.set_property(
        "Tags",
        DomValue::Tags(DomTags::from(vec!["Fruit".to_string()])),
    );

    let mesh_a = child(model_a, "MeshPart", "MeshA");
    mesh_a.set_property(
        "Tags",
        DomValue::Tags(DomTags::from(vec!["SwordPart".to_string()])),
    );

    let folder_b = child(root, "Folder", "FolderB");
    let model_c = child(folder_b, "Model", "ModelC");
    child(model_c, "Part", "PartC");

    let part_d = child(root, "Part", "PartD");
    let mut d_attrs = DomAttributes::new();
    d_attrs.insert("OnFire".into(), DomValue::Bool(true));
    part_d.set_property("Attributes", DomValue::Attributes(d_attrs));

    root
}

#[test]
fn class_is_a_and_order() {
    let root = build_tree();
    // IsA + preorder DFS order (parent before children, left to right).
    assert_eq!(run(root, "Part"), vec!["RedTree", "PartC", "PartD"]);
    assert_eq!(run(root, "MeshPart"), vec!["MeshA"]);
    // Unknown class matches nothing (no error). Case-sensitive.
    assert_eq!(run(root, "Paart"), Vec::<String>::new());
    assert_eq!(run(root, "meshpart"), Vec::<String>::new());
}

#[test]
fn tag_name_attribute() {
    let root = build_tree();
    assert_eq!(run(root, ".Fruit"), vec!["RedTree"]);
    assert_eq!(run(root, "#RedTree"), vec!["RedTree"]);
    assert_eq!(run(root, "[$FuelCapacity]"), Vec::<String>::new());
    // Both ModelA and PartD carry OnFire=true (preorder: ModelA first).
    assert_eq!(run(root, "[$OnFire = true]"), vec!["ModelA", "PartD"]);
    assert_eq!(run(root, "[$Variety = Fuji]"), vec!["ModelA"]);
    assert_eq!(run(root, "[$Variety = fuji]"), Vec::<String>::new());
}

#[test]
fn attribute_number_coercion() {
    let root = build_tree();
    // Numeric attribute matches unquoted and quoted numeric literals.
    assert_eq!(run(root, "[$Flavor = 4]"), vec!["ModelA"]);
    assert_eq!(run(root, r#"[$Flavor = "4"]"#), vec!["ModelA"]);
    assert_eq!(run(root, r#"[$Flavor = "4.0"]"#), vec!["ModelA"]);
}

#[test]
fn property_default_fallback() {
    let root = build_tree();
    // CanCollide defaults to true on Parts/MeshParts; RedTree is explicitly
    // false. Effective-value comparison must honor the reflection default.
    assert_eq!(
        run(root, "[CanCollide = true]"),
        vec!["MeshA", "PartC", "PartD"]
    );
    assert_eq!(run(root, "[CanCollide = false]"), vec!["RedTree"]);
}

#[test]
fn unsupported_type_errors() {
    let root = build_tree();
    // A Color3 property cannot be compared.
    let err = query_descendants(root, "[Color = 1]").unwrap_err();
    assert!(matches!(
        err,
        super::QueryError::UnsupportedPropertyType { .. }
    ));
    // Neither can a Vector3 attribute.
    let err = query_descendants(root, "[$Vec = 1]").unwrap_err();
    assert!(matches!(
        err,
        super::QueryError::UnsupportedAttributeType { .. }
    ));
}

#[test]
fn compound_and_whitespace() {
    let root = build_tree();
    // Whitespace is insignificant: a run of simples is one compound (AND).
    assert_eq!(run(root, "Part #RedTree"), vec!["RedTree"]);
    assert_eq!(run(root, "Part#RedTree"), vec!["RedTree"]);
    assert_eq!(run(root, "Model.Apple"), vec!["ModelA"]);
    // "Model Part" = IsA Model AND IsA Part = nothing.
    assert_eq!(run(root, "Model Part"), Vec::<String>::new());
}

#[test]
fn combinators() {
    let root = build_tree();
    // Direct children of root only.
    assert_eq!(run(root, "> Part"), vec!["PartD"]);
    // .SwordPart whose parent IsA Model.
    assert_eq!(run(root, "Model > .SwordPart"), vec!["MeshA"]);
    // Parts that are descendants of any Model.
    assert_eq!(run(root, "Model >> Part"), vec!["RedTree", "PartC"]);
}

#[test]
fn selector_list_no_dedup() {
    let root = build_tree();
    // Per-selector grouping, no de-duplication.
    assert_eq!(
        run(root, "Part, Part"),
        vec!["RedTree", "PartC", "PartD", "RedTree", "PartC", "PartD"]
    );
}

#[test]
fn not_and_has() {
    let root = build_tree();
    // :not returns all descendants except the excluded ones.
    assert_eq!(
        run(root, ":not(MeshPart)"),
        vec!["ModelA", "RedTree", "FolderB", "ModelC", "PartC", "PartD"]
    );
    // MeshA is tagged SwordPart but has no children, so no MeshPart has a
    // SwordPart-tagged direct child.
    assert_eq!(
        run(root, "MeshPart:has(> .SwordPart)"),
        Vec::<String>::new()
    );
    // ModelA contains RedTree (tagged Fruit).
    assert_eq!(run(root, "Model:has(.Fruit)"), vec!["ModelA"]);
}

#[test]
fn self_excluded() {
    let root = build_tree();
    // The receiver itself is never part of the result set.
    assert_eq!(run(root, "Folder"), vec!["FolderB"]);
}
