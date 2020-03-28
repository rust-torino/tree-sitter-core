use crate::*;

use libc::{fprintf, FILE};

pub type StackActionType = libc::c_uint;

pub type StackStatus = libc::c_uint;
pub const StackStatusHalted: StackStatus = 2;
pub const StackStatusPaused: StackStatus = 1;
pub const StackStatusActive: StackStatus = 0;

pub type StackAction = libc::c_uint;
pub const StackActionNone: StackActionType = 0;
pub const StackActionStop: StackActionType = 1;
pub const StackActionPop: StackActionType = 2;
pub type StackCallback =
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: *const StackIterator) -> StackAction>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SummarizeStackSession {
    pub summary: *mut StackSummary,
    pub max_depth: libc::c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackIteratorArray {
    pub contents: *mut StackIterator,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackSliceArray {
    pub contents: *mut StackSlice,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackHeadArray {
    pub contents: *mut StackHead,
    pub size: uint32_t,
    pub capacity: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackNodeArray {
    pub contents: *mut *mut StackNode,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
pub type StackIterateCallback =
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: TSStateId, _: uint32_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackIterateSession {
    pub payload: *mut libc::c_void,
    pub callback: StackIterateCallback,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackNode {
    pub state: TSStateId,
    pub position: Length,
    pub links: [StackLink; 8],
    pub link_count: libc::c_ushort,
    pub ref_count: uint32_t,
    pub error_cost: libc::c_uint,
    pub node_count: libc::c_uint,
    pub dynamic_precedence: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackLink {
    pub node: *mut StackNode,
    pub subtree: Subtree,
    pub is_pending: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackSummaryEntry {
    pub position: Length,
    pub depth: libc::c_uint,
    pub state: TSStateId,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackSlice {
    pub subtrees: SubtreeArray,
    pub version: StackVersion,
}
pub type StackVersion = libc::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackSummary {
    pub contents: *mut StackSummaryEntry,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackHead {
    pub node: *mut StackNode,
    pub last_external_token: Subtree,
    pub summary: *mut StackSummary,
    pub node_count_at_last_error: libc::c_uint,
    pub lookahead_when_paused: TSSymbol,
    pub status: StackStatus,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Stack {
    pub heads: StackHeadArray,
    pub slices: StackSliceArray,
    pub iterators: StackIteratorArray,
    pub node_pool: StackNodeArray,
    pub base_node: *mut StackNode,
    pub subtree_pool: *mut SubtreePool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackIterator {
    pub node: *mut StackNode,
    pub subtrees: SubtreeArray,
    pub subtree_count: uint32_t,
    pub is_pending: bool,
}

unsafe extern "C" fn stack_node_retain(mut self_0: *mut StackNode) {
    if self_0.is_null() {
        return;
    }
    assert!((*self_0).ref_count > 0 as libc::c_int as libc::c_uint);
    (*self_0).ref_count = (*self_0).ref_count.wrapping_add(1);
    assert!((*self_0).ref_count != 0 as libc::c_int as libc::c_uint);
}
unsafe extern "C" fn stack_node_release(
    mut self_0: *mut StackNode,
    mut pool: *mut StackNodeArray,
    mut subtree_pool: *mut SubtreePool,
) {
    loop {
        assert!((*self_0).ref_count != 0 as libc::c_int as libc::c_uint);
        (*self_0).ref_count = (*self_0).ref_count.wrapping_sub(1);
        if (*self_0).ref_count > 0 as libc::c_int as libc::c_uint {
            return;
        }
        let mut first_predecessor: *mut StackNode = 0 as *mut StackNode;
        if (*self_0).link_count as libc::c_int > 0 as libc::c_int {
            let mut i: libc::c_uint =
                ((*self_0).link_count as libc::c_int - 1 as libc::c_int) as libc::c_uint;
            while i > 0 as libc::c_int as libc::c_uint {
                let mut link: StackLink = (*self_0).links[i as usize];
                if !link.subtree.ptr.is_null() {
                    ts_subtree_release(subtree_pool, link.subtree);
                }
                stack_node_release(link.node, pool, subtree_pool);
                i = i.wrapping_sub(1)
            }
            let mut link_0: StackLink = (*self_0).links[0 as libc::c_int as usize];
            if !link_0.subtree.ptr.is_null() {
                ts_subtree_release(subtree_pool, link_0.subtree);
            }
            first_predecessor = (*self_0).links[0 as libc::c_int as usize].node
        }
        if (*pool).size < 50 as libc::c_int as libc::c_uint {
            array__grow(
                pool as *mut VoidArray,
                1 as libc::c_int as size_t,
                ::std::mem::size_of::<*mut StackNode>() as libc::c_ulong,
            );
            let fresh0 = (*pool).size;
            (*pool).size = (*pool).size.wrapping_add(1);
            let ref mut fresh1 = *(*pool).contents.offset(fresh0 as isize);
            *fresh1 = self_0
        } else {
            ts_free(self_0 as *mut libc::c_void);
        }
        if first_predecessor.is_null() {
            break;
        }
        self_0 = first_predecessor
    }
}
unsafe extern "C" fn stack_node_new(
    mut previous_node: *mut StackNode,
    mut subtree: Subtree,
    mut is_pending: bool,
    mut state: TSStateId,
    mut pool: *mut StackNodeArray,
) -> *mut StackNode {
    let mut node: *mut StackNode = if (*pool).size > 0 as libc::c_int as libc::c_uint {
        (*pool).size = (*pool).size.wrapping_sub(1);
        *(*pool).contents.offset((*pool).size as isize) as *mut libc::c_void
    } else {
        ts_malloc(::std::mem::size_of::<StackNode>() as libc::c_ulong)
    } as *mut StackNode;
    *node = {
        let mut init = StackNode {
            state: state,
            position: Length {
                bytes: 0,
                extent: TSPoint { row: 0, column: 0 },
            },
            links: [StackLink {
                node: 0 as *mut StackNode,
                subtree: Subtree {
                    data: SubtreeInlineData {
                        is_inline_visible_named_extra_has_changes_is_missing_is_keyword: [0; 1],
                        symbol: 0,
                        padding_bytes: 0,
                        size_bytes: 0,
                        padding_columns: 0,
                        padding_rows_lookahead_bytes: [0; 1],
                        parse_state: 0,
                    },
                },
                is_pending: false,
            }; 8],
            link_count: 0 as libc::c_int as libc::c_ushort,
            ref_count: 1 as libc::c_int as uint32_t,
            error_cost: 0,
            node_count: 0,
            dynamic_precedence: 0,
        };
        init
    };
    if !previous_node.is_null() {
        (*node).link_count = 1 as libc::c_int as libc::c_ushort;
        (*node).links[0 as libc::c_int as usize] = {
            let mut init = StackLink {
                node: previous_node,
                subtree: subtree,
                is_pending: is_pending,
            };
            init
        };
        (*node).position = (*previous_node).position;
        (*node).error_cost = (*previous_node).error_cost;
        (*node).dynamic_precedence = (*previous_node).dynamic_precedence;
        (*node).node_count = (*previous_node).node_count;
        if !subtree.ptr.is_null() {
            (*node).error_cost = (*node)
                .error_cost
                .wrapping_add(ts_subtree_error_cost(subtree));
            (*node).position = length_add((*node).position, ts_subtree_total_size(subtree));
            (*node).node_count = (*node)
                .node_count
                .wrapping_add(ts_subtree_node_count(subtree));
            (*node).dynamic_precedence += ts_subtree_dynamic_precedence(subtree)
        }
    } else {
        (*node).position = length_zero();
        (*node).error_cost = 0 as libc::c_int as libc::c_uint
    }
    return node;
}
unsafe extern "C" fn stack__subtree_is_equivalent(mut left: Subtree, mut right: Subtree) -> bool {
    return left.ptr == right.ptr
        || !left.ptr.is_null()
            && !right.ptr.is_null()
            && ts_subtree_symbol(left) as libc::c_int == ts_subtree_symbol(right) as libc::c_int
            && (ts_subtree_error_cost(left) > 0 as libc::c_int as libc::c_uint
                && ts_subtree_error_cost(right) > 0 as libc::c_int as libc::c_uint
                || ts_subtree_padding(left).bytes == ts_subtree_padding(right).bytes
                    && ts_subtree_size(left).bytes == ts_subtree_size(right).bytes
                    && ts_subtree_child_count(left) == ts_subtree_child_count(right)
                    && ts_subtree_extra(left) as libc::c_int
                        == ts_subtree_extra(right) as libc::c_int
                    && ts_subtree_external_scanner_state_eq(left, right) as libc::c_int != 0);
}
unsafe extern "C" fn stack_node_add_link(
    mut self_0: *mut StackNode,
    mut link: StackLink,
    mut subtree_pool: *mut SubtreePool,
) {
    if link.node == self_0 {
        return;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*self_0).link_count as libc::c_int {
        let mut existing_link: *mut StackLink =
            &mut *(*self_0).links.as_mut_ptr().offset(i as isize) as *mut StackLink;
        if stack__subtree_is_equivalent((*existing_link).subtree, link.subtree) {
            // In general, we preserve ambiguities until they are removed from the stack
            // during a pop operation where multiple paths lead to the same node. But in
            // the special case where two links directly connect the same pair of nodes,
            // we can safely remove the ambiguity ahead of time without changing behavior.
            if (*existing_link).node == link.node {
                if ts_subtree_dynamic_precedence(link.subtree)
                    > ts_subtree_dynamic_precedence((*existing_link).subtree)
                {
                    ts_subtree_retain(link.subtree);
                    ts_subtree_release(subtree_pool, (*existing_link).subtree);
                    (*existing_link).subtree = link.subtree;
                    (*self_0).dynamic_precedence = (*link.node).dynamic_precedence
                        + ts_subtree_dynamic_precedence(link.subtree)
                }
                return;
            }
            // If the previous nodes are mergeable, merge them recursively.
            if (*(*existing_link).node).state as libc::c_int == (*link.node).state as libc::c_int
                && (*(*existing_link).node).position.bytes == (*link.node).position.bytes
            {
                let mut j: libc::c_int = 0 as libc::c_int;
                while j < (*link.node).link_count as libc::c_int {
                    stack_node_add_link(
                        (*existing_link).node,
                        (*link.node).links[j as usize],
                        subtree_pool,
                    );
                    j += 1
                }
                let mut dynamic_precedence: int32_t = (*link.node).dynamic_precedence;
                if !link.subtree.ptr.is_null() {
                    dynamic_precedence += ts_subtree_dynamic_precedence(link.subtree)
                }
                if dynamic_precedence > (*self_0).dynamic_precedence {
                    (*self_0).dynamic_precedence = dynamic_precedence
                }
                return;
            }
        }
        i += 1
    }
    if (*self_0).link_count as libc::c_int == 8 as libc::c_int {
        return;
    }
    stack_node_retain(link.node);
    let mut node_count: libc::c_uint = (*link.node).node_count;
    let mut dynamic_precedence_0: libc::c_int = (*link.node).dynamic_precedence;
    let fresh2 = (*self_0).link_count;
    (*self_0).link_count = (*self_0).link_count.wrapping_add(1);
    (*self_0).links[fresh2 as usize] = link;
    if !link.subtree.ptr.is_null() {
        ts_subtree_retain(link.subtree);
        node_count = node_count.wrapping_add(ts_subtree_node_count(link.subtree));
        dynamic_precedence_0 += ts_subtree_dynamic_precedence(link.subtree)
    }
    if node_count > (*self_0).node_count {
        (*self_0).node_count = node_count
    }
    if dynamic_precedence_0 > (*self_0).dynamic_precedence {
        (*self_0).dynamic_precedence = dynamic_precedence_0
    };
}
unsafe extern "C" fn stack_head_delete(
    mut self_0: *mut StackHead,
    mut pool: *mut StackNodeArray,
    mut subtree_pool: *mut SubtreePool,
) {
    if !(*self_0).node.is_null() {
        if !(*self_0).last_external_token.ptr.is_null() {
            ts_subtree_release(subtree_pool, (*self_0).last_external_token);
        }
        if !(*self_0).summary.is_null() {
            array__delete((*self_0).summary as *mut VoidArray);
            ts_free((*self_0).summary as *mut libc::c_void);
        }
        stack_node_release((*self_0).node, pool, subtree_pool);
    };
}
unsafe extern "C" fn ts_stack__add_version(
    mut self_0: *mut Stack,
    mut original_version: StackVersion,
    mut node: *mut StackNode,
) -> StackVersion {
    let mut head: StackHead = {
        let mut init = StackHead {
            node: node,
            last_external_token: (*(*self_0).heads.contents.offset(original_version as isize))
                .last_external_token,
            summary: 0 as *mut StackSummary,
            node_count_at_last_error: (*(*self_0).heads.contents.offset(original_version as isize))
                .node_count_at_last_error,
            lookahead_when_paused: 0 as libc::c_int as TSSymbol,
            status: StackStatusActive,
        };
        init
    };
    array__grow(
        &mut (*self_0).heads as *mut StackHeadArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackHead>() as libc::c_ulong,
    );
    let fresh3 = (*self_0).heads.size;
    (*self_0).heads.size = (*self_0).heads.size.wrapping_add(1);
    *(*self_0).heads.contents.offset(fresh3 as isize) = head;
    stack_node_retain(node);
    if !head.last_external_token.ptr.is_null() {
        ts_subtree_retain(head.last_external_token);
    }
    return (*self_0)
        .heads
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint);
}
unsafe extern "C" fn ts_stack__add_slice(
    mut self_0: *mut Stack,
    mut original_version: StackVersion,
    mut node: *mut StackNode,
    mut subtrees: *mut SubtreeArray,
) {
    let mut i: uint32_t = (*self_0)
        .slices
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint);
    while i.wrapping_add(1 as libc::c_int as libc::c_uint) > 0 as libc::c_int as libc::c_uint {
        let mut version: StackVersion = (*(*self_0).slices.contents.offset(i as isize)).version;
        if (*(*self_0).heads.contents.offset(version as isize)).node == node {
            let mut slice: StackSlice = {
                let mut init = StackSlice {
                    subtrees: *subtrees,
                    version: version,
                };
                init
            };
            array__splice(
                &mut (*self_0).slices as *mut StackSliceArray as *mut VoidArray,
                ::std::mem::size_of::<StackSlice>() as libc::c_ulong,
                i.wrapping_add(1 as libc::c_int as libc::c_uint),
                0 as libc::c_int as uint32_t,
                1 as libc::c_int as uint32_t,
                &mut slice as *mut StackSlice as *const libc::c_void,
            );
            return;
        }
        i = i.wrapping_sub(1)
    }
    let mut version_0: StackVersion = ts_stack__add_version(self_0, original_version, node);
    let mut slice_0: StackSlice = {
        let mut init = StackSlice {
            subtrees: *subtrees,
            version: version_0,
        };
        init
    };
    array__grow(
        &mut (*self_0).slices as *mut StackSliceArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackSlice>() as libc::c_ulong,
    );
    let fresh4 = (*self_0).slices.size;
    (*self_0).slices.size = (*self_0).slices.size.wrapping_add(1);
    *(*self_0).slices.contents.offset(fresh4 as isize) = slice_0;
}
#[inline(always)]
unsafe extern "C" fn stack__iter(
    mut self_0: *mut Stack,
    mut version: StackVersion,
    mut callback: StackCallback,
    mut payload: *mut libc::c_void,
    mut goal_subtree_count: libc::c_int,
) -> StackSliceArray {
    (*self_0).slices.size = 0 as libc::c_int as uint32_t;
    (*self_0).iterators.size = 0 as libc::c_int as uint32_t;
    assert!(version < (*self_0).heads.size);
    let mut head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    let mut iterator: StackIterator = {
        let mut init = StackIterator {
            node: (*head).node,
            subtrees: {
                let mut init = SubtreeArray {
                    contents: 0 as *mut Subtree,
                    size: 0 as libc::c_int as uint32_t,
                    capacity: 0 as libc::c_int as uint32_t,
                };
                init
            },
            subtree_count: 0 as libc::c_int as uint32_t,
            is_pending: 1 as libc::c_int != 0,
        };
        init
    };
    let mut include_subtrees: bool = 0 as libc::c_int != 0;
    if goal_subtree_count >= 0 as libc::c_int {
        include_subtrees = 1 as libc::c_int != 0;
        array__reserve(
            &mut iterator.subtrees as *mut SubtreeArray as *mut VoidArray,
            ::std::mem::size_of::<Subtree>() as libc::c_ulong,
            goal_subtree_count as uint32_t,
        );
    }
    array__grow(
        &mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackIterator>() as libc::c_ulong,
    );
    let fresh5 = (*self_0).iterators.size;
    (*self_0).iterators.size = (*self_0).iterators.size.wrapping_add(1);
    *(*self_0).iterators.contents.offset(fresh5 as isize) = iterator;
    while (*self_0).iterators.size > 0 as libc::c_int as libc::c_uint {
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        let mut size: uint32_t = (*self_0).iterators.size;
        while i < size {
            let mut iterator_0: *mut StackIterator =
                &mut *(*self_0).iterators.contents.offset(i as isize) as *mut StackIterator;
            let mut node: *mut StackNode = (*iterator_0).node;
            let mut action: StackAction =
                callback.expect("non-null function pointer")(payload, iterator_0);
            let mut should_pop: bool = action & StackActionPop as libc::c_int as libc::c_uint != 0;
            let mut should_stop: bool = action & StackActionStop as libc::c_int as libc::c_uint
                != 0
                || (*node).link_count as libc::c_int == 0 as libc::c_int;
            if should_pop {
                let mut subtrees: SubtreeArray = (*iterator_0).subtrees;
                if !should_stop {
                    ts_subtree_array_copy(subtrees, &mut subtrees);
                }
                ts_subtree_array_reverse(&mut subtrees);
                ts_stack__add_slice(self_0, version, node, &mut subtrees);
            }
            if should_stop {
                if !should_pop {
                    ts_subtree_array_delete((*self_0).subtree_pool, &mut (*iterator_0).subtrees);
                }
                array__erase(
                    &mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray,
                    ::std::mem::size_of::<StackIterator>() as libc::c_ulong,
                    i,
                );
                i = i.wrapping_sub(1);
                size = size.wrapping_sub(1)
            } else {
                let mut current_block_39: u64;
                let mut j: uint32_t = 1 as libc::c_int as uint32_t;
                while j <= (*node).link_count as libc::c_uint {
                    let mut next_iterator: *mut StackIterator = 0 as *mut StackIterator;
                    let mut link: StackLink = StackLink {
                        node: 0 as *mut StackNode,
                        subtree: Subtree {
                            data: SubtreeInlineData {
                                is_inline_visible_named_extra_has_changes_is_missing_is_keyword: [0;
                                    1],
                                symbol: 0,
                                padding_bytes: 0,
                                size_bytes: 0,
                                padding_columns: 0,
                                padding_rows_lookahead_bytes: [0; 1],
                                parse_state: 0,
                            },
                        },
                        is_pending: false,
                    };
                    if j == (*node).link_count as libc::c_uint {
                        link = (*node).links[0 as libc::c_int as usize];
                        next_iterator = &mut *(*self_0).iterators.contents.offset(i as isize)
                            as *mut StackIterator;
                        current_block_39 = 11048769245176032998;
                    } else if (*self_0).iterators.size >= 64 as libc::c_int as libc::c_uint {
                        current_block_39 = 7172762164747879670;
                    } else {
                        link = (*node).links[j as usize];
                        let mut current_iterator: StackIterator =
                            *(*self_0).iterators.contents.offset(i as isize);
                        array__grow(
                            &mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray,
                            1 as libc::c_int as size_t,
                            ::std::mem::size_of::<StackIterator>() as libc::c_ulong,
                        );
                        let fresh6 = (*self_0).iterators.size;
                        (*self_0).iterators.size = (*self_0).iterators.size.wrapping_add(1);
                        *(*self_0).iterators.contents.offset(fresh6 as isize) = current_iterator;
                        assert!((*self_0)
                            .iterators
                            .size
                            .wrapping_sub(1 as libc::c_int as libc::c_uint)
                            < (*self_0).iterators.size
                        );
                        next_iterator = &mut *(*self_0).iterators.contents.offset(
                            (*self_0)
                                .iterators
                                .size
                                .wrapping_sub(1 as libc::c_int as libc::c_uint)
                                as isize,
                        ) as *mut StackIterator;
                        ts_subtree_array_copy(
                            (*next_iterator).subtrees,
                            &mut (*next_iterator).subtrees,
                        );
                        current_block_39 = 11048769245176032998;
                    }
                    match current_block_39 {
                        11048769245176032998 => {
                            (*next_iterator).node = link.node;
                            if !link.subtree.ptr.is_null() {
                                if include_subtrees {
                                    array__grow(
                                        &mut (*next_iterator).subtrees as *mut SubtreeArray
                                            as *mut VoidArray,
                                        1 as libc::c_int as size_t,
                                        ::std::mem::size_of::<Subtree>() as libc::c_ulong,
                                    );
                                    let fresh7 = (*next_iterator).subtrees.size;
                                    (*next_iterator).subtrees.size =
                                        (*next_iterator).subtrees.size.wrapping_add(1);
                                    *(*next_iterator).subtrees.contents.offset(fresh7 as isize) =
                                        link.subtree;
                                    ts_subtree_retain(link.subtree);
                                }
                                if !ts_subtree_extra(link.subtree) {
                                    (*next_iterator).subtree_count =
                                        (*next_iterator).subtree_count.wrapping_add(1);
                                    if !link.is_pending {
                                        (*next_iterator).is_pending = 0 as libc::c_int != 0
                                    }
                                }
                            } else {
                                (*next_iterator).subtree_count =
                                    (*next_iterator).subtree_count.wrapping_add(1);
                                (*next_iterator).is_pending = 0 as libc::c_int != 0
                            }
                        }
                        _ => {}
                    }
                    j = j.wrapping_add(1)
                }
            }
            i = i.wrapping_add(1)
        }
    }
    return (*self_0).slices;
}
// Create a stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_new(mut subtree_pool: *mut SubtreePool) -> *mut Stack {
    let mut self_0: *mut Stack = ts_calloc(
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<Stack>() as libc::c_ulong,
    ) as *mut Stack;
    (*self_0).heads.size = 0 as libc::c_int as uint32_t;
    (*self_0).heads.capacity = 0 as libc::c_int as uint32_t;
    (*self_0).heads.contents = 0 as *mut StackHead;
    (*self_0).slices.size = 0 as libc::c_int as uint32_t;
    (*self_0).slices.capacity = 0 as libc::c_int as uint32_t;
    (*self_0).slices.contents = 0 as *mut StackSlice;
    (*self_0).iterators.size = 0 as libc::c_int as uint32_t;
    (*self_0).iterators.capacity = 0 as libc::c_int as uint32_t;
    (*self_0).iterators.contents = 0 as *mut StackIterator;
    (*self_0).node_pool.size = 0 as libc::c_int as uint32_t;
    (*self_0).node_pool.capacity = 0 as libc::c_int as uint32_t;
    (*self_0).node_pool.contents = 0 as *mut *mut StackNode;
    array__reserve(
        &mut (*self_0).heads as *mut StackHeadArray as *mut VoidArray,
        ::std::mem::size_of::<StackHead>() as libc::c_ulong,
        4 as libc::c_int as uint32_t,
    );
    array__reserve(
        &mut (*self_0).slices as *mut StackSliceArray as *mut VoidArray,
        ::std::mem::size_of::<StackSlice>() as libc::c_ulong,
        4 as libc::c_int as uint32_t,
    );
    array__reserve(
        &mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray,
        ::std::mem::size_of::<StackIterator>() as libc::c_ulong,
        4 as libc::c_int as uint32_t,
    );
    array__reserve(
        &mut (*self_0).node_pool as *mut StackNodeArray as *mut VoidArray,
        ::std::mem::size_of::<*mut StackNode>() as libc::c_ulong,
        50 as libc::c_int as uint32_t,
    );
    (*self_0).subtree_pool = subtree_pool;
    (*self_0).base_node = stack_node_new(
        0 as *mut StackNode,
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        },
        0 as libc::c_int != 0,
        1 as libc::c_int as TSStateId,
        &mut (*self_0).node_pool,
    );
    ts_stack_clear(self_0);
    return self_0;
}
// Release the memory reserved for a given stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_delete(mut self_0: *mut Stack) {
    if !(*self_0).slices.contents.is_null() {
        array__delete(&mut (*self_0).slices as *mut StackSliceArray as *mut VoidArray);
    }
    if !(*self_0).iterators.contents.is_null() {
        array__delete(&mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray);
    }
    stack_node_release(
        (*self_0).base_node,
        &mut (*self_0).node_pool,
        (*self_0).subtree_pool,
    );
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*self_0).heads.size {
        stack_head_delete(
            &mut *(*self_0).heads.contents.offset(i as isize),
            &mut (*self_0).node_pool,
            (*self_0).subtree_pool,
        );
        i = i.wrapping_add(1)
    }
    (*self_0).heads.size = 0 as libc::c_int as uint32_t;
    if !(*self_0).node_pool.contents.is_null() {
        let mut i_0: uint32_t = 0 as libc::c_int as uint32_t;
        while i_0 < (*self_0).node_pool.size {
            ts_free(*(*self_0).node_pool.contents.offset(i_0 as isize) as *mut libc::c_void);
            i_0 = i_0.wrapping_add(1)
        }
        array__delete(&mut (*self_0).node_pool as *mut StackNodeArray as *mut VoidArray);
    }
    array__delete(&mut (*self_0).heads as *mut StackHeadArray as *mut VoidArray);
    ts_free(self_0 as *mut libc::c_void);
}
// Get the stack's current number of versions.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_version_count(mut self_0: *const Stack) -> uint32_t {
    return (*self_0).heads.size;
}
// Get the state at the top of the given version of the stack. If the stack is
// empty, this returns the initial state, 0.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_state(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> TSStateId {
    assert!(version < (*self_0).heads.size);
    return (*(*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).node)
        .state;
}
// Get the position of the given version of the stack within the document.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_position(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> Length {
    assert!(version < (*self_0).heads.size);
    return (*(*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).node)
        .position;
}
// Get the last external token associated with a given version of the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_last_external_token(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> Subtree {
    assert!(version < (*self_0).heads.size);
    return (*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead))
        .last_external_token;
}
// Set the last external token associated with a given version of the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_set_last_external_token(
    mut self_0: *mut Stack,
    mut version: StackVersion,
    mut token: Subtree,
) {
    assert!(version < (*self_0).heads.size);
    let mut head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    if !token.ptr.is_null() {
        ts_subtree_retain(token);
    }
    if !(*head).last_external_token.ptr.is_null() {
        ts_subtree_release((*self_0).subtree_pool, (*head).last_external_token);
    }
    (*head).last_external_token = token;
}
// Get the total cost of all errors on the given version of the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_error_cost(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> libc::c_uint {
    assert!(version < (*self_0).heads.size);
    let mut head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    let mut result: libc::c_uint = (*(*head).node).error_cost;
    if (*head).status as libc::c_uint == StackStatusPaused as libc::c_int as libc::c_uint
        || (*(*head).node).state as libc::c_int == 0 as libc::c_int
            && (*(*head).node).links[0 as libc::c_int as usize]
                .subtree
                .ptr
                .is_null()
    {
        result = result.wrapping_add(500 as libc::c_int as libc::c_uint)
    }
    return result;
}
// Get the maximum number of tree nodes reachable from this version of the stack
// since the last error was detected.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_node_count_since_error(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> libc::c_uint {
    assert!(version < (*self_0).heads.size);
    let mut head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    if (*(*head).node).node_count < (*head).node_count_at_last_error {
        (*head).node_count_at_last_error = (*(*head).node).node_count
    }
    return (*(*head).node)
        .node_count
        .wrapping_sub((*head).node_count_at_last_error);
}
// Push a tree and state onto the given version of the stack.
//
// This transfers ownership of the tree to the Stack. Callers that
// need to retain ownership of the tree for their own purposes should
// first retain the tree.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_push(
    mut self_0: *mut Stack,
    mut version: StackVersion,
    mut subtree: Subtree,
    mut pending: bool,
    mut state: TSStateId,
) {
    assert!(version < (*self_0).heads.size);
    let mut head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    let mut new_node: *mut StackNode = stack_node_new(
        (*head).node,
        subtree,
        pending,
        state,
        &mut (*self_0).node_pool,
    );
    if subtree.ptr.is_null() {
        (*head).node_count_at_last_error = (*new_node).node_count
    }
    (*head).node = new_node;
}
#[inline(always)]
unsafe extern "C" fn iterate_callback(
    mut payload: *mut libc::c_void,
    mut iterator: *const StackIterator,
) -> StackAction {
    let mut session: *mut StackIterateSession = payload as *mut StackIterateSession;
    (*session).callback.expect("non-null function pointer")(
        (*session).payload,
        (*(*iterator).node).state,
        (*iterator).subtree_count,
    );
    return StackActionNone as libc::c_int as StackAction;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_iterate(
    mut self_0: *mut Stack,
    mut version: StackVersion,
    mut callback: StackIterateCallback,
    mut payload: *mut libc::c_void,
) {
    let mut session: StackIterateSession = {
        let mut init = StackIterateSession {
            payload: payload,
            callback: callback,
        };
        init
    };
    stack__iter(
        self_0,
        version,
        Some(
            iterate_callback
                as unsafe extern "C" fn(
                    _: *mut libc::c_void,
                    _: *const StackIterator,
                ) -> StackAction,
        ),
        &mut session as *mut StackIterateSession as *mut libc::c_void,
        -(1 as libc::c_int),
    );
}
#[inline(always)]
unsafe extern "C" fn pop_count_callback(
    mut payload: *mut libc::c_void,
    mut iterator: *const StackIterator,
) -> StackAction {
    let mut goal_subtree_count: *mut libc::c_uint = payload as *mut libc::c_uint;
    if (*iterator).subtree_count == *goal_subtree_count {
        return (StackActionPop as libc::c_int | StackActionStop as libc::c_int) as StackAction;
    } else {
        return StackActionNone as libc::c_int as StackAction;
    };
}
// Pop the given number of entries from the given version of the stack. This
// operation can increase the number of stack versions by revealing multiple
// versions which had previously been merged. It returns an array that
// specifies the index of each revealed version and the trees that were
// removed from that version.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_pop_count(
    mut self_0: *mut Stack,
    mut version: StackVersion,
    mut count: uint32_t,
) -> StackSliceArray {
    return stack__iter(
        self_0,
        version,
        Some(
            pop_count_callback
                as unsafe extern "C" fn(
                    _: *mut libc::c_void,
                    _: *const StackIterator,
                ) -> StackAction,
        ),
        &mut count as *mut uint32_t as *mut libc::c_void,
        count as libc::c_int,
    );
}
#[inline(always)]
unsafe extern "C" fn pop_pending_callback(
    mut _payload: *mut libc::c_void,
    mut iterator: *const StackIterator,
) -> StackAction {
    if (*iterator).subtree_count >= 1 as libc::c_int as libc::c_uint {
        if (*iterator).is_pending {
            return (StackActionPop as libc::c_int | StackActionStop as libc::c_int) as StackAction;
        } else {
            return StackActionStop as libc::c_int as StackAction;
        }
    } else {
        return StackActionNone as libc::c_int as StackAction;
    };
}
// Remove any pending trees from the top of the given version of the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_pop_pending(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) -> StackSliceArray {
    let mut pop: StackSliceArray = stack__iter(
        self_0,
        version,
        Some(
            pop_pending_callback
                as unsafe extern "C" fn(
                    _: *mut libc::c_void,
                    _: *const StackIterator,
                ) -> StackAction,
        ),
        0 as *mut libc::c_void,
        0 as libc::c_int,
    );
    if pop.size > 0 as libc::c_int as libc::c_uint {
        ts_stack_renumber_version(
            self_0,
            (*pop.contents.offset(0 as libc::c_int as isize)).version,
            version,
        );
        (*pop.contents.offset(0 as libc::c_int as isize)).version = version
    }
    return pop;
}
#[inline(always)]
unsafe extern "C" fn pop_error_callback(
    mut payload: *mut libc::c_void,
    mut iterator: *const StackIterator,
) -> StackAction {
    if (*iterator).subtrees.size > 0 as libc::c_int as libc::c_uint {
        let mut found_error: *mut bool = payload as *mut bool;
        if !*found_error
            && ts_subtree_is_error(
                *(*iterator)
                    .subtrees
                    .contents
                    .offset(0 as libc::c_int as isize),
            ) as libc::c_int
                != 0
        {
            *found_error = 1 as libc::c_int != 0;
            return (StackActionPop as libc::c_int | StackActionStop as libc::c_int) as StackAction;
        } else {
            return StackActionStop as libc::c_int as StackAction;
        }
    } else {
        return StackActionNone as libc::c_int as StackAction;
    };
}
// Remove an error at the top of the given version of the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_pop_error(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) -> SubtreeArray {
    assert!(version < (*self_0).heads.size);
    let mut node: *mut StackNode =
        (*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).node;
    let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    while i < (*node).link_count as libc::c_uint {
        if !(*node).links[i as usize].subtree.ptr.is_null()
            && ts_subtree_is_error((*node).links[i as usize].subtree) as libc::c_int != 0
        {
            let mut found_error: bool = 0 as libc::c_int != 0;
            let mut pop: StackSliceArray = stack__iter(
                self_0,
                version,
                Some(
                    pop_error_callback
                        as unsafe extern "C" fn(
                            _: *mut libc::c_void,
                            _: *const StackIterator,
                        ) -> StackAction,
                ),
                &mut found_error as *mut bool as *mut libc::c_void,
                1 as libc::c_int,
            );
            if pop.size > 0 as libc::c_int as libc::c_uint {
                assert!(pop.size == 1 as libc::c_int as libc::c_uint);
                ts_stack_renumber_version(
                    self_0,
                    (*pop.contents.offset(0 as libc::c_int as isize)).version,
                    version,
                );
                return (*pop.contents.offset(0 as libc::c_int as isize)).subtrees;
            }
            break;
        } else {
            i = i.wrapping_add(1)
        }
    }
    return {
        let mut init = SubtreeArray {
            contents: 0 as *mut Subtree,
            size: 0 as libc::c_int as uint32_t,
            capacity: 0,
        };
        init
    };
}
#[inline(always)]
unsafe extern "C" fn pop_all_callback(
    mut _payload: *mut libc::c_void,
    mut iterator: *const StackIterator,
) -> StackAction {
    return if (*(*iterator).node).link_count as libc::c_int == 0 as libc::c_int {
        StackActionPop as libc::c_int
    } else {
        StackActionNone as libc::c_int
    } as StackAction;
}
// Remove any all trees from the given version of the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_pop_all(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) -> StackSliceArray {
    return stack__iter(
        self_0,
        version,
        Some(
            pop_all_callback
                as unsafe extern "C" fn(
                    _: *mut libc::c_void,
                    _: *const StackIterator,
                ) -> StackAction,
        ),
        0 as *mut libc::c_void,
        0 as libc::c_int,
    );
}
#[inline(always)]
unsafe extern "C" fn summarize_stack_callback(
    mut payload: *mut libc::c_void,
    mut iterator: *const StackIterator,
) -> StackAction {
    let mut session: *mut SummarizeStackSession = payload as *mut SummarizeStackSession;
    let mut state: TSStateId = (*(*iterator).node).state;
    let mut depth: libc::c_uint = (*iterator).subtree_count;
    if depth > (*session).max_depth {
        return StackActionStop as libc::c_int as StackAction;
    }
    let mut i: libc::c_uint = (*(*session).summary)
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint);
    while i.wrapping_add(1 as libc::c_int as libc::c_uint) > 0 as libc::c_int as libc::c_uint {
        let mut entry: StackSummaryEntry = *(*(*session).summary).contents.offset(i as isize);
        if entry.depth < depth {
            break;
        }
        if entry.depth == depth && entry.state as libc::c_int == state as libc::c_int {
            return StackActionNone as libc::c_int as StackAction;
        }
        i = i.wrapping_sub(1)
    }
    array__grow(
        (*session).summary as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackSummaryEntry>() as libc::c_ulong,
    );
    let fresh8 = (*(*session).summary).size;
    (*(*session).summary).size = (*(*session).summary).size.wrapping_add(1);
    *(*(*session).summary).contents.offset(fresh8 as isize) = {
        let mut init = StackSummaryEntry {
            position: (*(*iterator).node).position,
            depth: depth,
            state: state,
        };
        init
    };
    return StackActionNone as libc::c_int as StackAction;
}
// Compute a summary of all the parse states near the top of the given
// version of the stack and store the summary for later retrieval.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_record_summary(
    mut self_0: *mut Stack,
    mut version: StackVersion,
    mut max_depth: libc::c_uint,
) {
    let mut session: SummarizeStackSession = {
        let mut init = SummarizeStackSession {
            summary: ts_malloc(::std::mem::size_of::<StackSummary>() as libc::c_ulong)
                as *mut StackSummary,
            max_depth: max_depth,
        };
        init
    };
    (*session.summary).size = 0 as libc::c_int as uint32_t;
    (*session.summary).capacity = 0 as libc::c_int as uint32_t;
    (*session.summary).contents = 0 as *mut StackSummaryEntry;
    stack__iter(
        self_0,
        version,
        Some(
            summarize_stack_callback
                as unsafe extern "C" fn(
                    _: *mut libc::c_void,
                    _: *const StackIterator,
                ) -> StackAction,
        ),
        &mut session as *mut SummarizeStackSession as *mut libc::c_void,
        -(1 as libc::c_int),
    );
    let ref mut fresh9 = (*(*self_0).heads.contents.offset(version as isize)).summary;
    *fresh9 = session.summary;
}
// Retrieve a summary of all the parse states near the top of the
// given version of the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_get_summary(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) -> *mut StackSummary {
    assert!(version < (*self_0).heads.size);

    return (*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).summary;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_dynamic_precedence(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) -> libc::c_int {
    assert!(version < (*self_0).heads.size);

    return (*(*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).node)
        .dynamic_precedence;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_has_advanced_since_error(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> bool {
    assert!(version < (*self_0).heads.size);

    let mut head: *const StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    let mut node: *const StackNode = (*head).node;
    if (*node).error_cost == 0 as libc::c_int as libc::c_uint {
        return 1 as libc::c_int != 0;
    }
    while !node.is_null() {
        if !((*node).link_count as libc::c_int > 0 as libc::c_int) {
            break;
        }
        let mut subtree: Subtree = (*node).links[0 as libc::c_int as usize].subtree;
        if subtree.ptr.is_null() {
            break;
        }
        if ts_subtree_total_bytes(subtree) > 0 as libc::c_int as libc::c_uint {
            return 1 as libc::c_int != 0;
        } else {
            if !((*node).node_count > (*head).node_count_at_last_error
                && ts_subtree_error_cost(subtree) == 0 as libc::c_int as libc::c_uint)
            {
                break;
            }
            node = (*node).links[0 as libc::c_int as usize].node
        }
    }
    return 0 as libc::c_int != 0;
}
// Remove the given version from the stack.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_remove_version(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) {
    assert!(version < (*self_0).heads.size);
    stack_head_delete(
        &mut *(*self_0).heads.contents.offset(version as isize),
        &mut (*self_0).node_pool,
        (*self_0).subtree_pool,
    );
    array__erase(
        &mut (*self_0).heads as *mut StackHeadArray as *mut VoidArray,
        ::std::mem::size_of::<StackHead>() as libc::c_ulong,
        version,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_renumber_version(
    mut self_0: *mut Stack,
    mut v1: StackVersion,
    mut v2: StackVersion,
) {
    if v1 == v2 {
        return;
    }
    assert!(v2 < v1);
    assert!(v1 < (*self_0).heads.size);
    let mut source_head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(v1 as isize) as *mut StackHead;
    let mut target_head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(v2 as isize) as *mut StackHead;
    if !(*target_head).summary.is_null() && (*source_head).summary.is_null() {
        (*source_head).summary = (*target_head).summary;
        (*target_head).summary = 0 as *mut StackSummary
    }
    stack_head_delete(
        target_head,
        &mut (*self_0).node_pool,
        (*self_0).subtree_pool,
    );
    *target_head = *source_head;
    array__erase(
        &mut (*self_0).heads as *mut StackHeadArray as *mut VoidArray,
        ::std::mem::size_of::<StackHead>() as libc::c_ulong,
        v1,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_swap_versions(
    mut self_0: *mut Stack,
    mut v1: StackVersion,
    mut v2: StackVersion,
) {
    let mut temporary_head: StackHead = *(*self_0).heads.contents.offset(v1 as isize);
    *(*self_0).heads.contents.offset(v1 as isize) = *(*self_0).heads.contents.offset(v2 as isize);
    *(*self_0).heads.contents.offset(v2 as isize) = temporary_head;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_copy_version(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) -> StackVersion {
    assert!(version < (*self_0).heads.size);
    array__grow(
        &mut (*self_0).heads as *mut StackHeadArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackHead>() as libc::c_ulong,
    );
    let fresh10 = (*self_0).heads.size;
    (*self_0).heads.size = (*self_0).heads.size.wrapping_add(1);
    *(*self_0).heads.contents.offset(fresh10 as isize) =
        *(*self_0).heads.contents.offset(version as isize);
    assert!((*self_0)
        .heads
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint)
        < (*self_0).heads.size
    );
    let mut head: *mut StackHead = &mut *(*self_0).heads.contents.offset(
        (*self_0)
            .heads
            .size
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
    ) as *mut StackHead;
    stack_node_retain((*head).node);
    if !(*head).last_external_token.ptr.is_null() {
        ts_subtree_retain((*head).last_external_token);
    }
    (*head).summary = 0 as *mut StackSummary;
    return (*self_0)
        .heads
        .size
        .wrapping_sub(1 as libc::c_int as libc::c_uint);
}
// Merge the given two stack versions if possible, returning true
// if they were successfully merged and false otherwise.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_merge(
    mut self_0: *mut Stack,
    mut version1: StackVersion,
    mut version2: StackVersion,
) -> bool {
    if !ts_stack_can_merge(self_0, version1, version2) {
        return 0 as libc::c_int != 0;
    }
    let mut head1: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version1 as isize) as *mut StackHead;
    let mut head2: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version2 as isize) as *mut StackHead;
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*(*head2).node).link_count as libc::c_uint {
        stack_node_add_link(
            (*head1).node,
            (*(*head2).node).links[i as usize],
            (*self_0).subtree_pool,
        );
        i = i.wrapping_add(1)
    }
    if (*(*head1).node).state as libc::c_int == 0 as libc::c_int {
        (*head1).node_count_at_last_error = (*(*head1).node).node_count
    }
    ts_stack_remove_version(self_0, version2);
    return 1 as libc::c_int != 0;
}
// Determine whether the given two stack versions can be merged.
#[no_mangle]
pub unsafe extern "C" fn ts_stack_can_merge(
    mut self_0: *mut Stack,
    mut version1: StackVersion,
    mut version2: StackVersion,
) -> bool {
    let mut head1: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version1 as isize) as *mut StackHead;
    let mut head2: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version2 as isize) as *mut StackHead;
    return (*head1).status as libc::c_uint == StackStatusActive as libc::c_int as libc::c_uint
        && (*head2).status as libc::c_uint == StackStatusActive as libc::c_int as libc::c_uint
        && (*(*head1).node).state as libc::c_int == (*(*head2).node).state as libc::c_int
        && (*(*head1).node).position.bytes == (*(*head2).node).position.bytes
        && (*(*head1).node).error_cost == (*(*head2).node).error_cost
        && ts_subtree_external_scanner_state_eq(
            (*head1).last_external_token,
            (*head2).last_external_token,
        ) as libc::c_int
            != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_halt(mut self_0: *mut Stack, mut version: StackVersion) {
    assert!(version < (*self_0).heads.size);
    (*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).status =
        StackStatusHalted;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_pause(
    mut self_0: *mut Stack,
    mut version: StackVersion,
    mut lookahead: TSSymbol,
) {
    assert!(version < (*self_0).heads.size);
    let mut head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    (*head).status = StackStatusPaused;
    (*head).lookahead_when_paused = lookahead;
    (*head).node_count_at_last_error = (*(*head).node).node_count;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_is_active(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> bool {
    assert!(version < (*self_0).heads.size);
    return (*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).status
        as libc::c_uint
        == StackStatusActive as libc::c_int as libc::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_is_halted(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> bool {
    assert!(version < (*self_0).heads.size);
    return (*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).status
        as libc::c_uint
        == StackStatusHalted as libc::c_int as libc::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_is_paused(
    mut self_0: *const Stack,
    mut version: StackVersion,
) -> bool {
    assert!(version < (*self_0).heads.size);
    return (*(&mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead)).status
        as libc::c_uint
        == StackStatusPaused as libc::c_int as libc::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_resume(
    mut self_0: *mut Stack,
    mut version: StackVersion,
) -> TSSymbol {
    assert!(version < (*self_0).heads.size);
    let mut head: *mut StackHead =
        &mut *(*self_0).heads.contents.offset(version as isize) as *mut StackHead;
    assert!((*head).status as libc::c_uint == StackStatusPaused as libc::c_int as libc::c_uint);
    let mut result: TSSymbol = (*head).lookahead_when_paused;
    (*head).status = StackStatusActive;
    (*head).lookahead_when_paused = 0 as libc::c_int as TSSymbol;
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_clear(mut self_0: *mut Stack) {
    stack_node_retain((*self_0).base_node);
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*self_0).heads.size {
        stack_head_delete(
            &mut *(*self_0).heads.contents.offset(i as isize),
            &mut (*self_0).node_pool,
            (*self_0).subtree_pool,
        );
        i = i.wrapping_add(1)
    }
    (*self_0).heads.size = 0 as libc::c_int as uint32_t;
    array__grow(
        &mut (*self_0).heads as *mut StackHeadArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<StackHead>() as libc::c_ulong,
    );
    let fresh11 = (*self_0).heads.size;
    (*self_0).heads.size = (*self_0).heads.size.wrapping_add(1);
    *(*self_0).heads.contents.offset(fresh11 as isize) = {
        let mut init = StackHead {
            node: (*self_0).base_node,
            last_external_token: Subtree {
                ptr: 0 as *const SubtreeHeapData,
            },
            summary: 0 as *mut StackSummary,
            node_count_at_last_error: 0,
            lookahead_when_paused: 0 as libc::c_int as TSSymbol,
            status: StackStatusActive,
        };
        init
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_stack_print_dot_graph(
    mut self_0: *mut Stack,
    mut language: *const TSLanguage,
    mut f: *mut FILE,
) -> bool {
    array__reserve(
        &mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray,
        ::std::mem::size_of::<StackIterator>() as libc::c_ulong,
        32 as libc::c_int as uint32_t,
    );
    let mut was_recording_allocations: bool = ts_toggle_allocation_recording(0 as libc::c_int != 0);
    if f.is_null() {
        f = stderr
    }
    fprintf(
        f,
        b"digraph stack {\n\x00" as *const u8 as *const libc::c_char,
    );
    fprintf(
        f,
        b"rankdir=\"RL\";\n\x00" as *const u8 as *const libc::c_char,
    );
    fprintf(
        f,
        b"edge [arrowhead=none]\n\x00" as *const u8 as *const libc::c_char,
    );
    let mut visited_nodes: StackNodeArray = {
        let mut init = StackNodeArray {
            contents: 0 as *mut *mut StackNode,
            size: 0 as libc::c_int as uint32_t,
            capacity: 0 as libc::c_int as uint32_t,
        };
        init
    };
    (*self_0).iterators.size = 0 as libc::c_int as uint32_t;
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*self_0).heads.size {
        let mut head: *mut StackHead =
            &mut *(*self_0).heads.contents.offset(i as isize) as *mut StackHead;
        if !((*head).status as libc::c_uint == StackStatusHalted as libc::c_int as libc::c_uint) {
            fprintf(
                f,
                b"node_head_%u [shape=none, label=\"\"]\n\x00" as *const u8 as *const libc::c_char,
                i,
            );
            fprintf(
                f,
                b"node_head_%u -> node_%p [\x00" as *const u8 as *const libc::c_char,
                i,
                (*head).node,
            );
            if (*head).status as libc::c_uint == StackStatusPaused as libc::c_int as libc::c_uint {
                fprintf(f, b"color=red \x00" as *const u8 as *const libc::c_char);
            }
            fprintf(f,
                    b"label=%u, fontcolor=blue, weight=10000, labeltooltip=\"node_count: %u\nerror_cost: %u\x00"
                        as *const u8 as *const libc::c_char, i,
                    ts_stack_node_count_since_error(self_0, i),
                    ts_stack_error_cost(self_0, i));
            if !(*head).last_external_token.ptr.is_null() {
                let mut state: *const ExternalScannerState = &(*(*head).last_external_token.ptr)
                    .c2rust_unnamed
                    .external_scanner_state;
                let mut data: *const libc::c_char = ts_external_scanner_state_data(state);
                fprintf(
                    f,
                    b"\nexternal_scanner_state:\x00" as *const u8 as *const libc::c_char,
                );
                let mut j: uint32_t = 0 as libc::c_int as uint32_t;
                while j < (*state).length {
                    fprintf(
                        f,
                        b" %2X\x00" as *const u8 as *const libc::c_char,
                        *data.offset(j as isize) as libc::c_int,
                    );
                    j = j.wrapping_add(1)
                }
            }
            fprintf(f, b"\"]\n\x00" as *const u8 as *const libc::c_char);
            array__grow(
                &mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray,
                1 as libc::c_int as size_t,
                ::std::mem::size_of::<StackIterator>() as libc::c_ulong,
            );
            let fresh12 = (*self_0).iterators.size;
            (*self_0).iterators.size = (*self_0).iterators.size.wrapping_add(1);
            *(*self_0).iterators.contents.offset(fresh12 as isize) = {
                let mut init = StackIterator {
                    node: (*head).node,
                    subtrees: SubtreeArray {
                        contents: 0 as *mut Subtree,
                        size: 0,
                        capacity: 0,
                    },
                    subtree_count: 0,
                    is_pending: false,
                };
                init
            }
        }
        i = i.wrapping_add(1)
    }
    let mut all_iterators_done: bool = 0 as libc::c_int != 0;
    while !all_iterators_done {
        all_iterators_done = 1 as libc::c_int != 0;
        let mut i_0: uint32_t = 0 as libc::c_int as uint32_t;
        while i_0 < (*self_0).iterators.size {
            let mut iterator: StackIterator = *(*self_0).iterators.contents.offset(i_0 as isize);
            let mut node: *mut StackNode = iterator.node;
            let mut j_0: uint32_t = 0 as libc::c_int as uint32_t;
            while j_0 < visited_nodes.size {
                if *visited_nodes.contents.offset(j_0 as isize) == node {
                    node = 0 as *mut StackNode;
                    break;
                } else {
                    j_0 = j_0.wrapping_add(1)
                }
            }
            if !node.is_null() {
                all_iterators_done = 0 as libc::c_int != 0;
                fprintf(
                    f,
                    b"node_%p [\x00" as *const u8 as *const libc::c_char,
                    node,
                );
                if (*node).state as libc::c_int == 0 as libc::c_int {
                    fprintf(f, b"label=\"?\"\x00" as *const u8 as *const libc::c_char);
                } else if (*node).link_count as libc::c_int == 1 as libc::c_int
                    && !(*node).links[0 as libc::c_int as usize]
                        .subtree
                        .ptr
                        .is_null()
                    && ts_subtree_extra((*node).links[0 as libc::c_int as usize].subtree)
                        as libc::c_int
                        != 0
                {
                    fprintf(
                        f,
                        b"shape=point margin=0 label=\"\"\x00" as *const u8 as *const libc::c_char,
                    );
                } else {
                    fprintf(
                        f,
                        b"label=\"%d\"\x00" as *const u8 as *const libc::c_char,
                        (*node).state as libc::c_int,
                    );
                }
                fprintf(f,
                        b" tooltip=\"position: %u,%u\nnode_count:%u\nerror_cost: %u\ndynamic_precedence: %d\"];\n\x00"
                            as *const u8 as *const libc::c_char,
                        (*node).position.extent.row.wrapping_add(1 as
                                                                     libc::c_int
                                                                     as
                                                                     libc::c_uint),
                        (*node).position.extent.column, (*node).node_count,
                        (*node).error_cost, (*node).dynamic_precedence);
                let mut j_1: libc::c_int = 0 as libc::c_int;
                while j_1 < (*node).link_count as libc::c_int {
                    let mut link: StackLink = (*node).links[j_1 as usize];
                    fprintf(
                        f,
                        b"node_%p -> node_%p [\x00" as *const u8 as *const libc::c_char,
                        node,
                        link.node,
                    );
                    if link.is_pending {
                        fprintf(f, b"style=dashed \x00" as *const u8 as *const libc::c_char);
                    }
                    if !link.subtree.ptr.is_null()
                        && ts_subtree_extra(link.subtree) as libc::c_int != 0
                    {
                        fprintf(
                            f,
                            b"fontcolor=gray \x00" as *const u8 as *const libc::c_char,
                        );
                    }
                    if link.subtree.ptr.is_null() {
                        fprintf(f, b"color=red\x00" as *const u8 as *const libc::c_char);
                    } else {
                        fprintf(f, b"label=\"\x00" as *const u8 as *const libc::c_char);
                        let mut quoted: bool = ts_subtree_visible(link.subtree) as libc::c_int != 0
                            && !ts_subtree_named(link.subtree);
                        if quoted {
                            fprintf(f, b"\'\x00" as *const u8 as *const libc::c_char);
                        }
                        let mut name: *const libc::c_char =
                            ts_language_symbol_name(language, ts_subtree_symbol(link.subtree));
                        let mut c: *const libc::c_char = name;
                        while *c != 0 {
                            if *c as libc::c_int == '\"' as i32 || *c as libc::c_int == '\\' as i32
                            {
                                fprintf(f, b"\\\x00" as *const u8 as *const libc::c_char);
                            }
                            fprintf(
                                f,
                                b"%c\x00" as *const u8 as *const libc::c_char,
                                *c as libc::c_int,
                            );
                            c = c.offset(1)
                        }
                        if quoted {
                            fprintf(f, b"\'\x00" as *const u8 as *const libc::c_char);
                        }
                        fprintf(f, b"\"\x00" as *const u8 as *const libc::c_char);
                        fprintf(
                            f,
                            b"labeltooltip=\"error_cost: %u\ndynamic_precedence: %u\"\x00"
                                as *const u8 as *const libc::c_char,
                            ts_subtree_error_cost(link.subtree),
                            ts_subtree_dynamic_precedence(link.subtree),
                        );
                    }
                    fprintf(f, b"];\n\x00" as *const u8 as *const libc::c_char);
                    let mut next_iterator: *mut StackIterator = 0 as *mut StackIterator;
                    if j_1 == 0 as libc::c_int {
                        next_iterator = &mut *(*self_0).iterators.contents.offset(i_0 as isize)
                            as *mut StackIterator
                    } else {
                        array__grow(
                            &mut (*self_0).iterators as *mut StackIteratorArray as *mut VoidArray,
                            1 as libc::c_int as size_t,
                            ::std::mem::size_of::<StackIterator>() as libc::c_ulong,
                        );
                        let fresh13 = (*self_0).iterators.size;
                        (*self_0).iterators.size = (*self_0).iterators.size.wrapping_add(1);
                        *(*self_0).iterators.contents.offset(fresh13 as isize) = iterator;
                        assert!((*self_0)
                            .iterators
                            .size
                            .wrapping_sub(1 as libc::c_int as libc::c_uint)
                            < (*self_0).iterators.size
                        );
                        next_iterator = &mut *(*self_0).iterators.contents.offset(
                            (*self_0)
                                .iterators
                                .size
                                .wrapping_sub(1 as libc::c_int as libc::c_uint)
                                as isize,
                        ) as *mut StackIterator
                    }
                    (*next_iterator).node = link.node;
                    j_1 += 1
                }
                array__grow(
                    &mut visited_nodes as *mut StackNodeArray as *mut VoidArray,
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<*mut StackNode>() as libc::c_ulong,
                );
                let fresh14 = visited_nodes.size;
                visited_nodes.size = visited_nodes.size.wrapping_add(1);
                let ref mut fresh15 = *visited_nodes.contents.offset(fresh14 as isize);
                *fresh15 = node
            }
            i_0 = i_0.wrapping_add(1)
        }
    }
    fprintf(f, b"}\n\x00" as *const u8 as *const libc::c_char);
    array__delete(&mut visited_nodes as *mut StackNodeArray as *mut VoidArray);
    ts_toggle_allocation_recording(was_recording_allocations);
    return 1 as libc::c_int != 0;
}
