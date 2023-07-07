use rbx_dom_weak::{
    types::{Ref as DomRef, VariantType as DomType},
    Instance as DomInstance, WeakDom,
};

use crate::shared::instance::class_is_a;

pub fn postprocess_dom_for_place(_dom: &mut WeakDom) {
    // Nothing here yet
}

pub fn postprocess_dom_for_model(dom: &mut WeakDom) {
    let root_ref = dom.root_ref();
    recurse_instances(dom, root_ref, |inst| {
        // Get rid of some unique ids - roblox does not
        // save these in model files, and we shouldn't either
        remove_matching_prop(inst, DomType::UniqueId, "UniqueId");
        remove_matching_prop(inst, DomType::UniqueId, "HistoryId");
        // Similar story with ScriptGuid - this is used
        // in the studio-only cloud script drafts feature
        if class_is_a(&inst.class, "LuaSourceContainer").unwrap_or(false) {
            inst.properties.remove("ScriptGuid");
        }
    });
}

fn recurse_instances<F>(dom: &mut WeakDom, dom_ref: DomRef, f: F)
where
    F: Fn(&mut DomInstance),
{
    let child_refs = match dom.get_by_ref_mut(dom_ref) {
        Some(inst) => {
            f(inst);
            inst.children().to_vec()
        }
        None => Vec::new(),
    };
    for child_ref in child_refs {
        recurse_instances(dom, child_ref, &f);
    }
}

fn remove_matching_prop(inst: &mut DomInstance, ty: DomType, name: &'static str) {
    if inst.properties.get(name).map_or(false, |u| u.ty() == ty) {
        inst.properties.remove(name);
    }
}
