use crate::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReusableNode {
    pub stack: StackEntryArray,
    pub last_external_token: Subtree,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackEntryArray {
    pub contents: *mut StackEntry,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackEntry {
    pub tree: Subtree,
    pub child_index: uint32_t,
    pub byte_offset: uint32_t,
}

#[inline]
pub unsafe extern "C" fn reusable_node_new() -> ReusableNode {
    return {
        let mut init = ReusableNode {
            stack: {
                let mut init = StackEntryArray {
                    contents: 0 as *mut StackEntry,
                    size: 0 as libc::c_int as uint32_t,
                    capacity: 0 as libc::c_int as uint32_t,
                };
                init
            },
            last_external_token: Subtree {
                ptr: 0 as *const SubtreeHeapData,
            },
        };
        init
    };
}
#[inline]
pub unsafe extern "C" fn reusable_node_clear(mut self_0: *mut ReusableNode) {
    (*self_0).stack.size = 0 as libc::c_int as uint32_t;
    (*self_0).last_external_token = Subtree {
        ptr: 0 as *const SubtreeHeapData,
    };
}
#[inline]
pub unsafe extern "C" fn reusable_node_reset(mut self_0: *mut ReusableNode, mut tree: Subtree) {
    reusable_node_clear(self_0);
    array__grow(
        &mut (*self_0).stack as *mut StackEntryArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackEntry>() as libc::c_ulong,
    );
    let fresh5 = (*self_0).stack.size;
    (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
    *(*self_0).stack.contents.offset(fresh5 as isize) = {
        let mut init = StackEntry {
            tree: tree,
            child_index: 0 as libc::c_int as uint32_t,
            byte_offset: 0 as libc::c_int as uint32_t,
        };
        init
    };
}
#[inline]
pub unsafe extern "C" fn reusable_node_tree(mut self_0: *mut ReusableNode) -> Subtree {
    return if (*self_0).stack.size > 0 as libc::c_int as libc::c_uint {
        (*(*self_0).stack.contents.offset(
            (*self_0)
                .stack
                .size
                .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
        ))
        .tree
    } else {
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        }
    };
}
#[inline]
pub unsafe extern "C" fn reusable_node_byte_offset(mut self_0: *mut ReusableNode) -> uint32_t {
    return if (*self_0).stack.size > 0 as libc::c_int as libc::c_uint {
        (*(*self_0).stack.contents.offset(
            (*self_0)
                .stack
                .size
                .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
        ))
        .byte_offset
    } else {
        4294967295 as libc::c_uint
    };
}
#[inline]
pub unsafe extern "C" fn reusable_node_delete(mut self_0: *mut ReusableNode) {
    array__delete(&mut (*self_0).stack as *mut StackEntryArray as *mut VoidArray);
}
#[inline]
pub unsafe extern "C" fn reusable_node_advance(mut self_0: *mut ReusableNode) {
    assert!(
        (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            < (*self_0).stack.size
    );
    let mut last_entry: StackEntry = *(&mut *(*self_0).stack.contents.offset(
        (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
    ) as *mut StackEntry);
    let mut byte_offset: uint32_t = last_entry
        .byte_offset
        .wrapping_add(ts_subtree_total_bytes(last_entry.tree));
    if ts_subtree_has_external_tokens(last_entry.tree) {
        (*self_0).last_external_token = ts_subtree_last_external_token(last_entry.tree)
    }
    let mut tree: Subtree = Subtree {
        data: SubtreeInlineData {
            is_inline_visible_named_extra_has_changes_is_missing_is_keyword: [0; 1],
            symbol: 0,
            padding_bytes: 0,
            size_bytes: 0,
            padding_columns: 0,
            padding_rows_lookahead_bytes: [0; 1],
            parse_state: 0,
        },
    };
    let mut next_index: uint32_t = 0;
    loop {
        (*self_0).stack.size = (*self_0).stack.size.wrapping_sub(1);
        let mut popped_entry: StackEntry = *(*self_0)
            .stack
            .contents
            .offset((*self_0).stack.size as isize);
        next_index = popped_entry
            .child_index
            .wrapping_add(1 as libc::c_int as libc::c_uint);
        if (*self_0).stack.size == 0 as libc::c_int as libc::c_uint {
            return;
        }
        assert!(
            (*self_0)
                .stack
                .size
                .wrapping_sub(1 as libc::c_int as libc::c_uint)
                < (*self_0).stack.size
        );
        tree = (*(&mut *(*self_0).stack.contents.offset(
            (*self_0)
                .stack
                .size
                .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
        ) as *mut StackEntry))
            .tree;
        if !(ts_subtree_child_count(tree) <= next_index) {
            break;
        }
    }
    array__grow(
        &mut (*self_0).stack as *mut StackEntryArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackEntry>() as libc::c_ulong,
    );
    let fresh6 = (*self_0).stack.size;
    (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
    *(*self_0).stack.contents.offset(fresh6 as isize) = {
        let mut init = StackEntry {
            tree: *(*tree.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(next_index as isize),
            child_index: next_index,
            byte_offset: byte_offset,
        };
        init
    };
}
#[inline]
pub unsafe extern "C" fn reusable_node_descend(mut self_0: *mut ReusableNode) -> bool {
    assert!(
        (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            < (*self_0).stack.size
    );
    let mut last_entry: StackEntry = *(&mut *(*self_0).stack.contents.offset(
        (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
    ) as *mut StackEntry);
    if ts_subtree_child_count(last_entry.tree) > 0 as libc::c_int as libc::c_uint {
        array__grow(
            &mut (*self_0).stack as *mut StackEntryArray as *mut VoidArray,
            1 as libc::c_int as size_t,
            ::std::mem::size_of::<StackEntry>() as libc::c_ulong,
        );
        let fresh7 = (*self_0).stack.size;
        (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
        *(*self_0).stack.contents.offset(fresh7 as isize) = {
            let mut init = StackEntry {
                tree: *(*last_entry.tree.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .children
                    .offset(0 as libc::c_int as isize),
                child_index: 0 as libc::c_int as uint32_t,
                byte_offset: last_entry.byte_offset,
            };
            init
        };
        return 1 as libc::c_int != 0;
    } else {
        return 0 as libc::c_int != 0;
    };
}
#[inline]
pub unsafe extern "C" fn reusable_node_advance_past_leaf(mut self_0: *mut ReusableNode) {
    while reusable_node_descend(self_0) {}
    reusable_node_advance(self_0);
}
