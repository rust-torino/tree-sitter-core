use crate::{util::WrappingOffsetFromExt, *};

use libc::{fprintf, fputc, fputs, malloc, memcmp, memcpy, snprintf, FILE};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Edit {
    pub start: Length,
    pub old_end: Length,
    pub new_end: Length,
}

static mut empty_state: ExternalScannerState = {
    let mut init = ExternalScannerState {
        c2rust_unnamed: ExternalScannerStateData {
            short_data: [
                0 as libc::c_int as libc::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        length: 0 as libc::c_int as uint32_t,
    };
    init
};

// ExternalScannerState
#[no_mangle]
pub unsafe extern "C" fn ts_external_scanner_state_init(
    mut self_0: *mut ExternalScannerState,
    mut data: *const libc::c_char,
    mut length: libc::c_uint,
) {
    (*self_0).length = length;
    if length as libc::c_ulong > ::std::mem::size_of::<[libc::c_char; 24]>() as libc::c_ulong {
        (*self_0).c2rust_unnamed.long_data = ts_malloc(length as size_t) as *mut libc::c_char;
        memcpy(
            (*self_0).c2rust_unnamed.long_data as *mut libc::c_void,
            data as *const libc::c_void,
            length as usize,
        );
    } else {
        memcpy(
            (*self_0).c2rust_unnamed.short_data.as_mut_ptr() as *mut libc::c_void,
            data as *const libc::c_void,
            length as usize,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_external_scanner_state_copy(
    mut self_0: *const ExternalScannerState,
) -> ExternalScannerState {
    let mut result: ExternalScannerState = *self_0;
    if (*self_0).length as libc::c_ulong
        > ::std::mem::size_of::<[libc::c_char; 24]>() as libc::c_ulong
    {
        result.c2rust_unnamed.long_data =
            ts_malloc((*self_0).length as size_t) as *mut libc::c_char;
        memcpy(
            result.c2rust_unnamed.long_data as *mut libc::c_void,
            (*self_0).c2rust_unnamed.long_data as *const libc::c_void,
            (*self_0).length as usize,
        );
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_external_scanner_state_delete(mut self_0: *mut ExternalScannerState) {
    if (*self_0).length as libc::c_ulong
        > ::std::mem::size_of::<[libc::c_char; 24]>() as libc::c_ulong
    {
        ts_free((*self_0).c2rust_unnamed.long_data as *mut libc::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_external_scanner_state_data(
    mut self_0: *const ExternalScannerState,
) -> *const libc::c_char {
    if (*self_0).length as libc::c_ulong
        > ::std::mem::size_of::<[libc::c_char; 24]>() as libc::c_ulong
    {
        return (*self_0).c2rust_unnamed.long_data;
    } else {
        return (*self_0).c2rust_unnamed.short_data.as_ptr();
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_external_scanner_state_eq(
    mut a: *const ExternalScannerState,
    mut b: *const ExternalScannerState,
) -> bool {
    return a == b
        || (*a).length == (*b).length
            && memcmp(
                ts_external_scanner_state_data(a) as *const libc::c_void,
                ts_external_scanner_state_data(b) as *const libc::c_void,
                (*a).length as usize,
            ) == 0;
}
// SubtreeArray
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_array_copy(
    mut self_0: SubtreeArray,
    mut dest: *mut SubtreeArray,
) {
    (*dest).size = self_0.size;
    (*dest).capacity = self_0.capacity;
    (*dest).contents = self_0.contents;
    if self_0.capacity > 0 as libc::c_int as libc::c_uint {
        (*dest).contents = ts_calloc(
            self_0.capacity as size_t,
            ::std::mem::size_of::<Subtree>() as libc::c_ulong,
        ) as *mut Subtree;
        memcpy(
            (*dest).contents as *mut libc::c_void,
            self_0.contents as *const libc::c_void,
            (self_0.size as usize).wrapping_mul(::std::mem::size_of::<Subtree>() as usize),
        );
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        while i < self_0.size {
            ts_subtree_retain(*(*dest).contents.offset(i as isize));
            i = i.wrapping_add(1)
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_array_delete(
    mut pool: *mut SubtreePool,
    mut self_0: *mut SubtreeArray,
) {
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*self_0).size {
        ts_subtree_release(pool, *(*self_0).contents.offset(i as isize));
        i = i.wrapping_add(1)
    }
    array__delete(self_0 as *mut VoidArray);
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_array_remove_trailing_extras(
    mut self_0: *mut SubtreeArray,
) -> SubtreeArray {
    let mut result: SubtreeArray = {
        let mut init = SubtreeArray {
            contents: 0 as *mut Subtree,
            size: 0 as libc::c_int as uint32_t,
            capacity: 0 as libc::c_int as uint32_t,
        };
        init
    };
    let mut i: uint32_t = (*self_0)
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint);
    while i.wrapping_add(1 as libc::c_int as libc::c_uint) > 0 as libc::c_int as libc::c_uint {
        let mut child: Subtree = *(*self_0).contents.offset(i as isize);
        if !ts_subtree_extra(child) {
            break;
        }
        array__grow(
            &mut result as *mut SubtreeArray as *mut VoidArray,
            1 as libc::c_int as size_t,
            ::std::mem::size_of::<Subtree>() as libc::c_ulong,
        );
        let fresh4 = result.size;
        result.size = result.size.wrapping_add(1);
        *result.contents.offset(fresh4 as isize) = child;
        i = i.wrapping_sub(1)
    }
    (*self_0).size = i.wrapping_add(1 as libc::c_int as libc::c_uint);
    ts_subtree_array_reverse(&mut result);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_array_reverse(mut self_0: *mut SubtreeArray) {
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    let mut limit: uint32_t = (*self_0)
        .size
        .wrapping_div(2 as libc::c_int as libc::c_uint);
    while i < limit {
        let mut reverse_index: size_t = (*self_0)
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            .wrapping_sub(i) as size_t;
        let mut swap: Subtree = *(*self_0).contents.offset(i as isize);
        *(*self_0).contents.offset(i as isize) = *(*self_0).contents.offset(reverse_index as isize);
        *(*self_0).contents.offset(reverse_index as isize) = swap;
        i = i.wrapping_add(1)
    }
}
// SubtreePool
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_pool_new(mut capacity: uint32_t) -> SubtreePool {
    let mut self_0: SubtreePool = {
        let mut init = SubtreePool {
            free_trees: {
                let mut init = MutableSubtreeArray {
                    contents: 0 as *mut MutableSubtree,
                    size: 0 as libc::c_int as uint32_t,
                    capacity: 0 as libc::c_int as uint32_t,
                };
                init
            },
            tree_stack: {
                let mut init = MutableSubtreeArray {
                    contents: 0 as *mut MutableSubtree,
                    size: 0 as libc::c_int as uint32_t,
                    capacity: 0 as libc::c_int as uint32_t,
                };
                init
            },
        };
        init
    };
    array__reserve(
        &mut self_0.free_trees as *mut MutableSubtreeArray as *mut VoidArray,
        ::std::mem::size_of::<MutableSubtree>() as libc::c_ulong,
        capacity,
    );
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_pool_delete(mut self_0: *mut SubtreePool) {
    if !(*self_0).free_trees.contents.is_null() {
        let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
        while i < (*self_0).free_trees.size {
            ts_free((*(*self_0).free_trees.contents.offset(i as isize)).ptr as *mut libc::c_void);
            i = i.wrapping_add(1)
        }
        array__delete(&mut (*self_0).free_trees as *mut MutableSubtreeArray as *mut VoidArray);
    }
    if !(*self_0).tree_stack.contents.is_null() {
        array__delete(&mut (*self_0).tree_stack as *mut MutableSubtreeArray as *mut VoidArray);
    };
}
unsafe extern "C" fn ts_subtree_pool_allocate(
    mut self_0: *mut SubtreePool,
) -> *mut SubtreeHeapData {
    if (*self_0).free_trees.size > 0 as libc::c_int as libc::c_uint {
        (*self_0).free_trees.size = (*self_0).free_trees.size.wrapping_sub(1);
        return (*(*self_0)
            .free_trees
            .contents
            .offset((*self_0).free_trees.size as isize))
        .ptr;
    } else {
        return ts_malloc(::std::mem::size_of::<SubtreeHeapData>() as libc::c_ulong)
            as *mut SubtreeHeapData;
    };
}
unsafe extern "C" fn ts_subtree_pool_free(
    mut self_0: *mut SubtreePool,
    mut tree: *mut SubtreeHeapData,
) {
    if (*self_0).free_trees.capacity > 0 as libc::c_int as libc::c_uint
        && (*self_0)
            .free_trees
            .size
            .wrapping_add(1 as libc::c_int as libc::c_uint)
            <= 32 as libc::c_int as libc::c_uint
    {
        array__grow(
            &mut (*self_0).free_trees as *mut MutableSubtreeArray as *mut VoidArray,
            1 as libc::c_int as size_t,
            ::std::mem::size_of::<MutableSubtree>() as libc::c_ulong,
        );
        let fresh5 = (*self_0).free_trees.size;
        (*self_0).free_trees.size = (*self_0).free_trees.size.wrapping_add(1);
        *(*self_0).free_trees.contents.offset(fresh5 as isize) = MutableSubtree { ptr: tree }
    } else {
        ts_free(tree as *mut libc::c_void);
    };
}
// Subtree
#[inline]
unsafe extern "C" fn ts_subtree_can_inline(
    mut padding: Length,
    mut size: Length,
    mut lookahead_bytes: uint32_t,
) -> bool {
    return padding.bytes < 255 as libc::c_int as libc::c_uint
        && padding.extent.row < 16 as libc::c_int as libc::c_uint
        && padding.extent.column < 255 as libc::c_int as libc::c_uint
        && size.extent.row == 0 as libc::c_int as libc::c_uint
        && size.extent.column < 255 as libc::c_int as libc::c_uint
        && lookahead_bytes < 16 as libc::c_int as libc::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_new_leaf(
    mut pool: *mut SubtreePool,
    mut symbol: TSSymbol,
    mut padding: Length,
    mut size: Length,
    mut lookahead_bytes: uint32_t,
    mut parse_state: TSStateId,
    mut has_external_tokens: bool,
    mut is_keyword: bool,
    mut language: *const TSLanguage,
) -> Subtree {
    let mut metadata: TSSymbolMetadata = ts_language_symbol_metadata(language, symbol);
    let mut extra: bool = symbol as libc::c_int == 0 as libc::c_int;
    let mut is_inline: bool = symbol as libc::c_int <= 255 as libc::c_int
        && !has_external_tokens
        && ts_subtree_can_inline(padding, size, lookahead_bytes) as libc::c_int != 0;
    if is_inline {
        return Subtree {
            data: {
                let mut init = SubtreeInlineData {
                    is_inline_visible_named_extra_has_changes_is_missing_is_keyword: [0; 1],
                    padding_rows_lookahead_bytes: [0; 1],
                    symbol: symbol as uint8_t,
                    padding_bytes: padding.bytes as uint8_t,
                    size_bytes: size.bytes as uint8_t,
                    padding_columns: padding.extent.column as uint8_t,
                    parse_state: parse_state,
                };
                init.set_is_inline(1 as libc::c_int != 0);
                init.set_visible(metadata.visible());
                init.set_named(metadata.named());
                init.set_extra(extra);
                init.set_has_changes(0 as libc::c_int != 0);
                init.set_is_missing(0 as libc::c_int != 0);
                init.set_is_keyword(is_keyword);
                init.set_padding_rows(padding.extent.row as uint8_t);
                init.set_lookahead_bytes(lookahead_bytes as uint8_t);
                init
            },
        };
    } else {
        let mut data: *mut SubtreeHeapData = ts_subtree_pool_allocate(pool);
        *data = {
            let mut init =
                    SubtreeHeapData{visible_named_extra_fragile_left_fragile_right_has_changes_has_external_tokens_is_missing_is_keyword:
                                        [0; 2],
                                    c2rust_padding: [0; 2],
                                    ref_count: 1 as libc::c_int as uint32_t,
                                    padding: padding,
                                    size: size,
                                    lookahead_bytes: lookahead_bytes,
                                    error_cost: 0 as libc::c_int as uint32_t,
                                    child_count: 0 as libc::c_int as uint32_t,
                                    symbol: symbol,
                                    parse_state: parse_state,
                                    c2rust_unnamed:
                                        SubtreeHeapDataContent{c2rust_unnamed:
                                                            {
                                                                let mut init =
                                                                    SubtreeHeapDataContentData{children:
                                                                                        0
                                                                                            as
                                                                                            *mut Subtree,
                                                                                    visible_child_count:
                                                                                        0,
                                                                                    named_child_count:
                                                                                        0,
                                                                                    node_count:
                                                                                        0,
                                                                                    repeat_depth:
                                                                                        0,
                                                                                    dynamic_precedence:
                                                                                        0,
                                                                                    production_id:
                                                                                        0,
                                                                                    first_leaf:
                                                                                        {
                                                                                            let mut init =
                                                                                                SubtreeHeapDataContentDataFirstLeaf{symbol:
                                                                                                                    0
                                                                                                                        as
                                                                                                                        libc::c_int
                                                                                                                        as
                                                                                                                        TSSymbol,
                                                                                                                parse_state:
                                                                                                                    0
                                                                                                                        as
                                                                                                                        libc::c_int
                                                                                                                        as
                                                                                                                        TSStateId,};
                                                                                            init
                                                                                        },};
                                                                init
                                                            },},};
            init.set_visible(metadata.visible());
            init.set_named(metadata.named());
            init.set_extra(extra);
            init.set_fragile_left(0 as libc::c_int != 0);
            init.set_fragile_right(0 as libc::c_int != 0);
            init.set_has_changes(0 as libc::c_int != 0);
            init.set_has_external_tokens(has_external_tokens);
            init.set_is_missing(0 as libc::c_int != 0);
            init.set_is_keyword(is_keyword);
            init
        };
        return Subtree { ptr: data };
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_set_symbol(
    mut self_0: *mut MutableSubtree,
    mut symbol: TSSymbol,
    mut language: *const TSLanguage,
) {
    let mut metadata: TSSymbolMetadata = ts_language_symbol_metadata(language, symbol);
    if (*self_0).data.is_inline() {
        assert!((symbol as libc::c_int) < 255 as libc::c_int);
        (*self_0).data.symbol = symbol as uint8_t;
        (*self_0).data.set_named(metadata.named());
        (*self_0).data.set_visible(metadata.visible())
    } else {
        (*(*self_0).ptr).symbol = symbol;
        (*(*self_0).ptr).set_named(metadata.named());
        (*(*self_0).ptr).set_visible(metadata.visible())
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_new_error(
    mut pool: *mut SubtreePool,
    mut lookahead_char: int32_t,
    mut padding: Length,
    mut size: Length,
    mut bytes_scanned: uint32_t,
    mut parse_state: TSStateId,
    mut language: *const TSLanguage,
) -> Subtree {
    let mut result: Subtree = ts_subtree_new_leaf(
        pool,
        -(1 as libc::c_int) as TSSymbol,
        padding,
        size,
        bytes_scanned,
        parse_state,
        0 as libc::c_int != 0,
        0 as libc::c_int != 0,
        language,
    );
    let mut data: *mut SubtreeHeapData = result.ptr as *mut SubtreeHeapData;
    (*data).set_fragile_left(1 as libc::c_int != 0);
    (*data).set_fragile_right(1 as libc::c_int != 0);
    (*data).c2rust_unnamed.lookahead_char = lookahead_char;
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_make_mut(
    mut pool: *mut SubtreePool,
    mut self_0: Subtree,
) -> MutableSubtree {
    if self_0.data.is_inline() {
        return MutableSubtree { data: self_0.data };
    }
    if (*self_0.ptr).ref_count == 1 as libc::c_int as libc::c_uint {
        return ts_subtree_to_mut_unsafe(self_0);
    }
    let mut result: *mut SubtreeHeapData = ts_subtree_pool_allocate(pool);
    memcpy(
        result as *mut libc::c_void,
        self_0.ptr as *const libc::c_void,
        ::std::mem::size_of::<SubtreeHeapData>() as usize,
    );
    if (*result).child_count > 0 as libc::c_int as libc::c_uint {
        (*result).c2rust_unnamed.c2rust_unnamed.children = ts_calloc(
            (*self_0.ptr).child_count as size_t,
            ::std::mem::size_of::<Subtree>() as libc::c_ulong,
        ) as *mut Subtree;
        memcpy(
            (*result).c2rust_unnamed.c2rust_unnamed.children as *mut libc::c_void,
            (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.children as *const libc::c_void,
            ((*result).child_count as usize)
                .wrapping_mul(::std::mem::size_of::<Subtree>() as usize),
        );
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        while i < (*result).child_count {
            ts_subtree_retain(
                *(*result)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .children
                    .offset(i as isize),
            );
            i = i.wrapping_add(1)
        }
    } else if (*result).has_external_tokens() {
        (*result).c2rust_unnamed.external_scanner_state =
            ts_external_scanner_state_copy(&(*self_0.ptr).c2rust_unnamed.external_scanner_state)
    }
    ::std::ptr::write_volatile(
        &mut (*result).ref_count as *mut uint32_t,
        1 as libc::c_int as uint32_t,
    );
    ts_subtree_release(pool, self_0);
    return MutableSubtree { ptr: result };
}
unsafe extern "C" fn ts_subtree__compress(
    mut self_0: MutableSubtree,
    mut count: libc::c_uint,
    mut language: *const TSLanguage,
    mut stack: *mut MutableSubtreeArray,
) {
    let mut initial_stack_size: libc::c_uint = (*stack).size;
    let mut tree: MutableSubtree = self_0;
    let mut symbol: TSSymbol = (*tree.ptr).symbol;
    let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    while i < count {
        if (*tree.ptr).ref_count > 1 as libc::c_int as libc::c_uint
            || (*tree.ptr).child_count < 2 as libc::c_int as libc::c_uint
        {
            break;
        }
        let mut child: MutableSubtree = ts_subtree_to_mut_unsafe(
            *(*tree.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(0 as libc::c_int as isize),
        );
        if child.data.is_inline() as libc::c_int != 0
            || (*child.ptr).child_count < 2 as libc::c_int as libc::c_uint
            || (*child.ptr).ref_count > 1 as libc::c_int as libc::c_uint
            || (*child.ptr).symbol as libc::c_int != symbol as libc::c_int
        {
            break;
        }
        let mut grandchild: MutableSubtree = ts_subtree_to_mut_unsafe(
            *(*child.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(0 as libc::c_int as isize),
        );
        if grandchild.data.is_inline() as libc::c_int != 0
            || (*grandchild.ptr).child_count < 2 as libc::c_int as libc::c_uint
            || (*grandchild.ptr).ref_count > 1 as libc::c_int as libc::c_uint
            || (*grandchild.ptr).symbol as libc::c_int != symbol as libc::c_int
        {
            break;
        }
        *(*tree.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(0 as libc::c_int as isize) = ts_subtree_from_mut(grandchild);
        *(*child.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(0 as libc::c_int as isize) = *(*grandchild.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(
                (*grandchild.ptr)
                    .child_count
                    .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
            );
        *(*grandchild.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(
                (*grandchild.ptr)
                    .child_count
                    .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
            ) = ts_subtree_from_mut(child);
        array__grow(
            stack as *mut VoidArray,
            1 as libc::c_int as size_t,
            ::std::mem::size_of::<MutableSubtree>() as libc::c_ulong,
        );
        let fresh6 = (*stack).size;
        (*stack).size = (*stack).size.wrapping_add(1);
        *(*stack).contents.offset(fresh6 as isize) = tree;
        tree = grandchild;
        i = i.wrapping_add(1)
    }
    while (*stack).size > initial_stack_size {
        (*stack).size = (*stack).size.wrapping_sub(1);
        tree = *(*stack).contents.offset((*stack).size as isize);
        let mut child_0: MutableSubtree = ts_subtree_to_mut_unsafe(
            *(*tree.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(0 as libc::c_int as isize),
        );
        let mut grandchild_0: MutableSubtree = ts_subtree_to_mut_unsafe(
            *(*child_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(
                    (*child_0.ptr)
                        .child_count
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        as isize,
                ),
        );
        ts_subtree_set_children(
            grandchild_0,
            (*grandchild_0.ptr).c2rust_unnamed.c2rust_unnamed.children,
            (*grandchild_0.ptr).child_count,
            language,
        );
        ts_subtree_set_children(
            child_0,
            (*child_0.ptr).c2rust_unnamed.c2rust_unnamed.children,
            (*child_0.ptr).child_count,
            language,
        );
        ts_subtree_set_children(
            tree,
            (*tree.ptr).c2rust_unnamed.c2rust_unnamed.children,
            (*tree.ptr).child_count,
            language,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_balance(
    mut self_0: Subtree,
    mut pool: *mut SubtreePool,
    mut language: *const TSLanguage,
) {
    (*pool).tree_stack.size = 0 as libc::c_int as uint32_t;
    if ts_subtree_child_count(self_0) > 0 as libc::c_int as libc::c_uint
        && (*self_0.ptr).ref_count == 1 as libc::c_int as libc::c_uint
    {
        array__grow(
            &mut (*pool).tree_stack as *mut MutableSubtreeArray as *mut VoidArray,
            1 as libc::c_int as size_t,
            ::std::mem::size_of::<MutableSubtree>() as libc::c_ulong,
        );
        let fresh7 = (*pool).tree_stack.size;
        (*pool).tree_stack.size = (*pool).tree_stack.size.wrapping_add(1);
        *(*pool).tree_stack.contents.offset(fresh7 as isize) = ts_subtree_to_mut_unsafe(self_0)
    }
    while (*pool).tree_stack.size > 0 as libc::c_int as libc::c_uint {
        (*pool).tree_stack.size = (*pool).tree_stack.size.wrapping_sub(1);
        let mut tree: MutableSubtree = *(*pool)
            .tree_stack
            .contents
            .offset((*pool).tree_stack.size as isize);
        if (*tree.ptr).c2rust_unnamed.c2rust_unnamed.repeat_depth > 0 as libc::c_int as libc::c_uint
        {
            let mut child1: Subtree = *(*tree.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(0 as libc::c_int as isize);
            let mut child2: Subtree = *(*tree.ptr).c2rust_unnamed.c2rust_unnamed.children.offset(
                (*tree.ptr)
                    .child_count
                    .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
            );
            let mut repeat_delta: libc::c_long = ts_subtree_repeat_depth(child1) as libc::c_long
                - ts_subtree_repeat_depth(child2) as libc::c_long;
            if repeat_delta > 0 as libc::c_int as libc::c_long {
                let mut n: libc::c_uint = repeat_delta as libc::c_uint;
                let mut i: libc::c_uint = n.wrapping_div(2 as libc::c_int as libc::c_uint);
                while i > 0 as libc::c_int as libc::c_uint {
                    ts_subtree__compress(tree, i, language, &mut (*pool).tree_stack);
                    n = n.wrapping_sub(i);
                    i = i.wrapping_div(2 as libc::c_int as libc::c_uint)
                }
            }
        }
        let mut i_0: uint32_t = 0 as libc::c_int as uint32_t;
        while i_0 < (*tree.ptr).child_count {
            let mut child: Subtree = *(*tree.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(i_0 as isize);
            if ts_subtree_child_count(child) > 0 as libc::c_int as libc::c_uint
                && (*child.ptr).ref_count == 1 as libc::c_int as libc::c_uint
            {
                array__grow(
                    &mut (*pool).tree_stack as *mut MutableSubtreeArray as *mut VoidArray,
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<MutableSubtree>() as libc::c_ulong,
                );
                let fresh8 = (*pool).tree_stack.size;
                (*pool).tree_stack.size = (*pool).tree_stack.size.wrapping_add(1);
                *(*pool).tree_stack.contents.offset(fresh8 as isize) =
                    ts_subtree_to_mut_unsafe(child)
            }
            i_0 = i_0.wrapping_add(1)
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_set_children(
    mut self_0: MutableSubtree,
    mut children: *mut Subtree,
    mut child_count: uint32_t,
    mut language: *const TSLanguage,
) {
    assert!(!self_0.data.is_inline());
    if (*self_0.ptr).child_count > 0 as libc::c_int as libc::c_uint
        && children != (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.children
    {
        ts_free((*self_0.ptr).c2rust_unnamed.c2rust_unnamed.children as *mut libc::c_void);
    }
    (*self_0.ptr).child_count = child_count;
    (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.children = children;
    (*self_0.ptr)
        .c2rust_unnamed
        .c2rust_unnamed
        .named_child_count = 0 as libc::c_int as uint32_t;
    (*self_0.ptr)
        .c2rust_unnamed
        .c2rust_unnamed
        .visible_child_count = 0 as libc::c_int as uint32_t;
    (*self_0.ptr).error_cost = 0 as libc::c_int as uint32_t;
    (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.repeat_depth = 0 as libc::c_int as uint32_t;
    (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.node_count = 1 as libc::c_int as uint32_t;
    (*self_0.ptr).set_has_external_tokens(0 as libc::c_int != 0);
    (*self_0.ptr)
        .c2rust_unnamed
        .c2rust_unnamed
        .dynamic_precedence = 0 as libc::c_int;
    let mut non_extra_index: uint32_t = 0 as libc::c_int as uint32_t;
    let mut alias_sequence: *const TSSymbol = ts_language_alias_sequence(
        language,
        (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.production_id as uint32_t,
    );
    let mut lookahead_end_byte: uint32_t = 0 as libc::c_int as uint32_t;
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*self_0.ptr).child_count {
        let mut child: Subtree = *(*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(i as isize);
        if i == 0 as libc::c_int as libc::c_uint {
            (*self_0.ptr).padding = ts_subtree_padding(child);
            (*self_0.ptr).size = ts_subtree_size(child)
        } else {
            (*self_0.ptr).size = length_add((*self_0.ptr).size, ts_subtree_total_size(child))
        }
        let mut child_lookahead_end_byte: uint32_t = (*self_0.ptr)
            .padding
            .bytes
            .wrapping_add((*self_0.ptr).size.bytes)
            .wrapping_add(ts_subtree_lookahead_bytes(child));
        if child_lookahead_end_byte > lookahead_end_byte {
            lookahead_end_byte = child_lookahead_end_byte
        }
        if ts_subtree_symbol(child) as libc::c_int
            != -(1 as libc::c_int) as TSSymbol as libc::c_int - 1 as libc::c_int
        {
            (*self_0.ptr).error_cost = ((*self_0.ptr).error_cost as libc::c_uint)
                .wrapping_add(ts_subtree_error_cost(child))
                as uint32_t as uint32_t
        }
        (*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .dynamic_precedence += ts_subtree_dynamic_precedence(child);
        (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.node_count =
            ((*self_0.ptr).c2rust_unnamed.c2rust_unnamed.node_count as libc::c_uint)
                .wrapping_add(ts_subtree_node_count(child)) as uint32_t as uint32_t;
        if !alias_sequence.is_null()
            && *alias_sequence.offset(non_extra_index as isize) as libc::c_int != 0 as libc::c_int
            && !ts_subtree_extra(child)
        {
            (*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .visible_child_count = (*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .visible_child_count
                .wrapping_add(1);
            if ts_language_symbol_metadata(
                language,
                *alias_sequence.offset(non_extra_index as isize),
            )
            .named()
            {
                (*self_0.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .named_child_count = (*self_0.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .named_child_count
                    .wrapping_add(1)
            }
        } else if ts_subtree_visible(child) {
            (*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .visible_child_count = (*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .visible_child_count
                .wrapping_add(1);
            if ts_subtree_named(child) {
                (*self_0.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .named_child_count = (*self_0.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .named_child_count
                    .wrapping_add(1)
            }
        } else if ts_subtree_child_count(child) > 0 as libc::c_int as libc::c_uint {
            (*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .visible_child_count = ((*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .visible_child_count as libc::c_uint)
                .wrapping_add(
                    (*child.ptr)
                        .c2rust_unnamed
                        .c2rust_unnamed
                        .visible_child_count,
                ) as uint32_t as uint32_t;
            (*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .named_child_count = ((*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .named_child_count as libc::c_uint)
                .wrapping_add((*child.ptr).c2rust_unnamed.c2rust_unnamed.named_child_count)
                as uint32_t as uint32_t
        }
        if ts_subtree_has_external_tokens(child) {
            (*self_0.ptr).set_has_external_tokens(1 as libc::c_int != 0)
        }
        if ts_subtree_is_error(child) {
            (*self_0.ptr).set_fragile_right(1 as libc::c_int != 0);
            (*self_0.ptr).set_fragile_left((*self_0.ptr).fragile_right());
            (*self_0.ptr).parse_state = TS_TREE_STATE_NONE
        }
        if !ts_subtree_extra(child) {
            non_extra_index = non_extra_index.wrapping_add(1)
        }
        i = i.wrapping_add(1)
    }
    (*self_0.ptr).lookahead_bytes = lookahead_end_byte
        .wrapping_sub((*self_0.ptr).size.bytes)
        .wrapping_sub((*self_0.ptr).padding.bytes);
    if (*self_0.ptr).symbol as libc::c_int == -(1 as libc::c_int) as TSSymbol as libc::c_int
        || (*self_0.ptr).symbol as libc::c_int
            == -(1 as libc::c_int) as TSSymbol as libc::c_int - 1 as libc::c_int
    {
        (*self_0.ptr).error_cost = ((*self_0.ptr).error_cost as libc::c_uint).wrapping_add(
            (500 as libc::c_int as libc::c_uint)
                .wrapping_add(
                    (1 as libc::c_int as libc::c_uint).wrapping_mul((*self_0.ptr).size.bytes),
                )
                .wrapping_add(
                    (30 as libc::c_int as libc::c_uint).wrapping_mul((*self_0.ptr).size.extent.row),
                ),
        ) as uint32_t as uint32_t;
        let mut i_0: uint32_t = 0 as libc::c_int as uint32_t;
        while i_0 < (*self_0.ptr).child_count {
            let mut child_0: Subtree = *(*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(i_0 as isize);
            let mut grandchild_count: uint32_t = ts_subtree_child_count(child_0);
            if !ts_subtree_extra(child_0) {
                if !(ts_subtree_is_error(child_0) as libc::c_int != 0
                    && grandchild_count == 0 as libc::c_int as libc::c_uint)
                {
                    if ts_subtree_visible(child_0) {
                        (*self_0.ptr).error_cost = ((*self_0.ptr).error_cost as libc::c_uint)
                            .wrapping_add(100 as libc::c_int as libc::c_uint)
                            as uint32_t
                            as uint32_t
                    } else if grandchild_count > 0 as libc::c_int as libc::c_uint {
                        (*self_0.ptr).error_cost =
                            ((*self_0.ptr).error_cost as libc::c_uint).wrapping_add(
                                (100 as libc::c_int as libc::c_uint).wrapping_mul(
                                    (*child_0.ptr)
                                        .c2rust_unnamed
                                        .c2rust_unnamed
                                        .visible_child_count,
                                ),
                            ) as uint32_t as uint32_t
                    }
                }
            }
            i_0 = i_0.wrapping_add(1)
        }
    }
    if (*self_0.ptr).child_count > 0 as libc::c_int as libc::c_uint {
        let mut first_child: Subtree = *(*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(0 as libc::c_int as isize);
        let mut last_child: Subtree = *(*self_0.ptr).c2rust_unnamed.c2rust_unnamed.children.offset(
            (*self_0.ptr)
                .child_count
                .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
        );
        (*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .first_leaf
            .symbol = ts_subtree_leaf_symbol(first_child);
        (*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .first_leaf
            .parse_state = ts_subtree_leaf_parse_state(first_child);
        if ts_subtree_fragile_left(first_child) {
            (*self_0.ptr).set_fragile_left(1 as libc::c_int != 0)
        }
        if ts_subtree_fragile_right(last_child) {
            (*self_0.ptr).set_fragile_right(1 as libc::c_int != 0)
        }
        if (*self_0.ptr).child_count >= 2 as libc::c_int as libc::c_uint
            && !(*self_0.ptr).visible()
            && !(*self_0.ptr).named()
            && ts_subtree_symbol(first_child) as libc::c_int == (*self_0.ptr).symbol as libc::c_int
        {
            if ts_subtree_repeat_depth(first_child) > ts_subtree_repeat_depth(last_child) {
                (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.repeat_depth =
                    ts_subtree_repeat_depth(first_child)
                        .wrapping_add(1 as libc::c_int as libc::c_uint)
            } else {
                (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.repeat_depth =
                    ts_subtree_repeat_depth(last_child)
                        .wrapping_add(1 as libc::c_int as libc::c_uint)
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_new_node(
    mut pool: *mut SubtreePool,
    mut symbol: TSSymbol,
    mut children: *mut SubtreeArray,
    mut production_id: libc::c_uint,
    mut language: *const TSLanguage,
) -> MutableSubtree {
    let mut metadata: TSSymbolMetadata = ts_language_symbol_metadata(language, symbol);
    let mut fragile: bool = symbol as libc::c_int == -(1 as libc::c_int) as TSSymbol as libc::c_int
        || symbol as libc::c_int
            == -(1 as libc::c_int) as TSSymbol as libc::c_int - 1 as libc::c_int;
    let mut data: *mut SubtreeHeapData = ts_subtree_pool_allocate(pool);
    *data = {
        let mut init =
                SubtreeHeapData{visible_named_extra_fragile_left_fragile_right_has_changes_has_external_tokens_is_missing_is_keyword:
                                    [0; 2],
                                c2rust_padding: [0; 2],
                                ref_count: 1 as libc::c_int as uint32_t,
                                padding:
                                    Length{bytes: 0,
                                           extent:
                                               TSPoint{row: 0, column: 0,},},
                                size:
                                    Length{bytes: 0,
                                           extent:
                                               TSPoint{row: 0, column: 0,},},
                                lookahead_bytes: 0,
                                error_cost: 0,
                                child_count: 0,
                                symbol: symbol,
                                parse_state: 0,
                                c2rust_unnamed:
                                    SubtreeHeapDataContent{c2rust_unnamed:
                                                        {
                                                            let mut init =
                                                                SubtreeHeapDataContentData{children:
                                                                                    0
                                                                                        as
                                                                                        *mut Subtree,
                                                                                visible_child_count:
                                                                                    0,
                                                                                named_child_count:
                                                                                    0,
                                                                                node_count:
                                                                                    0
                                                                                        as
                                                                                        libc::c_int
                                                                                        as
                                                                                        uint32_t,
                                                                                repeat_depth:
                                                                                    0,
                                                                                dynamic_precedence:
                                                                                    0,
                                                                                production_id:
                                                                                    production_id
                                                                                        as
                                                                                        uint16_t,
                                                                                first_leaf:
                                                                                    {
                                                                                        let mut init =
                                                                                            SubtreeHeapDataContentDataFirstLeaf{symbol:
                                                                                                                0
                                                                                                                    as
                                                                                                                    libc::c_int
                                                                                                                    as
                                                                                                                    TSSymbol,
                                                                                                            parse_state:
                                                                                                                0
                                                                                                                    as
                                                                                                                    libc::c_int
                                                                                                                    as
                                                                                                                    TSStateId,};
                                                                                        init
                                                                                    },};
                                                            init
                                                        },},};
        init.set_visible(metadata.visible());
        init.set_named(metadata.named());
        init.set_extra(false);
        init.set_fragile_left(fragile);
        init.set_fragile_right(fragile);
        init.set_has_changes(0 as libc::c_int != 0);
        init.set_has_external_tokens(false);
        init.set_is_missing(false);
        init.set_is_keyword(0 as libc::c_int != 0);
        init
    };
    let mut result: MutableSubtree = MutableSubtree { ptr: data };
    ts_subtree_set_children(result, (*children).contents, (*children).size, language);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_new_error_node(
    mut pool: *mut SubtreePool,
    mut children: *mut SubtreeArray,
    mut extra: bool,
    mut language: *const TSLanguage,
) -> Subtree {
    let mut result: MutableSubtree = ts_subtree_new_node(
        pool,
        -(1 as libc::c_int) as TSSymbol,
        children,
        0 as libc::c_int as libc::c_uint,
        language,
    );
    (*result.ptr).set_extra(extra);
    return ts_subtree_from_mut(result);
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_new_missing_leaf(
    mut pool: *mut SubtreePool,
    mut symbol: TSSymbol,
    mut padding: Length,
    mut language: *const TSLanguage,
) -> Subtree {
    let mut result: Subtree = ts_subtree_new_leaf(
        pool,
        symbol,
        padding,
        length_zero(),
        0 as libc::c_int as uint32_t,
        0 as libc::c_int as TSStateId,
        0 as libc::c_int != 0,
        0 as libc::c_int != 0,
        language,
    );
    if result.data.is_inline() {
        result.data.set_is_missing(1 as libc::c_int != 0)
    } else {
        let ref mut fresh9 = *(result.ptr as *mut SubtreeHeapData);
        (*fresh9).set_is_missing(1 as libc::c_int != 0)
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_retain(mut self_0: Subtree) {
    if self_0.data.is_inline() {
        return;
    }
    assert!((*self_0.ptr).ref_count > 0 as libc::c_int as libc::c_uint);
    atomic_inc(&(*self_0.ptr).ref_count as *const uint32_t as *mut uint32_t);
    assert!((*self_0.ptr).ref_count != 0 as libc::c_int as libc::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_release(mut pool: *mut SubtreePool, mut self_0: Subtree) {
    if self_0.data.is_inline() {
        return;
    }
    (*pool).tree_stack.size = 0 as libc::c_int as uint32_t;
    assert!((*self_0.ptr).ref_count > 0 as libc::c_int as libc::c_uint);
    if atomic_dec(&(*self_0.ptr).ref_count as *const uint32_t as *mut uint32_t)
        == 0 as libc::c_int as libc::c_uint
    {
        array__grow(
            &mut (*pool).tree_stack as *mut MutableSubtreeArray as *mut VoidArray,
            1 as libc::c_int as size_t,
            ::std::mem::size_of::<MutableSubtree>() as libc::c_ulong,
        );
        let fresh10 = (*pool).tree_stack.size;
        (*pool).tree_stack.size = (*pool).tree_stack.size.wrapping_add(1);
        *(*pool).tree_stack.contents.offset(fresh10 as isize) = ts_subtree_to_mut_unsafe(self_0)
    }
    while (*pool).tree_stack.size > 0 as libc::c_int as libc::c_uint {
        (*pool).tree_stack.size = (*pool).tree_stack.size.wrapping_sub(1);
        let mut tree: MutableSubtree = *(*pool)
            .tree_stack
            .contents
            .offset((*pool).tree_stack.size as isize);
        if (*tree.ptr).child_count > 0 as libc::c_int as libc::c_uint {
            let mut i: uint32_t = 0 as libc::c_int as uint32_t;
            while i < (*tree.ptr).child_count {
                let mut child: Subtree = *(*tree.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .children
                    .offset(i as isize);
                if !child.data.is_inline() {
                    assert!((*child.ptr).ref_count > 0 as libc::c_int as libc::c_uint);
                    if atomic_dec(&(*child.ptr).ref_count as *const uint32_t as *mut uint32_t)
                        == 0 as libc::c_int as libc::c_uint
                    {
                        array__grow(
                            &mut (*pool).tree_stack as *mut MutableSubtreeArray as *mut VoidArray,
                            1 as libc::c_int as size_t,
                            ::std::mem::size_of::<MutableSubtree>() as libc::c_ulong,
                        );
                        let fresh11 = (*pool).tree_stack.size;
                        (*pool).tree_stack.size = (*pool).tree_stack.size.wrapping_add(1);
                        *(*pool).tree_stack.contents.offset(fresh11 as isize) =
                            ts_subtree_to_mut_unsafe(child)
                    }
                }
                i = i.wrapping_add(1)
            }
            ts_free((*tree.ptr).c2rust_unnamed.c2rust_unnamed.children as *mut libc::c_void);
        } else if (*tree.ptr).has_external_tokens() {
            ts_external_scanner_state_delete(
                &mut (*tree.ptr).c2rust_unnamed.external_scanner_state,
            );
        }
        ts_subtree_pool_free(pool, tree.ptr);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_eq(mut self_0: Subtree, mut other: Subtree) -> bool {
    if self_0.data.is_inline() as libc::c_int != 0 || other.data.is_inline() as libc::c_int != 0 {
        return memcmp(
            &mut self_0 as *mut Subtree as *const libc::c_void,
            &mut other as *mut Subtree as *const libc::c_void,
            ::std::mem::size_of::<SubtreeInlineData>(),
        ) == 0 as libc::c_int;
    }
    if !self_0.ptr.is_null() {
        if other.ptr.is_null() {
            return 0 as libc::c_int != 0;
        }
    } else {
        return other.ptr.is_null();
    }
    if (*self_0.ptr).symbol as libc::c_int != (*other.ptr).symbol as libc::c_int {
        return 0 as libc::c_int != 0;
    }
    if (*self_0.ptr).visible() as libc::c_int != (*other.ptr).visible() as libc::c_int {
        return 0 as libc::c_int != 0;
    }
    if (*self_0.ptr).named() as libc::c_int != (*other.ptr).named() as libc::c_int {
        return 0 as libc::c_int != 0;
    }
    if (*self_0.ptr).padding.bytes != (*other.ptr).padding.bytes {
        return 0 as libc::c_int != 0;
    }
    if (*self_0.ptr).size.bytes != (*other.ptr).size.bytes {
        return 0 as libc::c_int != 0;
    }
    if (*self_0.ptr).symbol as libc::c_int == -(1 as libc::c_int) as TSSymbol as libc::c_int {
        return (*self_0.ptr).c2rust_unnamed.lookahead_char
            == (*other.ptr).c2rust_unnamed.lookahead_char;
    }
    if (*self_0.ptr).child_count != (*other.ptr).child_count {
        return 0 as libc::c_int != 0;
    }
    if (*self_0.ptr).child_count > 0 as libc::c_int as libc::c_uint {
        if (*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .visible_child_count
            != (*other.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .visible_child_count
        {
            return 0 as libc::c_int != 0;
        }
        if (*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .named_child_count
            != (*other.ptr).c2rust_unnamed.c2rust_unnamed.named_child_count
        {
            return 0 as libc::c_int != 0;
        }
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        while i < (*self_0.ptr).child_count {
            if !ts_subtree_eq(
                *(*self_0.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .children
                    .offset(i as isize),
                *(*other.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .children
                    .offset(i as isize),
            ) {
                return 0 as libc::c_int != 0;
            }
            i = i.wrapping_add(1)
        }
    }
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_compare(mut left: Subtree, mut right: Subtree) -> libc::c_int {
    if (ts_subtree_symbol(left) as libc::c_int) < ts_subtree_symbol(right) as libc::c_int {
        return -(1 as libc::c_int);
    }
    if (ts_subtree_symbol(right) as libc::c_int) < ts_subtree_symbol(left) as libc::c_int {
        return 1 as libc::c_int;
    }
    if ts_subtree_child_count(left) < ts_subtree_child_count(right) {
        return -(1 as libc::c_int);
    }
    if ts_subtree_child_count(right) < ts_subtree_child_count(left) {
        return 1 as libc::c_int;
    }
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    let mut n: uint32_t = ts_subtree_child_count(left);
    while i < n {
        let mut left_child: Subtree = *(*left.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(i as isize);
        let mut right_child: Subtree = *(*right.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(i as isize);
        match ts_subtree_compare(left_child, right_child) {
            -1 => return -(1 as libc::c_int),
            1 => return 1 as libc::c_int,
            _ => {}
        }
        i = i.wrapping_add(1)
    }
    return 0 as libc::c_int;
}
#[inline]
unsafe extern "C" fn ts_subtree_set_has_changes(mut self_0: *mut MutableSubtree) {
    if (*self_0).data.is_inline() {
        (*self_0).data.set_has_changes(1 as libc::c_int != 0)
    } else {
        (*(*self_0).ptr).set_has_changes(1 as libc::c_int != 0)
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_edit(
    mut self_0: Subtree,
    mut edit: *const TSInputEdit,
    mut pool: *mut SubtreePool,
) -> Subtree {
    #[derive(Copy, Clone)]
    #[repr(C)]
    struct LocStackEntry {
        pub tree: *mut Subtree,
        pub edit: Edit,
    }

    #[derive(Copy, Clone)]
    #[repr(C)]
    struct StackEntryArray {
        pub contents: *mut LocStackEntry,
        pub size: uint32_t,
        pub capacity: uint32_t,
    }

    let mut stack: StackEntryArray = {
        let mut init = StackEntryArray {
            contents: 0 as *mut LocStackEntry,
            size: 0 as libc::c_int as uint32_t,
            capacity: 0 as libc::c_int as uint32_t,
        };
        init
    };
    array__grow(
        &mut stack as *mut StackEntryArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<LocStackEntry>() as libc::c_ulong,
    );
    let fresh12 = stack.size;
    stack.size = stack.size.wrapping_add(1);
    *stack.contents.offset(fresh12 as isize) = {
        let mut init = LocStackEntry {
            tree: &mut self_0,
            edit: {
                let mut init = Edit {
                    start: {
                        let mut init = Length {
                            bytes: (*edit).start_byte,
                            extent: (*edit).start_point,
                        };
                        init
                    },
                    old_end: {
                        let mut init = Length {
                            bytes: (*edit).old_end_byte,
                            extent: (*edit).old_end_point,
                        };
                        init
                    },
                    new_end: {
                        let mut init = Length {
                            bytes: (*edit).new_end_byte,
                            extent: (*edit).new_end_point,
                        };
                        init
                    },
                };
                init
            },
        };
        init
    };
    while stack.size != 0 {
        stack.size = stack.size.wrapping_sub(1);
        let mut entry: LocStackEntry = *stack.contents.offset(stack.size as isize);
        let mut edit_0: Edit = entry.edit;
        let mut is_noop: bool = edit_0.old_end.bytes == edit_0.start.bytes
            && edit_0.new_end.bytes == edit_0.start.bytes;
        let mut is_pure_insertion: bool = edit_0.old_end.bytes == edit_0.start.bytes;
        let mut size: Length = ts_subtree_size(*entry.tree);
        let mut padding: Length = ts_subtree_padding(*entry.tree);
        let mut lookahead_bytes: uint32_t = ts_subtree_lookahead_bytes(*entry.tree);
        let mut end_byte: uint32_t = padding
            .bytes
            .wrapping_add(size.bytes)
            .wrapping_add(lookahead_bytes);
        if edit_0.start.bytes > end_byte
            || is_noop as libc::c_int != 0 && edit_0.start.bytes == end_byte
        {
            continue;
        }
        // If the edit is entirely within the space before this subtree, then shift this
        // subtree over according to the edit without changing its size.
        if edit_0.old_end.bytes <= padding.bytes {
            padding = length_add(edit_0.new_end, length_sub(padding, edit_0.old_end))
        } else if edit_0.start.bytes < padding.bytes {
            size = length_sub(size, length_sub(edit_0.old_end, padding));
            padding = edit_0.new_end
        } else if edit_0.start.bytes == padding.bytes && is_pure_insertion as libc::c_int != 0 {
            padding = edit_0.new_end
        } else {
            // If the edit starts in the space before this subtree and extends into this subtree,
            // shrink the subtree's content to compensate for the change in the space before it.
            // If the edit is a pure insertion right at the start of the subtree,
            // shift the subtree over according to the insertion.
            // If the edit is within this subtree, resize the subtree to reflect the edit.
            let mut total_bytes: uint32_t = padding.bytes.wrapping_add(size.bytes);
            if edit_0.start.bytes < total_bytes
                || edit_0.start.bytes == total_bytes && is_pure_insertion as libc::c_int != 0
            {
                size = length_add(
                    length_sub(edit_0.new_end, padding),
                    length_sub(size, length_sub(edit_0.old_end, padding)),
                )
            }
        }
        let mut result: MutableSubtree = ts_subtree_make_mut(pool, *entry.tree);
        if result.data.is_inline() {
            if ts_subtree_can_inline(padding, size, lookahead_bytes) {
                result.data.padding_bytes = padding.bytes as uint8_t;
                result.data.set_padding_rows(padding.extent.row as uint8_t);
                result.data.padding_columns = padding.extent.column as uint8_t;
                result.data.size_bytes = size.bytes as uint8_t
            } else {
                let mut data: *mut SubtreeHeapData = ts_subtree_pool_allocate(pool);
                ::std::ptr::write_volatile(
                    &mut (*data).ref_count as *mut uint32_t,
                    1 as libc::c_int as uint32_t,
                );
                (*data).padding = padding;
                (*data).size = size;
                (*data).lookahead_bytes = lookahead_bytes;
                (*data).error_cost = 0 as libc::c_int as uint32_t;
                (*data).child_count = 0 as libc::c_int as uint32_t;
                (*data).symbol = result.data.symbol as TSSymbol;
                (*data).parse_state = result.data.parse_state;
                (*data).set_visible(result.data.visible());
                (*data).set_named(result.data.named());
                (*data).set_extra(result.data.extra());
                (*data).set_fragile_left(0 as libc::c_int != 0);
                (*data).set_fragile_right(0 as libc::c_int != 0);
                (*data).set_has_changes(0 as libc::c_int != 0);
                (*data).set_has_external_tokens(0 as libc::c_int != 0);
                (*data).set_is_missing(result.data.is_missing());
                (*data).set_is_keyword(result.data.is_keyword());
                result.ptr = data
            }
        } else {
            (*result.ptr).padding = padding;
            (*result.ptr).size = size
        }
        ts_subtree_set_has_changes(&mut result);
        *entry.tree = ts_subtree_from_mut(result);
        let mut child_left: Length = Length {
            bytes: 0,
            extent: TSPoint { row: 0, column: 0 },
        };
        let mut child_right: Length = length_zero();
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        let mut n: uint32_t = ts_subtree_child_count(*entry.tree);
        while i < n {
            let mut child: *mut Subtree = &mut *(*result.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(i as isize) as *mut Subtree;
            let mut child_size: Length = ts_subtree_total_size(*child);
            child_left = child_right;
            child_right = length_add(child_left, child_size);
            // If this child ends before the edit, it is not affected.
            if !(child_right
                .bytes
                .wrapping_add(ts_subtree_lookahead_bytes(*child))
                < edit_0.start.bytes)
            {
                // If this child starts after the edit, then we're done processing children.
                if child_left.bytes > edit_0.old_end.bytes
                    || child_left.bytes == edit_0.old_end.bytes
                        && child_size.bytes > 0 as libc::c_int as libc::c_uint
                        && i > 0 as libc::c_int as libc::c_uint
                {
                    break;
                }
                // Transform edit into the child's coordinate space.
                let mut child_edit: Edit = {
                    let mut init = Edit {
                        start: length_sub(edit_0.start, child_left),
                        old_end: length_sub(edit_0.old_end, child_left),
                        new_end: length_sub(edit_0.new_end, child_left),
                    };
                    init
                };
                // Clamp child_edit to the child's bounds.
                if edit_0.start.bytes < child_left.bytes {
                    child_edit.start = length_zero()
                }
                if edit_0.old_end.bytes < child_left.bytes {
                    child_edit.old_end = length_zero()
                }
                if edit_0.new_end.bytes < child_left.bytes {
                    child_edit.new_end = length_zero()
                }
                if edit_0.old_end.bytes > child_right.bytes {
                    child_edit.old_end = child_size
                }
                // Interpret all inserted text as applying to the *first* child that touches the edit.
                // Subsequent children are only never have any text inserted into them; they are only
                // shrunk to compensate for the edit.
                if child_right.bytes > edit_0.start.bytes
                    || child_right.bytes == edit_0.start.bytes
                        && is_pure_insertion as libc::c_int != 0
                {
                    edit_0.new_end = edit_0.start
                } else {
                    // Children that occur before the edit are not reshaped by the edit.
                    child_edit.old_end = child_edit.start;
                    child_edit.new_end = child_edit.start
                }
                // Queue processing of this child's subtree.
                array__grow(
                    &mut stack as *mut StackEntryArray as *mut VoidArray,
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<LocStackEntry>() as libc::c_ulong,
                );
                let fresh13 = stack.size;
                stack.size = stack.size.wrapping_add(1);
                *stack.contents.offset(fresh13 as isize) = {
                    let mut init = LocStackEntry {
                        tree: child,
                        edit: child_edit,
                    };
                    init
                }
            }
            i = i.wrapping_add(1)
        }
    }
    array__delete(&mut stack as *mut StackEntryArray as *mut VoidArray);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_last_external_token(mut tree: Subtree) -> Subtree {
    if !ts_subtree_has_external_tokens(tree) {
        return Subtree {
            ptr: 0 as *const SubtreeHeapData,
        };
    }
    while (*tree.ptr).child_count > 0 as libc::c_int as libc::c_uint {
        let mut i: uint32_t = (*tree.ptr)
            .child_count
            .wrapping_sub(1 as libc::c_int as libc::c_uint);
        while i.wrapping_add(1 as libc::c_int as libc::c_uint) > 0 as libc::c_int as libc::c_uint {
            let mut child: Subtree = *(*tree.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(i as isize);
            if ts_subtree_has_external_tokens(child) {
                tree = child;
                break;
            } else {
                i = i.wrapping_sub(1)
            }
        }
    }
    return tree;
}
unsafe extern "C" fn ts_subtree__write_char_to_string(
    mut s: *mut libc::c_char,
    mut n: size_t,
    mut c: int32_t,
) -> size_t {
    if c == -(1 as libc::c_int) {
        return snprintf(
            s,
            n as usize,
            b"INVALID\x00" as *const u8 as *const libc::c_char,
        ) as size_t;
    } else if c == '\u{0}' as i32 {
        return snprintf(
            s,
            n as usize,
            b"\'\\0\'\x00" as *const u8 as *const libc::c_char,
        ) as size_t;
    } else if c == '\n' as i32 {
        return snprintf(
            s,
            n as usize,
            b"\'\\n\'\x00" as *const u8 as *const libc::c_char,
        ) as size_t;
    } else if c == '\t' as i32 {
        return snprintf(
            s,
            n as usize,
            b"\'\\t\'\x00" as *const u8 as *const libc::c_char,
        ) as size_t;
    } else if c == '\r' as i32 {
        return snprintf(
            s,
            n as usize,
            b"\'\\r\'\x00" as *const u8 as *const libc::c_char,
        ) as size_t;
    } else if (0 as libc::c_int) < c
        && c < 128 as libc::c_int
        && ((c as u8).is_ascii_graphic() || c == ' ' as i32)
    {
        return snprintf(
            s,
            n as usize,
            b"\'%c\'\x00" as *const u8 as *const libc::c_char,
            c,
        ) as size_t;
    } else {
        return snprintf(
            s,
            n as usize,
            b"%d\x00" as *const u8 as *const libc::c_char,
            c,
        ) as size_t;
    };
}
unsafe extern "C" fn ts_subtree__write_dot_string(
    mut f: *mut FILE,
    mut string: *const libc::c_char,
) {
    let mut c: *const libc::c_char = string;
    while *c != 0 {
        if *c as libc::c_int == '\"' as i32 {
            fputs(b"\\\"\x00" as *const u8 as *const libc::c_char, f);
        } else if *c as libc::c_int == '\n' as i32 {
            fputs(b"\\n\x00" as *const u8 as *const libc::c_char, f);
        } else {
            fputc(*c as libc::c_int, f);
        }
        c = c.offset(1)
    }
}
static mut ROOT_FIELD: *const libc::c_char = b"__ROOT__\x00" as *const u8 as *const libc::c_char;
unsafe extern "C" fn ts_subtree__write_to_string(
    mut self_0: Subtree,
    mut string: *mut libc::c_char,
    mut limit: size_t,
    mut language: *const TSLanguage,
    mut include_all: bool,
    mut alias_symbol: TSSymbol,
    mut alias_is_named: bool,
    mut field_name: *const libc::c_char,
) -> size_t {
    if self_0.ptr.is_null() {
        return snprintf(
            string,
            limit as usize,
            b"(NULL)\x00" as *const u8 as *const libc::c_char,
        ) as size_t;
    }
    let mut cursor: *mut libc::c_char = string;
    let mut writer: *mut *mut libc::c_char = if limit > 0 as libc::c_int as libc::c_ulong {
        &mut cursor
    } else {
        &mut string
    };
    let mut is_root: bool = field_name == ROOT_FIELD;
    let mut is_visible: bool = include_all as libc::c_int != 0
        || ts_subtree_missing(self_0) as libc::c_int != 0
        || (if alias_symbol as libc::c_int != 0 {
            alias_is_named as libc::c_int
        } else {
            (ts_subtree_visible(self_0) as libc::c_int != 0
                && ts_subtree_named(self_0) as libc::c_int != 0) as libc::c_int
        }) != 0;
    if is_visible {
        if !is_root {
            cursor = cursor.offset(snprintf(
                *writer,
                limit as usize,
                b" \x00" as *const u8 as *const libc::c_char,
            ) as isize);
            if !field_name.is_null() {
                cursor = cursor.offset(snprintf(
                    *writer,
                    limit as usize,
                    b"%s: \x00" as *const u8 as *const libc::c_char,
                    field_name,
                ) as isize)
            }
        }
        if ts_subtree_is_error(self_0) as libc::c_int != 0
            && ts_subtree_child_count(self_0) == 0 as libc::c_int as libc::c_uint
            && (*self_0.ptr).size.bytes > 0 as libc::c_int as libc::c_uint
        {
            cursor = cursor.offset(snprintf(
                *writer,
                limit as usize,
                b"(UNEXPECTED \x00" as *const u8 as *const libc::c_char,
            ) as isize);
            cursor = cursor.offset(ts_subtree__write_char_to_string(
                *writer,
                limit,
                (*self_0.ptr).c2rust_unnamed.lookahead_char,
            ) as isize)
        } else {
            let mut symbol: TSSymbol = if alias_symbol as libc::c_int != 0 {
                alias_symbol as libc::c_int
            } else {
                ts_subtree_symbol(self_0) as libc::c_int
            } as TSSymbol;
            let mut symbol_name: *const libc::c_char = ts_language_symbol_name(language, symbol);
            if ts_subtree_missing(self_0) {
                cursor = cursor.offset(snprintf(
                    *writer,
                    limit as usize,
                    b"(MISSING \x00" as *const u8 as *const libc::c_char,
                ) as isize);
                if alias_is_named as libc::c_int != 0
                    || ts_subtree_named(self_0) as libc::c_int != 0
                {
                    cursor = cursor.offset(snprintf(
                        *writer,
                        limit as usize,
                        b"%s\x00" as *const u8 as *const libc::c_char,
                        symbol_name,
                    ) as isize)
                } else {
                    cursor = cursor.offset(snprintf(
                        *writer,
                        limit as usize,
                        b"\"%s\"\x00" as *const u8 as *const libc::c_char,
                        symbol_name,
                    ) as isize)
                }
            } else {
                cursor = cursor.offset(snprintf(
                    *writer,
                    limit as usize,
                    b"(%s\x00" as *const u8 as *const libc::c_char,
                    symbol_name,
                ) as isize)
            }
        }
    } else if is_root {
        let mut symbol_0: TSSymbol = ts_subtree_symbol(self_0);
        let mut symbol_name_0: *const libc::c_char = ts_language_symbol_name(language, symbol_0);
        cursor = cursor.offset(snprintf(
            *writer,
            limit as usize,
            b"(\"%s\")\x00" as *const u8 as *const libc::c_char,
            symbol_name_0,
        ) as isize)
    }
    if ts_subtree_child_count(self_0) != 0 {
        let mut alias_sequence: *const TSSymbol = ts_language_alias_sequence(
            language,
            (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.production_id as uint32_t,
        );
        let mut field_map: *const TSFieldMapEntry = 0 as *const TSFieldMapEntry;
        let mut field_map_end: *const TSFieldMapEntry = 0 as *const TSFieldMapEntry;
        ts_language_field_map(
            language,
            (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.production_id as uint32_t,
            &mut field_map,
            &mut field_map_end,
        );
        let mut structural_child_index: uint32_t = 0 as libc::c_int as uint32_t;
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        while i < (*self_0.ptr).child_count {
            let mut child: Subtree = *(*self_0.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .children
                .offset(i as isize);
            if ts_subtree_extra(child) {
                cursor = cursor.offset(ts_subtree__write_to_string(
                    child,
                    *writer,
                    limit,
                    language,
                    include_all,
                    0 as libc::c_int as TSSymbol,
                    0 as libc::c_int != 0,
                    0 as *const libc::c_char,
                ) as isize)
            } else {
                let mut alias_symbol_0: TSSymbol = if !alias_sequence.is_null() {
                    *alias_sequence.offset(structural_child_index as isize) as libc::c_int
                } else {
                    0 as libc::c_int
                } as TSSymbol;
                let mut alias_is_named_0: bool = if alias_symbol_0 as libc::c_int != 0 {
                    ts_language_symbol_metadata(language, alias_symbol_0).named() as libc::c_int
                } else {
                    0 as libc::c_int
                } != 0;
                let mut child_field_name: *const libc::c_char = if is_visible as libc::c_int != 0 {
                    0 as *const libc::c_char
                } else {
                    field_name
                };
                let mut i_0: *const TSFieldMapEntry = field_map;
                while i_0 < field_map_end {
                    if !(*i_0).inherited
                        && (*i_0).child_index as libc::c_uint == structural_child_index
                    {
                        child_field_name =
                            *(*language).field_names.offset((*i_0).field_id as isize);
                        break;
                    } else {
                        i_0 = i_0.offset(1)
                    }
                }
                cursor = cursor.offset(ts_subtree__write_to_string(
                    child,
                    *writer,
                    limit,
                    language,
                    include_all,
                    alias_symbol_0,
                    alias_is_named_0,
                    child_field_name,
                ) as isize);
                structural_child_index = structural_child_index.wrapping_add(1)
            }
            i = i.wrapping_add(1)
        }
    }
    if is_visible {
        cursor = cursor.offset(snprintf(
            *writer,
            limit as usize,
            b")\x00" as *const u8 as *const libc::c_char,
        ) as isize)
    }
    return cursor.wrapping_offset_from_(string) as libc::c_long as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_string(
    mut self_0: Subtree,
    mut language: *const TSLanguage,
    mut include_all: bool,
) -> *mut libc::c_char {
    let mut scratch_string: [libc::c_char; 1] = [0; 1];
    let mut size: size_t = ts_subtree__write_to_string(
        self_0,
        scratch_string.as_mut_ptr(),
        0 as libc::c_int as size_t,
        language,
        include_all,
        0 as libc::c_int as TSSymbol,
        0 as libc::c_int != 0,
        ROOT_FIELD,
    )
    .wrapping_add(1 as libc::c_int as libc::c_ulong);
    let mut result: *mut libc::c_char = malloc(
        size.wrapping_mul(::std::mem::size_of::<libc::c_char>() as libc::c_ulong) as libc::size_t,
    ) as *mut libc::c_char;
    ts_subtree__write_to_string(
        self_0,
        result,
        size,
        language,
        include_all,
        0 as libc::c_int as TSSymbol,
        0 as libc::c_int != 0,
        ROOT_FIELD,
    );
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree__print_dot_graph(
    mut self_0: *const Subtree,
    mut start_offset: uint32_t,
    mut language: *const TSLanguage,
    mut alias_symbol: TSSymbol,
    mut f: *mut FILE,
) {
    let mut subtree_symbol: TSSymbol = ts_subtree_symbol(*self_0);
    let mut symbol: TSSymbol = if alias_symbol as libc::c_int != 0 {
        alias_symbol as libc::c_int
    } else {
        subtree_symbol as libc::c_int
    } as TSSymbol;
    let mut end_offset: uint32_t = start_offset.wrapping_add(ts_subtree_total_bytes(*self_0));
    fprintf(
        f,
        b"tree_%p [label=\"\x00" as *const u8 as *const libc::c_char,
        self_0,
    );
    ts_subtree__write_dot_string(f, ts_language_symbol_name(language, symbol));
    fprintf(f, b"\"\x00" as *const u8 as *const libc::c_char);
    if ts_subtree_child_count(*self_0) == 0 as libc::c_int as libc::c_uint {
        fprintf(
            f,
            b", shape=plaintext\x00" as *const u8 as *const libc::c_char,
        );
    }
    if ts_subtree_extra(*self_0) {
        fprintf(
            f,
            b", fontcolor=gray\x00" as *const u8 as *const libc::c_char,
        );
    }
    fprintf(f,
            b", tooltip=\"range: %u - %u\nstate: %d\nerror-cost: %u\nhas-changes: %u\nrepeat-depth: %u\nlookahead-bytes: %u\x00"
                as *const u8 as *const libc::c_char, start_offset, end_offset,
            ts_subtree_parse_state(*self_0) as libc::c_int,
            ts_subtree_error_cost(*self_0),
            ts_subtree_has_changes(*self_0) as libc::c_int,
            ts_subtree_repeat_depth(*self_0),
            ts_subtree_lookahead_bytes(*self_0));
    if ts_subtree_is_error(*self_0) as libc::c_int != 0
        && ts_subtree_child_count(*self_0) == 0 as libc::c_int as libc::c_uint
    {
        fprintf(
            f,
            b"\ncharacter: \'%c\'\x00" as *const u8 as *const libc::c_char,
            (*(*self_0).ptr).c2rust_unnamed.lookahead_char,
        );
    }
    fprintf(f, b"\"]\n\x00" as *const u8 as *const libc::c_char);
    let mut child_start_offset: uint32_t = start_offset;
    let mut child_info_offset: uint32_t = ((*language).max_alias_sequence_length as libc::c_int
        * ts_subtree_production_id(*self_0) as libc::c_int)
        as uint32_t;
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    let mut n: uint32_t = ts_subtree_child_count(*self_0);
    while i < n {
        let mut child: *const Subtree = &mut *(*(*self_0).ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .children
            .offset(i as isize) as *mut Subtree;
        let mut alias_symbol_0: TSSymbol = 0 as libc::c_int as TSSymbol;
        if !ts_subtree_extra(*child) && child_info_offset != 0 {
            alias_symbol_0 = *(*language)
                .alias_sequences
                .offset(child_info_offset as isize);
            child_info_offset = child_info_offset.wrapping_add(1)
        }
        ts_subtree__print_dot_graph(child, child_start_offset, language, alias_symbol_0, f);
        fprintf(
            f,
            b"tree_%p -> tree_%p [tooltip=%u]\n\x00" as *const u8 as *const libc::c_char,
            self_0,
            child,
            i,
        );
        child_start_offset = (child_start_offset as libc::c_uint)
            .wrapping_add(ts_subtree_total_bytes(*child)) as uint32_t
            as uint32_t;
        i = i.wrapping_add(1)
    }
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_print_dot_graph(
    mut self_0: Subtree,
    mut language: *const TSLanguage,
    mut f: *mut FILE,
) {
    fprintf(
        f,
        b"digraph tree {\n\x00" as *const u8 as *const libc::c_char,
    );
    fprintf(
        f,
        b"edge [arrowhead=none]\n\x00" as *const u8 as *const libc::c_char,
    );
    ts_subtree__print_dot_graph(
        &mut self_0,
        0 as libc::c_int as uint32_t,
        language,
        0 as libc::c_int as TSSymbol,
        f,
    );
    fprintf(f, b"}\n\x00" as *const u8 as *const libc::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn ts_subtree_external_scanner_state_eq(
    mut self_0: Subtree,
    mut other: Subtree,
) -> bool {
    let mut state1: *const ExternalScannerState = &empty_state;
    let mut state2: *const ExternalScannerState = &empty_state;
    if !self_0.ptr.is_null()
        && ts_subtree_has_external_tokens(self_0) as libc::c_int != 0
        && (*self_0.ptr).child_count == 0
    {
        state1 = &(*self_0.ptr).c2rust_unnamed.external_scanner_state
    }
    if !other.ptr.is_null()
        && ts_subtree_has_external_tokens(other) as libc::c_int != 0
        && (*other.ptr).child_count == 0
    {
        state2 = &(*other.ptr).c2rust_unnamed.external_scanner_state
    }
    return ts_external_scanner_state_eq(state1, state2);
}
