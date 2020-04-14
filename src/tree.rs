use crate::*;

use std::{cell::RefCell, convert::TryInto, ptr};

const PARENT_CACHE_CAPACITY: usize = 32;

#[derive(Clone)]
pub struct ParentCache {
    pub cache: Vec<ParentCacheEntry>,
    pub start: u32,
}

#[derive(Clone)]
#[repr(C)]
pub struct TSTree<'lang> {
    pub root: Subtree,
    pub language: &'lang TSLanguage,
    pub parent_cache: RefCell<ParentCache>,
    pub included_ranges: Box<[TSRange]>,
}

#[no_mangle]
pub(crate) extern "C" fn ts_tree_new<'lang>(
    root: Subtree,
    language: &'lang TSLanguage,
    included_ranges: &[TSRange],
) -> Box<TSTree<'lang>> {
    let included_ranges = included_ranges.iter().cloned().collect();
    Box::new(TSTree {
        root,
        language,
        parent_cache: RefCell::new(ParentCache {
            cache: Vec::new(),
            start: 0,
        }),
        included_ranges,
    })
}

/// Create a shallow copy of the syntax tree. This is very fast.
///
/// You need to copy a syntax tree in order to use it on more than one thread at
/// a time, as syntax trees are not thread safe.
#[no_mangle]
pub unsafe extern "C" fn ts_tree_copy<'a>(tree: &'a TSTree) -> Box<TSTree<'a>> {
    ts_subtree_retain(tree.root);
    ts_tree_new(tree.root, tree.language, &tree.included_ranges)
}

/// Delete the syntax tree, freeing all of the memory that it used.
#[no_mangle]
pub unsafe extern "C" fn ts_tree_delete(tree: Option<Box<TSTree>>) {
    if let Some(tree) = tree {
        let mut pool = ts_subtree_pool_new(0);
        ts_subtree_release(&mut pool, tree.root);
        ts_subtree_pool_delete(&mut pool);
    }
}

/// Get the root node of the syntax tree.
#[no_mangle]
pub unsafe extern "C" fn ts_tree_root_node<'lang>(tree: &TSTree<'lang>) -> TSNode<'lang> {
    ts_node_new(
        tree as *const TSTree,
        &tree.root,
        ts_subtree_padding(tree.root),
        0 as TSSymbol,
    )
}

/// Get the language that was used to parse the syntax tree.
#[no_mangle]
pub extern "C" fn ts_tree_language(tree: &TSTree) -> *const TSLanguage {
    tree.language
}

/// Edit the syntax tree to keep it in sync with source code that has been
/// edited.
///
/// You must describe the edit both in terms of byte offsets and in terms of
/// (row, column) coordinates.
#[no_mangle]
pub unsafe extern "C" fn ts_tree_edit(tree: &mut TSTree, edit: &TSInputEdit) {
    for range in tree.included_ranges.iter_mut() {
        if range.end_byte >= edit.old_end_byte {
            if range.end_byte != std::u32::MAX {
                range.end_byte = edit.new_end_byte + range.end_byte + edit.old_end_byte;
                range.end_point = point_add(
                    edit.new_end_point,
                    point_sub(range.end_point, edit.old_end_point),
                );
                if range.end_byte < edit.new_end_byte {
                    range.end_byte = std::u32::MAX;
                    range.end_point = TSPoint {
                        row: std::u32::MAX,
                        column: std::u32::MAX,
                    }
                }
            }
            if range.start_byte >= edit.old_end_byte {
                range.start_byte = edit.new_end_byte + range.start_byte + edit.old_end_byte;
                range.start_point = point_add(
                    edit.new_end_point,
                    point_sub(range.start_point, edit.old_end_point),
                );
                if range.start_byte < edit.new_end_byte {
                    range.start_byte = std::u32::MAX;
                    range.start_point = TSPoint {
                        row: std::u32::MAX,
                        column: std::u32::MAX,
                    }
                }
            }
        }
    }
    let mut pool: SubtreePool = ts_subtree_pool_new(0);
    tree.root = ts_subtree_edit(tree.root, edit, &mut pool);
    tree.parent_cache.borrow_mut().start = 0;
    ts_subtree_pool_delete(&mut pool);
}

/// Compare an old edited syntax tree to a new syntax tree representing the same
/// document, returning an array of ranges whose syntactic structure has changed.
///
/// For this to work correctly, the old syntax tree must have been edited such
/// that its ranges match up to the new tree. Generally, you'll want to call
/// this function right after calling one of the `ts_parser_parse` functions.
/// You need to pass the old tree that was passed to parse, as well as the new
/// tree that was returned from that function.
///
/// The returned array is allocated using `malloc` and the caller is responsible
/// for freeing it using `free`. The length of the array will be written to the
/// given `length` pointer.
#[no_mangle]
pub unsafe extern "C" fn ts_tree_get_changed_ranges(
    self_0: &TSTree,
    other: &TSTree,
    count: &mut u32,
) -> *mut TSRange {
    let mut cursor1 = TreeCursor {
        tree: ptr::null(),
        stack: TreeCursorEntryArray {
            contents: ptr::null_mut(),
            size: 0,
            capacity: 0,
        },
    };
    let mut cursor2 = TreeCursor {
        tree: ptr::null(),
        stack: TreeCursorEntryArray {
            contents: ptr::null_mut(),
            size: 0,
            capacity: 0,
        },
    };
    ts_tree_cursor_init(&mut cursor1, ts_tree_root_node(self_0));
    ts_tree_cursor_init(&mut cursor2, ts_tree_root_node(other));
    let mut included_range_differences = TSRangeArray {
        contents: ptr::null_mut(),
        size: 0,
        capacity: 0,
    };
    ts_range_array_get_changed_ranges(
        self_0.included_ranges.as_ptr(),
        self_0.included_ranges.len().try_into().unwrap(),
        other.included_ranges.as_ptr(),
        other.included_ranges.len().try_into().unwrap(),
        &mut included_range_differences,
    );
    let mut result: *mut TSRange = ptr::null_mut::<TSRange>();
    *count = ts_subtree_get_changed_ranges(
        &(*self_0).root,
        &(*other).root,
        &mut cursor1,
        &mut cursor2,
        (*self_0).language,
        &included_range_differences,
        &mut result,
    );
    array__delete(&mut included_range_differences as *mut TSRangeArray as *mut VoidArray);
    array__delete(&mut cursor1.stack as *mut TreeCursorEntryArray as *mut VoidArray);
    array__delete(&mut cursor2.stack as *mut TreeCursorEntryArray as *mut VoidArray);
    result
}

#[no_mangle]
pub(crate) unsafe extern "C" fn ts_tree_get_cached_parent<'lang>(
    tree: &TSTree<'lang>,
    node: &TSNode<'lang>,
) -> TSNode<'lang> {
    let &ParentCache { ref cache, start } = &*tree.parent_cache.borrow();
    cache
        .iter()
        .cycle()
        .skip(start.try_into().unwrap())
        .take(PARENT_CACHE_CAPACITY)
        .find(|entry| entry.child == node.id as *const Subtree)
        .map(|entry| ts_node_new(tree, entry.parent, entry.position, entry.alias_symbol))
        .unwrap_or_else(|| ts_node_new(ptr::null(), ptr::null(), length_zero(), 0 as TSSymbol))
}

#[no_mangle]
pub(crate) unsafe extern "C" fn ts_tree_set_cached_parent(
    tree: &TSTree,
    node: &TSNode,
    parent: &TSNode,
) {
    let mut parent_cache = tree.parent_cache.borrow_mut();
    if parent_cache.cache.capacity() == 0 {
        parent_cache.cache.reserve_exact(PARENT_CACHE_CAPACITY);
    }

    let new_entry = ParentCacheEntry {
        child: node.id as *const Subtree,
        parent: parent.id as *const Subtree,
        position: Length {
            bytes: parent.context[0],
            extent: TSPoint {
                row: parent.context[1],
                column: parent.context[2],
            },
        },
        alias_symbol: parent.context[3] as TSSymbol,
    };

    if parent_cache.cache.len() < PARENT_CACHE_CAPACITY {
        parent_cache.cache.push(new_entry);
    } else {
        let start: usize = parent_cache.start.try_into().unwrap();
        parent_cache.cache[start % PARENT_CACHE_CAPACITY] = new_entry;
        parent_cache.start += 1
    }
}
