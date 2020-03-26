use crate::*;

use libc::{memcpy, FILE};

static mut PARENT_CACHE_CAPACITY: libc::c_uint = 32 as libc::c_int as libc::c_uint;

#[no_mangle]
pub unsafe extern "C" fn ts_tree_new(
    mut root: Subtree,
    mut language: *const TSLanguage,
    mut included_ranges: *const TSRange,
    mut included_range_count: libc::c_uint,
) -> *mut TSTree {
    let mut result: *mut TSTree =
        ts_malloc(::std::mem::size_of::<TSTree>() as libc::c_ulong) as *mut TSTree;
    (*result).root = root;
    (*result).language = language;
    (*result).parent_cache = 0 as *mut ParentCacheEntry;
    (*result).parent_cache_start = 0 as libc::c_int as uint32_t;
    (*result).parent_cache_size = 0 as libc::c_int as uint32_t;
    (*result).included_ranges = ts_calloc(
        included_range_count as size_t,
        ::std::mem::size_of::<TSRange>() as libc::c_ulong,
    ) as *mut TSRange;
    memcpy(
        (*result).included_ranges as *mut libc::c_void,
        included_ranges as *const libc::c_void,
        (included_range_count as libc::size_t)
            .wrapping_mul(::std::mem::size_of::<TSRange>() as libc::size_t),
    );
    (*result).included_range_count = included_range_count;
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_copy(mut self_0: *const TSTree) -> *mut TSTree {
    ts_subtree_retain((*self_0).root);
    return ts_tree_new(
        (*self_0).root,
        (*self_0).language,
        (*self_0).included_ranges,
        (*self_0).included_range_count,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_delete(mut self_0: *mut TSTree) {
    if self_0.is_null() {
        return;
    }
    let mut pool: SubtreePool = ts_subtree_pool_new(0 as libc::c_int as uint32_t);
    ts_subtree_release(&mut pool, (*self_0).root);
    ts_subtree_pool_delete(&mut pool);
    ts_free((*self_0).included_ranges as *mut libc::c_void);
    if !(*self_0).parent_cache.is_null() {
        ts_free((*self_0).parent_cache as *mut libc::c_void);
    }
    ts_free(self_0 as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_root_node(mut self_0: *const TSTree) -> TSNode {
    return ts_node_new(
        self_0,
        &(*self_0).root,
        ts_subtree_padding((*self_0).root),
        0 as libc::c_int as TSSymbol,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_language(mut self_0: *const TSTree) -> *const TSLanguage {
    return (*self_0).language;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_edit(mut self_0: *mut TSTree, mut edit: *const TSInputEdit) {
    let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    while i < (*self_0).included_range_count {
        let mut range: *mut TSRange =
            &mut *(*self_0).included_ranges.offset(i as isize) as *mut TSRange;
        if (*range).end_byte >= (*edit).old_end_byte {
            if (*range).end_byte != 4294967295 as libc::c_uint {
                (*range).end_byte = (*edit)
                    .new_end_byte
                    .wrapping_add((*range).end_byte.wrapping_sub((*edit).old_end_byte));
                (*range).end_point = point_add(
                    (*edit).new_end_point,
                    point_sub((*range).end_point, (*edit).old_end_point),
                );
                if (*range).end_byte < (*edit).new_end_byte {
                    (*range).end_byte = 4294967295 as libc::c_uint;
                    (*range).end_point = {
                        let mut init = TSPoint {
                            row: 4294967295 as libc::c_uint,
                            column: 4294967295 as libc::c_uint,
                        };
                        init
                    }
                }
            }
            if (*range).start_byte >= (*edit).old_end_byte {
                (*range).start_byte = (*edit)
                    .new_end_byte
                    .wrapping_add((*range).start_byte.wrapping_sub((*edit).old_end_byte));
                (*range).start_point = point_add(
                    (*edit).new_end_point,
                    point_sub((*range).start_point, (*edit).old_end_point),
                );
                if (*range).start_byte < (*edit).new_end_byte {
                    (*range).start_byte = 4294967295 as libc::c_uint;
                    (*range).start_point = {
                        let mut init = TSPoint {
                            row: 4294967295 as libc::c_uint,
                            column: 4294967295 as libc::c_uint,
                        };
                        init
                    }
                }
            }
        }
        i = i.wrapping_add(1)
    }
    let mut pool: SubtreePool = ts_subtree_pool_new(0 as libc::c_int as uint32_t);
    (*self_0).root = ts_subtree_edit((*self_0).root, edit, &mut pool);
    (*self_0).parent_cache_start = 0 as libc::c_int as uint32_t;
    (*self_0).parent_cache_size = 0 as libc::c_int as uint32_t;
    ts_subtree_pool_delete(&mut pool);
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_get_changed_ranges(
    mut self_0: *const TSTree,
    mut other: *const TSTree,
    mut count: *mut uint32_t,
) -> *mut TSRange {
    let mut cursor1: TreeCursor = {
        let mut init = TreeCursor {
            tree: 0 as *const TSTree,
            stack: {
                let mut init = TreeCursorEntryArray {
                    contents: 0 as *mut TreeCursorEntry,
                    size: 0 as libc::c_int as uint32_t,
                    capacity: 0 as libc::c_int as uint32_t,
                };
                init
            },
        };
        init
    };
    let mut cursor2: TreeCursor = {
        let mut init = TreeCursor {
            tree: 0 as *const TSTree,
            stack: {
                let mut init = TreeCursorEntryArray {
                    contents: 0 as *mut TreeCursorEntry,
                    size: 0 as libc::c_int as uint32_t,
                    capacity: 0 as libc::c_int as uint32_t,
                };
                init
            },
        };
        init
    };
    ts_tree_cursor_init(&mut cursor1, ts_tree_root_node(self_0));
    ts_tree_cursor_init(&mut cursor2, ts_tree_root_node(other));
    let mut included_range_differences: TSRangeArray = {
        let mut init = TSRangeArray {
            contents: 0 as *mut TSRange,
            size: 0 as libc::c_int as uint32_t,
            capacity: 0 as libc::c_int as uint32_t,
        };
        init
    };
    ts_range_array_get_changed_ranges(
        (*self_0).included_ranges,
        (*self_0).included_range_count,
        (*other).included_ranges,
        (*other).included_range_count,
        &mut included_range_differences,
    );
    let mut result: *mut TSRange = 0 as *mut TSRange;
    *count = ts_subtree_get_changed_ranges(
        &(*self_0).root,
        &(*other).root,
        &mut cursor1,
        &mut cursor2,
        (*self_0).language,
        &mut included_range_differences,
        &mut result,
    );
    array__delete(&mut included_range_differences as *mut TSRangeArray as *mut VoidArray);
    array__delete(&mut cursor1.stack as *mut TreeCursorEntryArray as *mut VoidArray);
    array__delete(&mut cursor2.stack as *mut TreeCursorEntryArray as *mut VoidArray);
    return result;
}

#[no_mangle]
pub unsafe extern "C" fn ts_tree_print_dot_graph(mut self_0: *const TSTree, mut file: *mut FILE) {
    ts_subtree_print_dot_graph((*self_0).root, (*self_0).language, file);
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_get_cached_parent(
    mut self_0: *const TSTree,
    mut node: *const TSNode,
) -> TSNode {
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*self_0).parent_cache_size {
        let mut index: uint32_t = (*self_0)
            .parent_cache_start
            .wrapping_add(i)
            .wrapping_rem(PARENT_CACHE_CAPACITY);
        let mut entry: *mut ParentCacheEntry =
            &mut *(*self_0).parent_cache.offset(index as isize) as *mut ParentCacheEntry;
        if (*entry).child == (*node).id as *const Subtree {
            return ts_node_new(
                self_0,
                (*entry).parent,
                (*entry).position,
                (*entry).alias_symbol,
            );
        }
        i = i.wrapping_add(1)
    }
    return ts_node_new(
        0 as *const TSTree,
        0 as *const Subtree,
        length_zero(),
        0 as libc::c_int as TSSymbol,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_set_cached_parent(
    mut _self: *const TSTree,
    mut node: *const TSNode,
    mut parent: *const TSNode,
) {
    let mut self_0: *mut TSTree = _self as *mut TSTree;
    if (*self_0).parent_cache.is_null() {
        (*self_0).parent_cache = ts_calloc(
            PARENT_CACHE_CAPACITY as size_t,
            ::std::mem::size_of::<ParentCacheEntry>() as libc::c_ulong,
        ) as *mut ParentCacheEntry
    }
    let mut index: uint32_t = (*self_0)
        .parent_cache_start
        .wrapping_add((*self_0).parent_cache_size)
        .wrapping_rem(PARENT_CACHE_CAPACITY);
    *(*self_0).parent_cache.offset(index as isize) = {
        let mut init = ParentCacheEntry {
            child: (*node).id as *const Subtree,
            parent: (*parent).id as *const Subtree,
            position: {
                let mut init = Length {
                    bytes: (*parent).context[0 as libc::c_int as usize],
                    extent: {
                        let mut init = TSPoint {
                            row: (*parent).context[1 as libc::c_int as usize],
                            column: (*parent).context[2 as libc::c_int as usize],
                        };
                        init
                    },
                };
                init
            },
            alias_symbol: (*parent).context[3 as libc::c_int as usize] as TSSymbol,
        };
        init
    };
    if (*self_0).parent_cache_size == PARENT_CACHE_CAPACITY {
        (*self_0).parent_cache_start = (*self_0).parent_cache_start.wrapping_add(1)
    } else {
        (*self_0).parent_cache_size = (*self_0).parent_cache_size.wrapping_add(1)
    };
}
