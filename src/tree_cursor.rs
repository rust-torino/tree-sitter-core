use crate::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TreeCursorEntryArray {
    pub contents: *mut TreeCursorEntry,
    pub size: uint32_t,
    pub capacity: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TreeCursor {
    pub tree: *const TSTree,
    pub stack: TreeCursorEntryArray,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CursorChildIterator {
    pub parent: Subtree,
    pub tree: *const TSTree,
    pub position: Length,
    pub child_index: uint32_t,
    pub structural_child_index: uint32_t,
    pub alias_sequence: *const TSSymbol,
}

// CursorChildIterator
#[inline]
unsafe extern "C" fn ts_tree_cursor_iterate_children(
    mut self_0: *const TreeCursor,
) -> CursorChildIterator {
    if (*self_0)
        .stack
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint)
        < (*self_0).stack.size
    {
    } else {
        __assert_fail(
            b"(uint32_t)(&self->stack)->size - 1 < (&self->stack)->size\x00" as *const u8
                as *const libc::c_char,
            b"lib/src/tree_cursor.c\x00" as *const u8 as *const libc::c_char,
            19 as libc::c_int as libc::c_uint,
            (*::std::mem::transmute::<&[u8; 72], &[libc::c_char; 72]>(
                b"CursorChildIterator ts_tree_cursor_iterate_children(const TreeCursor *)\x00",
            ))
            .as_ptr(),
        );
    }
    let mut last_entry: *mut TreeCursorEntry = &mut *(*self_0).stack.contents.offset(
        (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
    ) as *mut TreeCursorEntry;
    if ts_subtree_child_count(*(*last_entry).subtree) == 0 as libc::c_int as libc::c_uint {
        return {
            let mut init = CursorChildIterator {
                parent: Subtree {
                    ptr: 0 as *const SubtreeHeapData,
                },
                tree: (*self_0).tree,
                position: length_zero(),
                child_index: 0 as libc::c_int as uint32_t,
                structural_child_index: 0 as libc::c_int as uint32_t,
                alias_sequence: 0 as *const TSSymbol,
            };
            init
        };
    }
    let mut alias_sequence: *const TSSymbol = ts_language_alias_sequence(
        (*(*self_0).tree).language,
        (*(*(*last_entry).subtree).ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .production_id as uint32_t,
    );
    return {
        let mut init = CursorChildIterator {
            parent: *(*last_entry).subtree,
            tree: (*self_0).tree,
            position: (*last_entry).position,
            child_index: 0 as libc::c_int as uint32_t,
            structural_child_index: 0 as libc::c_int as uint32_t,
            alias_sequence: alias_sequence,
        };
        init
    };
}
#[inline]
unsafe extern "C" fn ts_tree_cursor_child_iterator_next(
    mut self_0: *mut CursorChildIterator,
    mut result: *mut TreeCursorEntry,
    mut visible: *mut bool,
) -> bool {
    if (*self_0).parent.ptr.is_null()
        || (*self_0).child_index == (*(*self_0).parent.ptr).child_count
    {
        return 0 as libc::c_int != 0;
    }
    let mut child: *const Subtree = &mut *(*(*self_0).parent.ptr)
        .c2rust_unnamed
        .c2rust_unnamed
        .children
        .offset((*self_0).child_index as isize) as *mut Subtree;
    *result = {
        let mut init = TreeCursorEntry {
            subtree: child,
            position: (*self_0).position,
            child_index: (*self_0).child_index,
            structural_child_index: (*self_0).structural_child_index,
        };
        init
    };
    *visible = ts_subtree_visible(*child);
    let mut extra: bool = ts_subtree_extra(*child);
    if !extra && !(*self_0).alias_sequence.is_null() {
        *visible = (*visible as libc::c_int
            | *(*self_0)
                .alias_sequence
                .offset((*self_0).structural_child_index as isize) as libc::c_int)
            != 0;
        (*self_0).structural_child_index = (*self_0).structural_child_index.wrapping_add(1)
    }
    (*self_0).position = length_add((*self_0).position, ts_subtree_size(*child));
    (*self_0).child_index = (*self_0).child_index.wrapping_add(1);
    if (*self_0).child_index < (*(*self_0).parent.ptr).child_count {
        let mut next_child: Subtree = *(*(*self_0).parent.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset((*self_0).child_index as isize);
        (*self_0).position = length_add((*self_0).position, ts_subtree_padding(next_child))
    }
    return 1 as libc::c_int != 0;
}
// TSTreeCursor - lifecycle
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_new(mut node: TSNode) -> TSTreeCursor {
    let mut self_0: TSTreeCursor = {
        let mut init = TSTreeCursor {
            tree: 0 as *const libc::c_void,
            id: 0 as *const libc::c_void,
            context: [0 as libc::c_int as uint32_t, 0 as libc::c_int as uint32_t],
        };
        init
    };
    ts_tree_cursor_init(&mut self_0 as *mut TSTreeCursor as *mut TreeCursor, node);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_reset(mut _self: *mut TSTreeCursor, mut node: TSNode) {
    ts_tree_cursor_init(_self as *mut TreeCursor, node);
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_init(mut self_0: *mut TreeCursor, mut node: TSNode) {
    (*self_0).tree = node.tree;
    (*self_0).stack.size = 0 as libc::c_int as uint32_t;
    array__grow(
        &mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
    );
    let fresh0 = (*self_0).stack.size;
    (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
    *(*self_0).stack.contents.offset(fresh0 as isize) = {
        let mut init = TreeCursorEntry {
            subtree: node.id as *const Subtree,
            position: {
                let mut init = Length {
                    bytes: ts_node_start_byte(node),
                    extent: ts_node_start_point(node),
                };
                init
            },
            child_index: 0 as libc::c_int as uint32_t,
            structural_child_index: 0 as libc::c_int as uint32_t,
        };
        init
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_delete(mut _self: *mut TSTreeCursor) {
    let mut self_0: *mut TreeCursor = _self as *mut TreeCursor;
    array__delete(&mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray);
}
// TSTreeCursor - walking the tree
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_goto_first_child(mut _self: *mut TSTreeCursor) -> bool {
    let mut self_0: *mut TreeCursor = _self as *mut TreeCursor;
    let mut did_descend: bool = false;
    loop {
        did_descend = 0 as libc::c_int != 0;
        let mut visible: bool = false;
        let mut entry: TreeCursorEntry = TreeCursorEntry {
            subtree: 0 as *const Subtree,
            position: Length {
                bytes: 0,
                extent: TSPoint { row: 0, column: 0 },
            },
            child_index: 0,
            structural_child_index: 0,
        };
        let mut iterator: CursorChildIterator = ts_tree_cursor_iterate_children(self_0);
        while ts_tree_cursor_child_iterator_next(&mut iterator, &mut entry, &mut visible) {
            if visible {
                array__grow(
                    &mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray,
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
                );
                let fresh1 = (*self_0).stack.size;
                (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
                *(*self_0).stack.contents.offset(fresh1 as isize) = entry;
                return 1 as libc::c_int != 0;
            }
            if !(ts_subtree_visible_child_count(*entry.subtree) > 0 as libc::c_int as libc::c_uint)
            {
                continue;
            }
            array__grow(
                &mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray,
                1 as libc::c_int as size_t,
                ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
            );
            let fresh2 = (*self_0).stack.size;
            (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
            *(*self_0).stack.contents.offset(fresh2 as isize) = entry;
            did_descend = 1 as libc::c_int != 0;
            break;
        }
        if !did_descend {
            break;
        }
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_goto_first_child_for_byte(
    mut _self: *mut TSTreeCursor,
    mut goal_byte: uint32_t,
) -> int64_t {
    let mut self_0: *mut TreeCursor = _self as *mut TreeCursor;
    let mut initial_size: uint32_t = (*self_0).stack.size;
    let mut visible_child_index: uint32_t = 0 as libc::c_int as uint32_t;
    let mut did_descend: bool = false;
    loop {
        did_descend = 0 as libc::c_int != 0;
        let mut visible: bool = false;
        let mut entry: TreeCursorEntry = TreeCursorEntry {
            subtree: 0 as *const Subtree,
            position: Length {
                bytes: 0,
                extent: TSPoint { row: 0, column: 0 },
            },
            child_index: 0,
            structural_child_index: 0,
        };
        let mut iterator: CursorChildIterator = ts_tree_cursor_iterate_children(self_0);
        while ts_tree_cursor_child_iterator_next(&mut iterator, &mut entry, &mut visible) {
            let mut end_byte: uint32_t = entry
                .position
                .bytes
                .wrapping_add(ts_subtree_size(*entry.subtree).bytes);
            let mut at_goal: bool = end_byte > goal_byte;
            let mut visible_child_count: uint32_t = ts_subtree_visible_child_count(*entry.subtree);
            if at_goal {
                if visible {
                    array__grow(
                        &mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray,
                        1 as libc::c_int as size_t,
                        ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
                    );
                    let fresh3 = (*self_0).stack.size;
                    (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
                    *(*self_0).stack.contents.offset(fresh3 as isize) = entry;
                    return visible_child_index as int64_t;
                }
                if !(visible_child_count > 0 as libc::c_int as libc::c_uint) {
                    continue;
                }
                array__grow(
                    &mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray,
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
                );
                let fresh4 = (*self_0).stack.size;
                (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
                *(*self_0).stack.contents.offset(fresh4 as isize) = entry;
                did_descend = 1 as libc::c_int != 0;
                break;
            } else if visible {
                visible_child_index = visible_child_index.wrapping_add(1)
            } else {
                visible_child_index = (visible_child_index as libc::c_uint)
                    .wrapping_add(visible_child_count)
                    as uint32_t as uint32_t
            }
        }
        if !did_descend {
            break;
        }
    }
    if (*self_0).stack.size > initial_size
        && ts_tree_cursor_goto_next_sibling(self_0 as *mut TSTreeCursor) as libc::c_int != 0
    {
        return visible_child_index as int64_t;
    }
    (*self_0).stack.size = initial_size;
    return -(1 as libc::c_int) as int64_t;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_goto_next_sibling(mut _self: *mut TSTreeCursor) -> bool {
    let mut self_0: *mut TreeCursor = _self as *mut TreeCursor;
    let mut initial_size: uint32_t = (*self_0).stack.size;
    while (*self_0).stack.size > 1 as libc::c_int as libc::c_uint {
        (*self_0).stack.size = (*self_0).stack.size.wrapping_sub(1);
        let mut entry: TreeCursorEntry = *(*self_0)
            .stack
            .contents
            .offset((*self_0).stack.size as isize);
        let mut iterator: CursorChildIterator = ts_tree_cursor_iterate_children(self_0);
        iterator.child_index = entry.child_index;
        iterator.structural_child_index = entry.structural_child_index;
        iterator.position = entry.position;
        let mut visible: bool = 0 as libc::c_int != 0;
        ts_tree_cursor_child_iterator_next(&mut iterator, &mut entry, &mut visible);
        if visible as libc::c_int != 0
            && (*self_0)
                .stack
                .size
                .wrapping_add(1 as libc::c_int as libc::c_uint)
                < initial_size
        {
            break;
        }
        while ts_tree_cursor_child_iterator_next(&mut iterator, &mut entry, &mut visible) {
            if visible {
                array__grow(
                    &mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray,
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
                );
                let fresh5 = (*self_0).stack.size;
                (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
                *(*self_0).stack.contents.offset(fresh5 as isize) = entry;
                return 1 as libc::c_int != 0;
            }
            if ts_subtree_visible_child_count(*entry.subtree) != 0 {
                array__grow(
                    &mut (*self_0).stack as *mut TreeCursorEntryArray as *mut VoidArray,
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
                );
                let fresh6 = (*self_0).stack.size;
                (*self_0).stack.size = (*self_0).stack.size.wrapping_add(1);
                *(*self_0).stack.contents.offset(fresh6 as isize) = entry;
                ts_tree_cursor_goto_first_child(_self);
                return 1 as libc::c_int != 0;
            }
        }
    }
    (*self_0).stack.size = initial_size;
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_goto_parent(mut _self: *mut TSTreeCursor) -> bool {
    let mut self_0: *mut TreeCursor = _self as *mut TreeCursor;
    let mut i: libc::c_uint = (*self_0)
        .stack
        .size
        .wrapping_sub(2 as libc::c_int as libc::c_uint);
    while i.wrapping_add(1 as libc::c_int as libc::c_uint) > 0 as libc::c_int as libc::c_uint {
        let mut entry: *mut TreeCursorEntry =
            &mut *(*self_0).stack.contents.offset(i as isize) as *mut TreeCursorEntry;
        let mut is_aliased: bool = 0 as libc::c_int != 0;
        if i > 0 as libc::c_int as libc::c_uint {
            let mut parent_entry: *mut TreeCursorEntry = &mut *(*self_0)
                .stack
                .contents
                .offset(i.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)
                as *mut TreeCursorEntry;
            let mut alias_sequence: *const TSSymbol = ts_language_alias_sequence(
                (*(*self_0).tree).language,
                (*(*(*parent_entry).subtree).ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .production_id as uint32_t,
            );
            is_aliased = !alias_sequence.is_null()
                && *alias_sequence.offset((*entry).structural_child_index as isize) as libc::c_int
                    != 0
        }
        if ts_subtree_visible(*(*entry).subtree) as libc::c_int != 0
            || is_aliased as libc::c_int != 0
        {
            (*self_0).stack.size = i.wrapping_add(1 as libc::c_int as libc::c_uint);
            return 1 as libc::c_int != 0;
        }
        i = i.wrapping_sub(1)
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_current_node(mut _self: *const TSTreeCursor) -> TSNode {
    let mut self_0: *const TreeCursor = _self as *const TreeCursor;
    if (*self_0)
        .stack
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint)
        < (*self_0).stack.size
    {
    } else {
        __assert_fail(
            b"(uint32_t)(&self->stack)->size - 1 < (&self->stack)->size\x00" as *const u8
                as *const libc::c_char,
            b"lib/src/tree_cursor.c\x00" as *const u8 as *const libc::c_char,
            227 as libc::c_int as libc::c_uint,
            (*::std::mem::transmute::<&[u8; 57], &[libc::c_char; 57]>(
                b"TSNode ts_tree_cursor_current_node(const TSTreeCursor *)\x00",
            ))
            .as_ptr(),
        );
    }
    let mut last_entry: *mut TreeCursorEntry = &mut *(*self_0).stack.contents.offset(
        (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
    ) as *mut TreeCursorEntry;
    let mut alias_symbol: TSSymbol = 0 as libc::c_int as TSSymbol;
    if (*self_0).stack.size > 1 as libc::c_int as libc::c_uint {
        let mut parent_entry: *mut TreeCursorEntry = &mut *(*self_0).stack.contents.offset(
            (*self_0)
                .stack
                .size
                .wrapping_sub(2 as libc::c_int as libc::c_uint) as isize,
        ) as *mut TreeCursorEntry;
        let mut alias_sequence: *const TSSymbol = ts_language_alias_sequence(
            (*(*self_0).tree).language,
            (*(*(*parent_entry).subtree).ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .production_id as uint32_t,
        );
        if !alias_sequence.is_null() && !ts_subtree_extra(*(*last_entry).subtree) {
            alias_symbol = *alias_sequence.offset((*last_entry).structural_child_index as isize)
        }
    }
    return ts_node_new(
        (*self_0).tree,
        (*last_entry).subtree,
        (*last_entry).position,
        alias_symbol,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_current_status(
    mut _self: *const TSTreeCursor,
    mut can_have_later_siblings: *mut bool,
    mut can_have_later_siblings_with_this_field: *mut bool,
) -> TSFieldId {
    let mut self_0: *const TreeCursor = _self as *const TreeCursor;
    let mut result: TSFieldId = 0 as libc::c_int as TSFieldId;
    *can_have_later_siblings = 0 as libc::c_int != 0;
    *can_have_later_siblings_with_this_field = 0 as libc::c_int != 0;
    // Walk up the tree, visiting the current node and its invisible ancestors,
    // because fields can refer to nodes through invisible *wrapper* nodes,
    let mut i: libc::c_uint = (*self_0)
        .stack
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint);
    while i > 0 as libc::c_int as libc::c_uint {
        let mut entry: *mut TreeCursorEntry =
            &mut *(*self_0).stack.contents.offset(i as isize) as *mut TreeCursorEntry;
        let mut parent_entry: *mut TreeCursorEntry = &mut *(*self_0)
            .stack
            .contents
            .offset(i.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)
            as *mut TreeCursorEntry;
        // Stop walking up when a visible ancestor is found.
        if i != (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
        {
            if ts_subtree_visible(*(*entry).subtree) {
                break;
            }
            let mut alias_sequence: *const TSSymbol = ts_language_alias_sequence(
                (*(*self_0).tree).language,
                (*(*(*parent_entry).subtree).ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .production_id as uint32_t,
            );
            if !alias_sequence.is_null()
                && *alias_sequence.offset((*entry).structural_child_index as isize) as libc::c_int
                    != 0
            {
                break;
            }
        }
        if ts_subtree_child_count(*(*parent_entry).subtree)
            > (*entry)
                .child_index
                .wrapping_add(1 as libc::c_int as libc::c_uint)
        {
            *can_have_later_siblings = 1 as libc::c_int != 0
        }
        if ts_subtree_extra(*(*entry).subtree) {
            break;
        }
        let mut field_map: *const TSFieldMapEntry = 0 as *const TSFieldMapEntry;
        let mut field_map_end: *const TSFieldMapEntry = 0 as *const TSFieldMapEntry;
        ts_language_field_map(
            (*(*self_0).tree).language,
            (*(*(*parent_entry).subtree).ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .production_id as uint32_t,
            &mut field_map,
            &mut field_map_end,
        );
        // Look for a field name associated with the current node.
        if result == 0 {
            let mut i_0: *const TSFieldMapEntry = field_map;
            while i_0 < field_map_end {
                if !(*i_0).inherited
                    && (*i_0).child_index as libc::c_uint == (*entry).structural_child_index
                {
                    result = (*i_0).field_id;
                    *can_have_later_siblings_with_this_field = 0 as libc::c_int != 0;
                    break;
                } else {
                    i_0 = i_0.offset(1)
                }
            }
        }
        // Determine if there other later siblings with the same field name.
        if result != 0 {
            let mut i_1: *const TSFieldMapEntry = field_map;
            while i_1 < field_map_end {
                if (*i_1).field_id as libc::c_int == result as libc::c_int
                    && (*i_1).child_index as libc::c_uint > (*entry).structural_child_index
                {
                    *can_have_later_siblings_with_this_field = 1 as libc::c_int != 0;
                    break;
                } else {
                    i_1 = i_1.offset(1)
                }
            }
        }
        i = i.wrapping_sub(1)
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_current_field_id(
    mut _self: *const TSTreeCursor,
) -> TSFieldId {
    let mut self_0: *const TreeCursor = _self as *const TreeCursor;
    // Walk up the tree, visiting the current node and its invisible ancestors.
    let mut i: libc::c_uint = (*self_0)
        .stack
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint);
    while i > 0 as libc::c_int as libc::c_uint {
        let mut entry: *mut TreeCursorEntry =
            &mut *(*self_0).stack.contents.offset(i as isize) as *mut TreeCursorEntry;
        let mut parent_entry: *mut TreeCursorEntry = &mut *(*self_0)
            .stack
            .contents
            .offset(i.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)
            as *mut TreeCursorEntry;
        // Stop walking up when another visible node is found.
        if i != (*self_0)
            .stack
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
        {
            if ts_subtree_visible(*(*entry).subtree) {
                break;
            }
            let mut alias_sequence: *const TSSymbol = ts_language_alias_sequence(
                (*(*self_0).tree).language,
                (*(*(*parent_entry).subtree).ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .production_id as uint32_t,
            );
            if !alias_sequence.is_null()
                && *alias_sequence.offset((*entry).structural_child_index as isize) as libc::c_int
                    != 0
            {
                break;
            }
        }
        if ts_subtree_extra(*(*entry).subtree) {
            break;
        }
        let mut field_map: *const TSFieldMapEntry = 0 as *const TSFieldMapEntry;
        let mut field_map_end: *const TSFieldMapEntry = 0 as *const TSFieldMapEntry;
        ts_language_field_map(
            (*(*self_0).tree).language,
            (*(*(*parent_entry).subtree).ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .production_id as uint32_t,
            &mut field_map,
            &mut field_map_end,
        );
        let mut i_0: *const TSFieldMapEntry = field_map;
        while i_0 < field_map_end {
            if !(*i_0).inherited
                && (*i_0).child_index as libc::c_uint == (*entry).structural_child_index
            {
                return (*i_0).field_id;
            }
            i_0 = i_0.offset(1)
        }
        i = i.wrapping_sub(1)
    }
    return 0 as libc::c_int as TSFieldId;
}
#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_current_field_name(
    mut _self: *const TSTreeCursor,
) -> *const libc::c_char {
    let mut id: TSFieldId = ts_tree_cursor_current_field_id(_self);
    if id != 0 {
        let mut self_0: *const TreeCursor = _self as *const TreeCursor;
        return *(*(*(*self_0).tree).language)
            .field_names
            .offset(id as isize);
    } else {
        return 0 as *const libc::c_char;
    };
}

#[no_mangle]
pub unsafe extern "C" fn ts_tree_cursor_copy(mut _cursor: *const TSTreeCursor) -> TSTreeCursor {
    let mut cursor: *const TreeCursor = _cursor as *const TreeCursor;
    let mut res: TSTreeCursor = {
        let mut init = TSTreeCursor {
            tree: 0 as *const libc::c_void,
            id: 0 as *const libc::c_void,
            context: [0 as libc::c_int as uint32_t, 0 as libc::c_int as uint32_t],
        };
        init
    };
    let mut copy: *mut TreeCursor = &mut res as *mut TSTreeCursor as *mut TreeCursor;
    (*copy).tree = (*cursor).tree;
    array__splice(
        &mut (*copy).stack as *mut TreeCursorEntryArray as *mut VoidArray,
        ::std::mem::size_of::<TreeCursorEntry>() as libc::c_ulong,
        (*copy).stack.size,
        0 as libc::c_int as uint32_t,
        (*cursor).stack.size,
        (*cursor).stack.contents as *const libc::c_void,
    );
    return res;
}
