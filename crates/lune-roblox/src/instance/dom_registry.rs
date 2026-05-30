/*!
    Global registry of weak doms that instances live in.

    Historically all instances lived in a single global "internal dom", and
    instances were *transferred* into it on parse and *cloned* out on save.
    That model made it impossible to tell two instances from different place
    files apart when their referents collided (see issue #356), and it never
    freed memory since orphaned instances lived at the root forever.

    Instead, every parsed document and the shared scratch dom used for manually
    created orphans is registered here under a unique [`DomId`]. An [`Instance`]
    is identified by the pair `(dom_id, dom_ref)` and the `dom_id` disambiguates
    referents that happen to collide between separate doms.
*/

use std::{
    collections::{HashMap, VecDeque},
    sync::{LazyLock, Mutex, MutexGuard},
};

use rbx_dom_weak::{InstanceBuilder as DomInstanceBuilder, WeakDom, types::Ref as DomRef};

pub type DomId = u64;

struct Registry {
    doms: HashMap<DomId, WeakDom>,
    next_id: DomId,
    default_id: Option<DomId>,
}

static REGISTRY: LazyLock<Mutex<Registry>> = LazyLock::new(|| {
    Mutex::new(Registry {
        doms: HashMap::new(),
        next_id: 1,
        default_id: None,
    })
});

fn lock() -> MutexGuard<'static, Registry> {
    REGISTRY
        .lock()
        .expect("Failed to lock instance dom registry")
}

/**
    Registers a new dom and returns its freshly allocated id.
*/
pub fn register(dom: WeakDom) -> DomId {
    let mut reg = lock();

    let id = reg.next_id;

    reg.next_id += 1;
    reg.doms.insert(id, dom);

    id
}

/**
    Returns the id of the shared default scratch dom, creating it if needed.

    Manually created orphan instances (e.g. `Instance.new(...)`) should live
    here until they are parented into another dom.
*/
pub fn default_dom() -> DomId {
    let mut reg = lock();

    if let Some(id) = reg.default_id
        && reg.doms.contains_key(&id)
    {
        return id;
    }

    let id = reg.next_id;

    reg.next_id += 1;
    reg.doms
        .insert(id, WeakDom::new(DomInstanceBuilder::new("ROOT")));
    reg.default_id = Some(id);

    id
}

/**
    Runs a closure with shared access to the dom with the given id.

    Returns `None` if no such dom exists (e.g. it was already torn down).

    **NOTE:** The closure must not re-enter the registry (no nested
    [`with`]/[`with_mut`] calls), as the registry lock is held for its duration.
    Collect referents inside the closure and build [`Instance`]s from the
    `&WeakDom` already in hand instead.
*/
pub fn with<R>(dom_id: DomId, f: impl FnOnce(&WeakDom) -> R) -> Option<R> {
    let reg = lock();
    reg.doms.get(&dom_id).map(f)
}

/**
    Runs a closure with mutable access to the dom with the given id.

    Returns `None` if no such dom exists. Same re-entrancy rules as [`with`].
*/
pub fn with_mut<R>(dom_id: DomId, f: impl FnOnce(&mut WeakDom) -> R) -> Option<R> {
    let mut reg = lock();
    reg.doms.get_mut(&dom_id).map(f)
}

fn collect_subtree(dom: &WeakDom, root: DomRef) -> Vec<DomRef> {
    let mut out = Vec::new();
    let mut queue = VecDeque::from([root]);

    while let Some(referent) = queue.pop_front() {
        out.push(referent);
        if let Some(inst) = dom.get_by_ref(referent) {
            queue.extend(inst.children().iter().copied());
        }
    }

    out
}

/**
    Reparents `child_ref` (currently in `child_dom`) under an optional parent.

    - `parent` is `None`: the instance is orphaned within its current dom (moved
      to that dom's root).
    - `parent` is in the same dom: a cheap `transfer_within`.
    - `parent` is in a different dom: the instance and all of its descendants
      are transferred into the parent's dom. The list of moved referents is
      returned so that callers can re-key any cached userdata; for same-dom
      moves an empty vec is returned.
*/
pub fn reparent(
    child_dom: DomId,
    child_ref: DomRef,
    parent: Option<(DomId, DomRef)>,
) -> Vec<DomRef> {
    let mut reg = lock();
    match parent {
        None => {
            if let Some(dom) = reg.doms.get_mut(&child_dom) {
                let dom_root = dom.root_ref();
                dom.transfer_within(child_ref, dom_root);
            }
            Vec::new()
        }
        Some((parent_dom, parent_ref)) if parent_dom == child_dom => {
            if let Some(dom) = reg.doms.get_mut(&child_dom) {
                dom.transfer_within(child_ref, parent_ref);
            }
            Vec::new()
        }
        Some((parent_dom, parent_ref)) => {
            let moved = match reg.doms.get(&child_dom) {
                Some(dom) => collect_subtree(dom, child_ref),
                None => return Vec::new(),
            };
            // Since we want to use get_disjoint_mut we cannot store doms in a BTreeMap. Unfortunate
            let [src, dst] = reg.doms.get_disjoint_mut([&child_dom, &parent_dom]);
            if let (Some(src), Some(dst)) = (src, dst) {
                src.transfer(child_ref, dst, parent_ref);
                moved
            } else {
                Vec::new()
            }
        }
    }
}

/**
    Drops the dom with the given id if its root instance has no children.

    Called after destroying an instance so that tearing down the root of a dom
    (e.g. a parsed `DataModel`) frees the entire dom and bounds memory use.
*/
pub fn drop_if_empty(dom_id: DomId) {
    let mut reg = lock();

    let empty = reg.doms.get(&dom_id).is_none_or(|dom| {
        dom.get_by_ref(dom.root_ref())
            .is_none_or(|root| root.children().is_empty())
    });

    if empty {
        reg.doms.remove(&dom_id);
        if reg.default_id == Some(dom_id) {
            reg.default_id = None;
        }
    }
}
