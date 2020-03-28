use crate::*;

use libc::{fclose, fdopen, fprintf, fputc, fputs, memcmp, snprintf, FILE};

static mut MAX_VERSION_COUNT: libc::c_uint = 6 as libc::c_int as libc::c_uint;
static mut MAX_VERSION_COUNT_OVERFLOW: libc::c_uint = 4 as libc::c_int as libc::c_uint;
static mut MAX_SUMMARY_DEPTH: libc::c_uint = 16 as libc::c_int as libc::c_uint;
static mut MAX_COST_DIFFERENCE: libc::c_uint =
    (16 as libc::c_int * 100 as libc::c_int) as libc::c_uint;
static mut OP_COUNT_PER_TIMEOUT_CHECK: libc::c_uint = 100 as libc::c_int as libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TokenCache {
    pub token: Subtree,
    pub last_external_token: Subtree,
    pub byte_index: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSParser {
    pub lexer: Lexer,
    pub stack: *mut Stack,
    pub tree_pool: SubtreePool,
    pub language: *const TSLanguage,
    pub reduce_actions: ReduceActionSet,
    pub finished_tree: Subtree,
    pub scratch_tree_data: SubtreeHeapData,
    pub scratch_tree: MutableSubtree,
    pub token_cache: TokenCache,
    pub reusable_node: ReusableNode,
    pub external_scanner_payload: *mut libc::c_void,
    pub dot_graph_file: *mut FILE,
    pub end_clock: TSClock,
    pub timeout_duration: TSDuration,
    pub accept_count: libc::c_uint,
    pub operation_count: libc::c_uint,
    pub cancellation_flag: *const size_t,
    pub old_tree: Subtree,
    pub included_range_differences: TSRangeArray,
    pub included_range_difference_index: libc::c_uint,
}

pub const ErrorComparisonTakeRight: ErrorComparison = 4;
pub const ErrorComparisonPreferRight: ErrorComparison = 3;
pub const ErrorComparisonNone: ErrorComparison = 2;
pub const ErrorComparisonPreferLeft: ErrorComparison = 1;
pub const ErrorComparisonTakeLeft: ErrorComparison = 0;
pub type ErrorComparison = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ErrorStatus {
    pub cost: libc::c_uint,
    pub node_count: libc::c_uint,
    pub dynamic_precedence: libc::c_int,
    pub is_in_error: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSStringInput {
    pub string: *const libc::c_char,
    pub length: uint32_t,
}

// StringInput
unsafe extern "C" fn ts_string_input_read(
    mut _self: *mut libc::c_void,
    mut byte: uint32_t,
    mut _id: TSPoint,
    mut length: *mut uint32_t,
) -> *const libc::c_char {
    let mut self_0: *mut TSStringInput = _self as *mut TSStringInput;
    if byte >= (*self_0).length {
        *length = 0 as libc::c_int as uint32_t;
        return b"\x00" as *const u8 as *const libc::c_char;
    } else {
        *length = (*self_0).length.wrapping_sub(byte);
        return (*self_0).string.offset(byte as isize);
    };
}
// Parser - Private
unsafe extern "C" fn ts_parser__log(mut self_0: *mut TSParser) {
    if (*self_0).lexer.logger.log.is_some() {
        (*self_0)
            .lexer
            .logger
            .log
            .expect("non-null function pointer")(
            (*self_0).lexer.logger.payload,
            TSLogTypeParse,
            (*self_0).lexer.debug_buffer.as_mut_ptr(),
        );
    }
    if !(*self_0).dot_graph_file.is_null() {
        fprintf(
            (*self_0).dot_graph_file,
            b"graph {\nlabel=\"\x00" as *const u8 as *const libc::c_char,
        );
        let mut c: *mut libc::c_char = &mut *(*self_0)
            .lexer
            .debug_buffer
            .as_mut_ptr()
            .offset(0 as libc::c_int as isize)
            as *mut libc::c_char;
        while *c as libc::c_int != 0 as libc::c_int {
            if *c as libc::c_int == '\"' as i32 {
                fputc('\\' as i32, (*self_0).dot_graph_file);
            }
            fputc(*c as libc::c_int, (*self_0).dot_graph_file);
            c = c.offset(1)
        }
        fprintf(
            (*self_0).dot_graph_file,
            b"\"\n}\n\n\x00" as *const u8 as *const libc::c_char,
        );
    };
}
unsafe extern "C" fn ts_parser__breakdown_top_of_stack(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
) -> bool {
    let mut did_break_down: bool = 0 as libc::c_int != 0;
    let mut pending: bool = 0 as libc::c_int != 0;
    loop {
        let mut pop: StackSliceArray = ts_stack_pop_pending((*self_0).stack, version);
        if pop.size == 0 {
            break;
        }
        did_break_down = 1 as libc::c_int != 0;
        pending = 0 as libc::c_int != 0;
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        while i < pop.size {
            let mut slice: StackSlice = *pop.contents.offset(i as isize);
            let mut state: TSStateId = ts_stack_state((*self_0).stack, slice.version);
            assert!((0 as libc::c_int as uint32_t) < slice.subtrees.size);
            let mut parent: Subtree =
                *(&mut *slice.subtrees.contents.offset(0 as libc::c_int as isize) as *mut Subtree);
            let mut j: uint32_t = 0 as libc::c_int as uint32_t;
            let mut n: uint32_t = ts_subtree_child_count(parent);
            while j < n {
                let mut child: Subtree = *(*parent.ptr)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .children
                    .offset(j as isize);
                pending = ts_subtree_child_count(child) > 0 as libc::c_int as libc::c_uint;
                if ts_subtree_is_error(child) {
                    state = 0 as libc::c_int as TSStateId
                } else if !ts_subtree_extra(child) {
                    state =
                        ts_language_next_state((*self_0).language, state, ts_subtree_symbol(child))
                }
                ts_subtree_retain(child);
                ts_stack_push((*self_0).stack, slice.version, child, pending, state);
                j = j.wrapping_add(1)
            }
            let mut j_0: uint32_t = 1 as libc::c_int as uint32_t;
            while j_0 < slice.subtrees.size {
                let mut tree: Subtree = *slice.subtrees.contents.offset(j_0 as isize);
                ts_stack_push(
                    (*self_0).stack,
                    slice.version,
                    tree,
                    0 as libc::c_int != 0,
                    state,
                );
                j_0 = j_0.wrapping_add(1)
            }
            ts_subtree_release(&mut (*self_0).tree_pool, parent);
            array__delete(&mut slice.subtrees as *mut SubtreeArray as *mut VoidArray);
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"breakdown_top_of_stack tree:%s\x00" as *const u8 as *const libc::c_char,
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(parent)),
                );
                ts_parser__log(self_0);
            }
            if !(*self_0).dot_graph_file.is_null() {
                ts_stack_print_dot_graph(
                    (*self_0).stack,
                    (*self_0).language,
                    (*self_0).dot_graph_file,
                );
                fputs(
                    b"\n\n\x00" as *const u8 as *const libc::c_char,
                    (*self_0).dot_graph_file,
                );
            }
            i = i.wrapping_add(1)
        }
        if !pending {
            break;
        }
    }
    return did_break_down;
}
unsafe extern "C" fn ts_parser__breakdown_lookahead(
    mut self_0: *mut TSParser,
    mut lookahead: *mut Subtree,
    mut state: TSStateId,
    mut reusable_node: *mut ReusableNode,
) {
    let mut did_descend: bool = 0 as libc::c_int != 0;
    let mut tree: Subtree = reusable_node_tree(reusable_node);
    while ts_subtree_child_count(tree) > 0 as libc::c_int as libc::c_uint
        && ts_subtree_parse_state(tree) as libc::c_int != state as libc::c_int
    {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"state_mismatch sym:%s\x00" as *const u8 as *const libc::c_char,
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(tree)),
            );
            ts_parser__log(self_0);
        }
        reusable_node_descend(reusable_node);
        tree = reusable_node_tree(reusable_node);
        did_descend = 1 as libc::c_int != 0
    }
    if did_descend {
        ts_subtree_release(&mut (*self_0).tree_pool, *lookahead);
        *lookahead = tree;
        ts_subtree_retain(*lookahead);
    };
}
unsafe extern "C" fn ts_parser__compare_versions(
    mut _self_0: *mut TSParser,
    mut a: ErrorStatus,
    mut b: ErrorStatus,
) -> ErrorComparison {
    if !a.is_in_error && b.is_in_error as libc::c_int != 0 {
        if a.cost < b.cost {
            return ErrorComparisonTakeLeft;
        } else {
            return ErrorComparisonPreferLeft;
        }
    }
    if a.is_in_error as libc::c_int != 0 && !b.is_in_error {
        if b.cost < a.cost {
            return ErrorComparisonTakeRight;
        } else {
            return ErrorComparisonPreferRight;
        }
    }
    if a.cost < b.cost {
        if b.cost
            .wrapping_sub(a.cost)
            .wrapping_mul((1 as libc::c_int as libc::c_uint).wrapping_add(a.node_count))
            > MAX_COST_DIFFERENCE
        {
            return ErrorComparisonTakeLeft;
        } else {
            return ErrorComparisonPreferLeft;
        }
    }
    if b.cost < a.cost {
        if a.cost
            .wrapping_sub(b.cost)
            .wrapping_mul((1 as libc::c_int as libc::c_uint).wrapping_add(b.node_count))
            > MAX_COST_DIFFERENCE
        {
            return ErrorComparisonTakeRight;
        } else {
            return ErrorComparisonPreferRight;
        }
    }
    if a.dynamic_precedence > b.dynamic_precedence {
        return ErrorComparisonPreferLeft;
    }
    if b.dynamic_precedence > a.dynamic_precedence {
        return ErrorComparisonPreferRight;
    }
    return ErrorComparisonNone;
}
unsafe extern "C" fn ts_parser__version_status(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
) -> ErrorStatus {
    let mut cost: libc::c_uint = ts_stack_error_cost((*self_0).stack, version);
    let mut is_paused: bool = ts_stack_is_paused((*self_0).stack, version);
    if is_paused {
        cost = cost.wrapping_add(100 as libc::c_int as libc::c_uint)
    }
    return {
        let mut init = ErrorStatus {
            cost: cost,
            node_count: ts_stack_node_count_since_error((*self_0).stack, version),
            dynamic_precedence: ts_stack_dynamic_precedence((*self_0).stack, version),
            is_in_error: is_paused as libc::c_int != 0
                || ts_stack_state((*self_0).stack, version) as libc::c_int == 0 as libc::c_int,
        };
        init
    };
}
unsafe extern "C" fn ts_parser__better_version_exists(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut is_in_error: bool,
    mut cost: libc::c_uint,
) -> bool {
    if !(*self_0).finished_tree.ptr.is_null()
        && ts_subtree_error_cost((*self_0).finished_tree) <= cost
    {
        return 1 as libc::c_int != 0;
    }
    let mut position: Length = ts_stack_position((*self_0).stack, version);
    let mut status: ErrorStatus = {
        let mut init = ErrorStatus {
            cost: cost,
            node_count: ts_stack_node_count_since_error((*self_0).stack, version),
            dynamic_precedence: ts_stack_dynamic_precedence((*self_0).stack, version),
            is_in_error: is_in_error,
        };
        init
    };
    let mut i: StackVersion = 0 as libc::c_int as StackVersion;
    let mut n: StackVersion = ts_stack_version_count((*self_0).stack);
    while i < n {
        if !(i == version
            || !ts_stack_is_active((*self_0).stack, i)
            || ts_stack_position((*self_0).stack, i).bytes < position.bytes)
        {
            let mut status_i: ErrorStatus = ts_parser__version_status(self_0, i);
            match ts_parser__compare_versions(self_0, status, status_i) as libc::c_uint {
                4 => return 1 as libc::c_int != 0,
                3 => {
                    if ts_stack_can_merge((*self_0).stack, i, version) {
                        return 1 as libc::c_int != 0;
                    }
                }
                _ => {}
            }
        }
        i = i.wrapping_add(1)
    }
    return 0 as libc::c_int != 0;
}
unsafe extern "C" fn ts_parser__restore_external_scanner(
    mut self_0: *mut TSParser,
    mut external_token: Subtree,
) {
    if !external_token.ptr.is_null() {
        (*(*self_0).language)
            .external_scanner
            .deserialize
            .expect("non-null function pointer")(
            (*self_0).external_scanner_payload,
            ts_external_scanner_state_data(
                &(*external_token.ptr).c2rust_unnamed.external_scanner_state,
            ),
            (*external_token.ptr)
                .c2rust_unnamed
                .external_scanner_state
                .length,
        );
    } else {
        (*(*self_0).language)
            .external_scanner
            .deserialize
            .expect("non-null function pointer")(
            (*self_0).external_scanner_payload,
            0 as *const libc::c_char,
            0 as libc::c_int as libc::c_uint,
        );
    };
}
unsafe extern "C" fn ts_parser__can_reuse_first_leaf(
    mut self_0: *mut TSParser,
    mut state: TSStateId,
    mut tree: Subtree,
    mut table_entry: *mut TableEntry,
) -> bool {
    let mut current_lex_mode: TSLexMode = *(*(*self_0).language).lex_modes.offset(state as isize);
    let mut leaf_symbol: TSSymbol = ts_subtree_leaf_symbol(tree);
    let mut leaf_state: TSStateId = ts_subtree_leaf_parse_state(tree);
    let mut leaf_lex_mode: TSLexMode = *(*(*self_0).language).lex_modes.offset(leaf_state as isize);
    // At the end of a non-terminal extra node, the lexer normally returns
    // NULL, which indicates that the parser should look for a reduce action
    // at symbol `0`. Avoid reusing tokens in this situation to ensure that
    // the same thing happens when incrementally reparsing.
    if current_lex_mode.lex_state as libc::c_int == -(1 as libc::c_int) as uint16_t as libc::c_int {
        return 0 as libc::c_int != 0;
    }
    // If the token was created in a state with the same set of lookaheads, it is reusable.
    if (*table_entry).action_count > 0 as libc::c_int as libc::c_uint
        && memcmp(
            &mut leaf_lex_mode as *mut TSLexMode as *const libc::c_void,
            &mut current_lex_mode as *mut TSLexMode as *const libc::c_void,
            ::std::mem::size_of::<TSLexMode>(),
        ) == 0 as libc::c_int
        && (leaf_symbol as libc::c_int
            != (*(*self_0).language).keyword_capture_token as libc::c_int
            || !ts_subtree_is_keyword(tree)
                && ts_subtree_parse_state(tree) as libc::c_int == state as libc::c_int)
    {
        return 1 as libc::c_int != 0;
    }
    // Empty tokens are not reusable in states with different lookaheads.
    if ts_subtree_size(tree).bytes == 0 as libc::c_int as libc::c_uint
        && leaf_symbol as libc::c_int != 0 as libc::c_int
    {
        return 0 as libc::c_int != 0;
    }
    // If the current state allows external tokens or other tokens that conflict with this
    // token, this token is not reusable.
    return current_lex_mode.external_lex_state as libc::c_int == 0 as libc::c_int
        && (*table_entry).is_reusable as libc::c_int != 0;
}
unsafe extern "C" fn ts_parser__lex(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut parse_state: TSStateId,
) -> Subtree {
    let mut start_position: Length = ts_stack_position((*self_0).stack, version);
    let mut external_token: Subtree = ts_stack_last_external_token((*self_0).stack, version);
    let mut lex_mode: TSLexMode = *(*(*self_0).language).lex_modes.offset(parse_state as isize);
    if lex_mode.lex_state as libc::c_int == -(1 as libc::c_int) as uint16_t as libc::c_int {
        return Subtree {
            ptr: 0 as *const SubtreeHeapData,
        };
    }
    let mut valid_external_tokens: *const bool = ts_language_enabled_external_tokens(
        (*self_0).language,
        lex_mode.external_lex_state as libc::c_uint,
    );
    let mut found_external_token: bool = 0 as libc::c_int != 0;
    let mut error_mode: bool = parse_state as libc::c_int == 0 as libc::c_int;
    let mut skipped_error: bool = 0 as libc::c_int != 0;
    let mut first_error_character: int32_t = 0 as libc::c_int;
    let mut error_start_position: Length = length_zero();
    let mut error_end_position: Length = length_zero();
    let mut lookahead_end_byte: uint32_t = 0 as libc::c_int as uint32_t;
    ts_lexer_reset(&mut (*self_0).lexer, start_position);
    loop {
        let mut current_position: Length = (*self_0).lexer.current_position;
        if !valid_external_tokens.is_null() {
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"lex_external state:%d, row:%u, column:%u\x00" as *const u8
                        as *const libc::c_char,
                    lex_mode.external_lex_state as libc::c_int,
                    current_position
                        .extent
                        .row
                        .wrapping_add(1 as libc::c_int as libc::c_uint),
                    current_position.extent.column,
                );
                ts_parser__log(self_0);
            }
            ts_lexer_start(&mut (*self_0).lexer);
            ts_parser__restore_external_scanner(self_0, external_token);
            let mut found_token: bool = (*(*self_0).language)
                .external_scanner
                .scan
                .expect("non-null function pointer")(
                (*self_0).external_scanner_payload,
                &mut (*self_0).lexer.data,
                valid_external_tokens,
            );
            ts_lexer_finish(&mut (*self_0).lexer, &mut lookahead_end_byte);
            // Zero-length external tokens are generally allowed, but they're not
            // allowed right after a syntax error. This is for two reasons:
            // 1. After a syntax error, the lexer is looking for any possible token,
            //    as opposed to the specific set of tokens that are valid in some
            //    parse state. In this situation, it's very easy for an external
            //    scanner to produce unwanted zero-length tokens.
            // 2. The parser sometimes inserts *missing* tokens to recover from
            //    errors. These tokens are also zero-length. If we allow more
            //    zero-length tokens to be created after missing tokens, it
            //    can lead to infinite loops. Forbidding zero-length tokens
            //    right at the point of error recovery is a conservative strategy
            //    for preventing this kind of infinite loop.
            if found_token as libc::c_int != 0
                && ((*self_0).lexer.token_end_position.bytes > current_position.bytes
                    || !error_mode
                        && ts_stack_has_advanced_since_error((*self_0).stack, version)
                            as libc::c_int
                            != 0)
            {
                found_external_token = 1 as libc::c_int != 0;
                break;
            } else {
                ts_lexer_reset(&mut (*self_0).lexer, current_position);
            }
        }
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"lex_internal state:%d, row:%u, column:%u\x00" as *const u8 as *const libc::c_char,
                lex_mode.lex_state as libc::c_int,
                current_position
                    .extent
                    .row
                    .wrapping_add(1 as libc::c_int as libc::c_uint),
                current_position.extent.column,
            );
            ts_parser__log(self_0);
        }
        ts_lexer_start(&mut (*self_0).lexer);
        let mut found_token_0: bool = (*(*self_0).language)
            .lex_fn
            .expect("non-null function pointer")(
            &mut (*self_0).lexer.data, lex_mode.lex_state
        );
        ts_lexer_finish(&mut (*self_0).lexer, &mut lookahead_end_byte);
        if found_token_0 {
            break;
        }
        if !error_mode {
            error_mode = 1 as libc::c_int != 0;
            lex_mode = *(*(*self_0).language)
                .lex_modes
                .offset(0 as libc::c_int as isize);
            valid_external_tokens = ts_language_enabled_external_tokens(
                (*self_0).language,
                lex_mode.external_lex_state as libc::c_uint,
            );
            ts_lexer_reset(&mut (*self_0).lexer, start_position);
        } else {
            if !skipped_error {
                if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                    snprintf(
                        (*self_0).lexer.debug_buffer.as_mut_ptr(),
                        1024,
                        b"skip_unrecognized_character\x00" as *const u8 as *const libc::c_char,
                    );
                    ts_parser__log(self_0);
                }
                skipped_error = 1 as libc::c_int != 0;
                error_start_position = (*self_0).lexer.token_start_position;
                error_end_position = (*self_0).lexer.token_start_position;
                first_error_character = (*self_0).lexer.data.lookahead
            }
            if (*self_0).lexer.current_position.bytes == error_end_position.bytes {
                if (*self_0).lexer.data.eof.expect("non-null function pointer")(
                    &mut (*self_0).lexer.data,
                ) {
                    (*self_0).lexer.data.result_symbol = -(1 as libc::c_int) as TSSymbol;
                    break;
                } else {
                    (*self_0)
                        .lexer
                        .data
                        .advance
                        .expect("non-null function pointer")(
                        &mut (*self_0).lexer.data,
                        0 as libc::c_int != 0,
                    );
                }
            }
            error_end_position = (*self_0).lexer.current_position
        }
    }
    let mut result: Subtree = Subtree {
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
    if skipped_error {
        let mut padding: Length = length_sub(error_start_position, start_position);
        let mut size: Length = length_sub(error_end_position, error_start_position);
        let mut lookahead_bytes: uint32_t =
            lookahead_end_byte.wrapping_sub(error_end_position.bytes);
        result = ts_subtree_new_error(
            &mut (*self_0).tree_pool,
            first_error_character,
            padding,
            size,
            lookahead_bytes,
            parse_state,
            (*self_0).language,
        );
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"lexed_lookahead sym:%s, size:%u, character:\'%c\'\x00" as *const u8
                    as *const libc::c_char,
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                ts_subtree_total_size(result).bytes,
                first_error_character,
            );
            ts_parser__log(self_0);
        }
    } else {
        if (*self_0).lexer.token_end_position.bytes < (*self_0).lexer.token_start_position.bytes {
            (*self_0).lexer.token_start_position = (*self_0).lexer.token_end_position
        }
        let mut is_keyword: bool = 0 as libc::c_int != 0;
        let mut symbol: TSSymbol = (*self_0).lexer.data.result_symbol;
        let mut padding_0: Length =
            length_sub((*self_0).lexer.token_start_position, start_position);
        let mut size_0: Length = length_sub(
            (*self_0).lexer.token_end_position,
            (*self_0).lexer.token_start_position,
        );
        let mut lookahead_bytes_0: uint32_t =
            lookahead_end_byte.wrapping_sub((*self_0).lexer.token_end_position.bytes);
        if found_external_token {
            symbol = *(*(*self_0).language)
                .external_scanner
                .symbol_map
                .offset(symbol as isize)
        } else if symbol as libc::c_int
            == (*(*self_0).language).keyword_capture_token as libc::c_int
            && symbol as libc::c_int != 0 as libc::c_int
        {
            let mut end_byte: uint32_t = (*self_0).lexer.token_end_position.bytes;
            ts_lexer_reset(&mut (*self_0).lexer, (*self_0).lexer.token_start_position);
            ts_lexer_start(&mut (*self_0).lexer);
            if (*(*self_0).language)
                .keyword_lex_fn
                .expect("non-null function pointer")(
                &mut (*self_0).lexer.data,
                0 as libc::c_int as TSStateId,
            ) as libc::c_int
                != 0
                && (*self_0).lexer.token_end_position.bytes == end_byte
                && ts_language_has_actions(
                    (*self_0).language,
                    parse_state,
                    (*self_0).lexer.data.result_symbol,
                ) as libc::c_int
                    != 0
            {
                is_keyword = 1 as libc::c_int != 0;
                symbol = (*self_0).lexer.data.result_symbol
            }
        }
        result = ts_subtree_new_leaf(
            &mut (*self_0).tree_pool,
            symbol,
            padding_0,
            size_0,
            lookahead_bytes_0,
            parse_state,
            found_external_token,
            is_keyword,
            (*self_0).language,
        );
        if found_external_token {
            let mut length: libc::c_uint = (*(*self_0).language)
                .external_scanner
                .serialize
                .expect("non-null function pointer")(
                (*self_0).external_scanner_payload,
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
            );
            ts_external_scanner_state_init(
                &mut (*(result.ptr as *mut SubtreeHeapData))
                    .c2rust_unnamed
                    .external_scanner_state,
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                length,
            );
        }
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"lexed_lookahead sym:%s, size:%u\x00" as *const u8 as *const libc::c_char,
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                ts_subtree_total_size(result).bytes,
            );
            ts_parser__log(self_0);
        }
    }
    return result;
}
unsafe extern "C" fn ts_parser__get_cached_token(
    mut self_0: *mut TSParser,
    mut state: TSStateId,
    mut position: size_t,
    mut last_external_token: Subtree,
    mut table_entry: *mut TableEntry,
) -> Subtree {
    let mut cache: *mut TokenCache = &mut (*self_0).token_cache;
    if !(*cache).token.ptr.is_null()
        && (*cache).byte_index as libc::c_ulong == position
        && ts_subtree_external_scanner_state_eq((*cache).last_external_token, last_external_token)
            as libc::c_int
            != 0
    {
        ts_language_table_entry(
            (*self_0).language,
            state,
            ts_subtree_symbol((*cache).token),
            table_entry,
        );
        if ts_parser__can_reuse_first_leaf(self_0, state, (*cache).token, table_entry) {
            ts_subtree_retain((*cache).token);
            return (*cache).token;
        }
    }
    return Subtree {
        ptr: 0 as *const SubtreeHeapData,
    };
}
unsafe extern "C" fn ts_parser__set_cached_token(
    mut self_0: *mut TSParser,
    mut byte_index: size_t,
    mut last_external_token: Subtree,
    mut token: Subtree,
) {
    let mut cache: *mut TokenCache = &mut (*self_0).token_cache;
    if !token.ptr.is_null() {
        ts_subtree_retain(token);
    }
    if !last_external_token.ptr.is_null() {
        ts_subtree_retain(last_external_token);
    }
    if !(*cache).token.ptr.is_null() {
        ts_subtree_release(&mut (*self_0).tree_pool, (*cache).token);
    }
    if !(*cache).last_external_token.ptr.is_null() {
        ts_subtree_release(&mut (*self_0).tree_pool, (*cache).last_external_token);
    }
    (*cache).token = token;
    (*cache).byte_index = byte_index as uint32_t;
    (*cache).last_external_token = last_external_token;
}
unsafe extern "C" fn ts_parser__has_included_range_difference(
    mut self_0: *const TSParser,
    mut start_position: uint32_t,
    mut end_position: uint32_t,
) -> bool {
    return ts_range_array_intersects(
        &(*self_0).included_range_differences,
        (*self_0).included_range_difference_index,
        start_position,
        end_position,
    );
}
unsafe extern "C" fn ts_parser__reuse_node(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut state: *mut TSStateId,
    mut position: uint32_t,
    mut last_external_token: Subtree,
    mut table_entry: *mut TableEntry,
) -> Subtree {
    let mut result: Subtree = Subtree {
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
    loop {
        result = reusable_node_tree(&mut (*self_0).reusable_node);
        if result.ptr.is_null() {
            break;
        }
        let mut byte_offset: uint32_t = reusable_node_byte_offset(&mut (*self_0).reusable_node);
        let mut end_byte_offset: uint32_t =
            byte_offset.wrapping_add(ts_subtree_total_bytes(result));
        // Do not reuse an EOF node if the included ranges array has changes
        // later on in the file.
        if ts_subtree_is_eof(result) {
            end_byte_offset = 4294967295 as libc::c_uint
        }
        if byte_offset > position {
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"before_reusable_node symbol:%s\x00" as *const u8 as *const libc::c_char,
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                );
                ts_parser__log(self_0);
            }
            break;
        } else if byte_offset < position {
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"past_reusable_node symbol:%s\x00" as *const u8 as *const libc::c_char,
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                );
                ts_parser__log(self_0);
            }
            if end_byte_offset <= position || !reusable_node_descend(&mut (*self_0).reusable_node) {
                reusable_node_advance(&mut (*self_0).reusable_node);
            }
        } else if !ts_subtree_external_scanner_state_eq(
            (*self_0).reusable_node.last_external_token,
            last_external_token,
        ) {
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"reusable_node_has_different_external_scanner_state symbol:%s\x00" as *const u8
                        as *const libc::c_char,
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                );
                ts_parser__log(self_0);
            }
            reusable_node_advance(&mut (*self_0).reusable_node);
        } else {
            let mut reason: *const libc::c_char = 0 as *const libc::c_char;
            if ts_subtree_has_changes(result) {
                reason = b"has_changes\x00" as *const u8 as *const libc::c_char
            } else if ts_subtree_is_error(result) {
                reason = b"is_error\x00" as *const u8 as *const libc::c_char
            } else if ts_subtree_missing(result) {
                reason = b"is_missing\x00" as *const u8 as *const libc::c_char
            } else if ts_subtree_is_fragile(result) {
                reason = b"is_fragile\x00" as *const u8 as *const libc::c_char
            } else if ts_parser__has_included_range_difference(self_0, byte_offset, end_byte_offset)
            {
                reason =
                    b"contains_different_included_range\x00" as *const u8 as *const libc::c_char
            }
            if !reason.is_null() {
                if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                    snprintf(
                        (*self_0).lexer.debug_buffer.as_mut_ptr(),
                        1024,
                        b"cant_reuse_node_%s tree:%s\x00" as *const u8 as *const libc::c_char,
                        reason,
                        ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                    );
                    ts_parser__log(self_0);
                }
                if !reusable_node_descend(&mut (*self_0).reusable_node) {
                    reusable_node_advance(&mut (*self_0).reusable_node);
                    ts_parser__breakdown_top_of_stack(self_0, version);
                    *state = ts_stack_state((*self_0).stack, version)
                }
            } else {
                let mut leaf_symbol: TSSymbol = ts_subtree_leaf_symbol(result);
                ts_language_table_entry((*self_0).language, *state, leaf_symbol, table_entry);
                if !ts_parser__can_reuse_first_leaf(self_0, *state, result, table_entry) {
                    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                        snprintf(
                            (*self_0).lexer.debug_buffer.as_mut_ptr(),
                            1024,
                            b"cant_reuse_node symbol:%s, first_leaf_symbol:%s\x00" as *const u8
                                as *const libc::c_char,
                            ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                            ts_language_symbol_name((*self_0).language, leaf_symbol),
                        );
                        ts_parser__log(self_0);
                    }
                    reusable_node_advance_past_leaf(&mut (*self_0).reusable_node);
                    break;
                } else {
                    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                        snprintf(
                            (*self_0).lexer.debug_buffer.as_mut_ptr(),
                            1024,
                            b"reuse_node symbol:%s\x00" as *const u8 as *const libc::c_char,
                            ts_language_symbol_name((*self_0).language, ts_subtree_symbol(result)),
                        );
                        ts_parser__log(self_0);
                    }
                    ts_subtree_retain(result);
                    return result;
                }
            }
        }
    }
    return Subtree {
        ptr: 0 as *const SubtreeHeapData,
    };
}
unsafe extern "C" fn ts_parser__select_tree(
    mut self_0: *mut TSParser,
    mut left: Subtree,
    mut right: Subtree,
) -> bool {
    if left.ptr.is_null() {
        return 1 as libc::c_int != 0;
    }
    if right.ptr.is_null() {
        return 0 as libc::c_int != 0;
    }
    if ts_subtree_error_cost(right) < ts_subtree_error_cost(left) {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"select_smaller_error symbol:%s, over_symbol:%s\x00" as *const u8
                    as *const libc::c_char,
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(right)),
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(left)),
            );
            ts_parser__log(self_0);
        }
        return 1 as libc::c_int != 0;
    }
    if ts_subtree_error_cost(left) < ts_subtree_error_cost(right) {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"select_smaller_error symbol:%s, over_symbol:%s\x00" as *const u8
                    as *const libc::c_char,
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(left)),
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(right)),
            );
            ts_parser__log(self_0);
        }
        return 0 as libc::c_int != 0;
    }
    if ts_subtree_dynamic_precedence(right) > ts_subtree_dynamic_precedence(left) {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"select_higher_precedence symbol:%s, prec:%u, over_symbol:%s, other_prec:%u\x00"
                    as *const u8 as *const libc::c_char,
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(right)),
                ts_subtree_dynamic_precedence(right),
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(left)),
                ts_subtree_dynamic_precedence(left),
            );
            ts_parser__log(self_0);
        }
        return 1 as libc::c_int != 0;
    }
    if ts_subtree_dynamic_precedence(left) > ts_subtree_dynamic_precedence(right) {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"select_higher_precedence symbol:%s, prec:%u, over_symbol:%s, other_prec:%u\x00"
                    as *const u8 as *const libc::c_char,
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(left)),
                ts_subtree_dynamic_precedence(left),
                ts_language_symbol_name((*self_0).language, ts_subtree_symbol(right)),
                ts_subtree_dynamic_precedence(right),
            );
            ts_parser__log(self_0);
        }
        return 0 as libc::c_int != 0;
    }
    if ts_subtree_error_cost(left) > 0 as libc::c_int as libc::c_uint {
        return 1 as libc::c_int != 0;
    }
    let mut comparison: libc::c_int = ts_subtree_compare(left, right);
    match comparison {
        -1 => {
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"select_earlier symbol:%s, over_symbol:%s\x00" as *const u8
                        as *const libc::c_char,
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(left)),
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(right)),
                );
                ts_parser__log(self_0);
            }
            return 0 as libc::c_int != 0;
        }
        1 => {
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"select_earlier symbol:%s, over_symbol:%s\x00" as *const u8
                        as *const libc::c_char,
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(right)),
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(left)),
                );
                ts_parser__log(self_0);
            }
            return 1 as libc::c_int != 0;
        }
        _ => {
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"select_existing symbol:%s, over_symbol:%s\x00" as *const u8
                        as *const libc::c_char,
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(left)),
                    ts_language_symbol_name((*self_0).language, ts_subtree_symbol(right)),
                );
                ts_parser__log(self_0);
            }
            return 0 as libc::c_int != 0;
        }
    };
}
unsafe extern "C" fn ts_parser__shift(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut state: TSStateId,
    mut lookahead: Subtree,
    mut extra: bool,
) {
    let mut subtree_to_push: Subtree = Subtree {
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
    if extra as libc::c_int != ts_subtree_extra(lookahead) as libc::c_int {
        let mut result: MutableSubtree = ts_subtree_make_mut(&mut (*self_0).tree_pool, lookahead);
        ts_subtree_set_extra(&mut result);
        subtree_to_push = ts_subtree_from_mut(result)
    } else {
        subtree_to_push = lookahead
    }
    let mut is_pending: bool =
        ts_subtree_child_count(subtree_to_push) > 0 as libc::c_int as libc::c_uint;
    ts_stack_push((*self_0).stack, version, subtree_to_push, is_pending, state);
    if ts_subtree_has_external_tokens(subtree_to_push) {
        ts_stack_set_last_external_token(
            (*self_0).stack,
            version,
            ts_subtree_last_external_token(subtree_to_push),
        );
    };
}
unsafe extern "C" fn ts_parser__replace_children(
    mut self_0: *mut TSParser,
    mut tree: *mut MutableSubtree,
    mut children: *mut SubtreeArray,
) -> bool {
    *(*self_0).scratch_tree.ptr = *(*tree).ptr;
    (*(*self_0).scratch_tree.ptr).child_count = 0 as libc::c_int as uint32_t;
    ts_subtree_set_children(
        (*self_0).scratch_tree,
        (*children).contents,
        (*children).size,
        (*self_0).language,
    );
    if ts_parser__select_tree(
        self_0,
        ts_subtree_from_mut(*tree),
        ts_subtree_from_mut((*self_0).scratch_tree),
    ) {
        *(*tree).ptr = *(*self_0).scratch_tree.ptr;
        return 1 as libc::c_int != 0;
    } else {
        return 0 as libc::c_int != 0;
    };
}
unsafe extern "C" fn ts_parser__reduce(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut symbol: TSSymbol,
    mut count: uint32_t,
    mut dynamic_precedence: libc::c_int,
    mut production_id: uint16_t,
    mut is_fragile: bool,
    mut is_extra: bool,
) -> StackVersion {
    let mut initial_version_count: uint32_t = ts_stack_version_count((*self_0).stack);
    let mut removed_version_count: uint32_t = 0 as libc::c_int as uint32_t;
    let mut pop: StackSliceArray = ts_stack_pop_count((*self_0).stack, version, count);
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < pop.size {
        let mut slice: StackSlice = *pop.contents.offset(i as isize);
        let mut slice_version: StackVersion = slice.version.wrapping_sub(removed_version_count);
        // Error recovery can sometimes cause lots of stack versions to merge,
        // such that a single pop operation can produce a lots of slices.
        // Avoid creating too many stack versions in that situation.
        if i > 0 as libc::c_int as libc::c_uint
            && slice_version > MAX_VERSION_COUNT.wrapping_add(MAX_VERSION_COUNT_OVERFLOW)
        {
            ts_stack_remove_version((*self_0).stack, slice_version);
            ts_subtree_array_delete(&mut (*self_0).tree_pool, &mut slice.subtrees);
            removed_version_count = removed_version_count.wrapping_add(1);
            while i.wrapping_add(1 as libc::c_int as libc::c_uint) < pop.size {
                let mut next_slice: StackSlice = *pop
                    .contents
                    .offset(i.wrapping_add(1 as libc::c_int as libc::c_uint) as isize);
                if next_slice.version != slice.version {
                    break;
                }
                ts_subtree_array_delete(&mut (*self_0).tree_pool, &mut next_slice.subtrees);
                i = i.wrapping_add(1)
            }
        } else {
            // Extra tokens on top of the stack should not be included in this new parent
            // node. They will be re-pushed onto the stack after the parent node is
            // created and pushed.
            let mut children: SubtreeArray = slice.subtrees;
            while children.size > 0 as libc::c_int as libc::c_uint
                && ts_subtree_extra(
                    *children.contents.offset(
                        children.size.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
                    ),
                ) as libc::c_int
                    != 0
            {
                children.size = children.size.wrapping_sub(1)
            }
            let mut parent: MutableSubtree = ts_subtree_new_node(
                &mut (*self_0).tree_pool,
                symbol,
                &mut children,
                production_id as libc::c_uint,
                (*self_0).language,
            );
            // This pop operation may have caused multiple stack versions to collapse
            // into one, because they all diverged from a common state. In that case,
            // choose one of the arrays of trees to be the parent node's children, and
            // delete the rest of the tree arrays.
            while i.wrapping_add(1 as libc::c_int as libc::c_uint) < pop.size {
                let mut next_slice_0: StackSlice = *pop
                    .contents
                    .offset(i.wrapping_add(1 as libc::c_int as libc::c_uint) as isize);
                if next_slice_0.version != slice.version {
                    break;
                }
                i = i.wrapping_add(1);
                let mut children_0: SubtreeArray = next_slice_0.subtrees;
                while children_0.size > 0 as libc::c_int as libc::c_uint
                    && ts_subtree_extra(
                        *children_0.contents.offset(
                            children_0
                                .size
                                .wrapping_sub(1 as libc::c_int as libc::c_uint)
                                as isize,
                        ),
                    ) as libc::c_int
                        != 0
                {
                    children_0.size = children_0.size.wrapping_sub(1)
                }
                if ts_parser__replace_children(self_0, &mut parent, &mut children_0) {
                    ts_subtree_array_delete(&mut (*self_0).tree_pool, &mut slice.subtrees);
                    slice = next_slice_0
                } else {
                    ts_subtree_array_delete(&mut (*self_0).tree_pool, &mut next_slice_0.subtrees);
                }
            }
            (*parent.ptr)
                .c2rust_unnamed
                .c2rust_unnamed
                .dynamic_precedence += dynamic_precedence;
            (*parent.ptr).c2rust_unnamed.c2rust_unnamed.production_id = production_id;
            let mut state: TSStateId = ts_stack_state((*self_0).stack, slice_version);
            let mut next_state: TSStateId =
                ts_language_next_state((*self_0).language, state, symbol);
            if is_extra {
                (*parent.ptr).set_extra(1 as libc::c_int != 0)
            }
            if is_fragile as libc::c_int != 0
                || pop.size > 1 as libc::c_int as libc::c_uint
                || initial_version_count > 1 as libc::c_int as libc::c_uint
            {
                (*parent.ptr).set_fragile_left(1 as libc::c_int != 0);
                (*parent.ptr).set_fragile_right(1 as libc::c_int != 0);
                (*parent.ptr).parse_state = TS_TREE_STATE_NONE
            } else {
                (*parent.ptr).parse_state = state
            }
            // Push the parent node onto the stack, along with any extra tokens that
            // were previously on top of the stack.
            ts_stack_push(
                (*self_0).stack,
                slice_version,
                ts_subtree_from_mut(parent),
                0 as libc::c_int != 0,
                next_state,
            );
            let mut j: uint32_t = (*parent.ptr).child_count;
            while j < slice.subtrees.size {
                ts_stack_push(
                    (*self_0).stack,
                    slice_version,
                    *slice.subtrees.contents.offset(j as isize),
                    0 as libc::c_int != 0,
                    next_state,
                );
                j = j.wrapping_add(1)
            }
            let mut j_0: StackVersion = 0 as libc::c_int as StackVersion;
            while j_0 < slice_version {
                if !(j_0 == version) {
                    if ts_stack_merge((*self_0).stack, j_0, slice_version) {
                        removed_version_count = removed_version_count.wrapping_add(1);
                        break;
                    }
                }
                j_0 = j_0.wrapping_add(1)
            }
        }
        i = i.wrapping_add(1)
    }
    // Return the first new stack version that was created.
    return if ts_stack_version_count((*self_0).stack) > initial_version_count {
        initial_version_count
    } else {
        -(1 as libc::c_int) as StackVersion
    };
}
unsafe extern "C" fn ts_parser__accept(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut lookahead: Subtree,
) {
    assert!(ts_subtree_is_eof(lookahead));
    ts_stack_push(
        (*self_0).stack,
        version,
        lookahead,
        0 as libc::c_int != 0,
        1 as libc::c_int as TSStateId,
    );
    let mut pop: StackSliceArray = ts_stack_pop_all((*self_0).stack, version);
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < pop.size {
        let mut trees: SubtreeArray = (*pop.contents.offset(i as isize)).subtrees;
        let mut root: Subtree = Subtree {
            ptr: 0 as *const SubtreeHeapData,
        };
        let mut j: uint32_t = trees.size.wrapping_sub(1 as libc::c_int as libc::c_uint);
        while j.wrapping_add(1 as libc::c_int as libc::c_uint) > 0 as libc::c_int as libc::c_uint {
            let mut child: Subtree = *trees.contents.offset(j as isize);
            if !ts_subtree_extra(child) {
                assert!(!child.data.is_inline());
                let mut child_count: uint32_t = ts_subtree_child_count(child);
                let mut k: uint32_t = 0 as libc::c_int as uint32_t;
                while k < child_count {
                    ts_subtree_retain(
                        *(*child.ptr)
                            .c2rust_unnamed
                            .c2rust_unnamed
                            .children
                            .offset(k as isize),
                    );
                    k = k.wrapping_add(1)
                }
                array__splice(
                    &mut trees as *mut SubtreeArray as *mut VoidArray,
                    ::std::mem::size_of::<Subtree>() as libc::c_ulong,
                    j,
                    1 as libc::c_int as uint32_t,
                    child_count,
                    (*child.ptr).c2rust_unnamed.c2rust_unnamed.children as *const libc::c_void,
                );
                root = ts_subtree_from_mut(ts_subtree_new_node(
                    &mut (*self_0).tree_pool,
                    ts_subtree_symbol(child),
                    &mut trees,
                    (*child.ptr).c2rust_unnamed.c2rust_unnamed.production_id as libc::c_uint,
                    (*self_0).language,
                ));
                ts_subtree_release(&mut (*self_0).tree_pool, child);
                break;
            } else {
                j = j.wrapping_sub(1)
            }
        }
        assert!(!root.ptr.is_null());
        (*self_0).accept_count = (*self_0).accept_count.wrapping_add(1);
        if !(*self_0).finished_tree.ptr.is_null() {
            if ts_parser__select_tree(self_0, (*self_0).finished_tree, root) {
                ts_subtree_release(&mut (*self_0).tree_pool, (*self_0).finished_tree);
                (*self_0).finished_tree = root
            } else {
                ts_subtree_release(&mut (*self_0).tree_pool, root);
            }
        } else {
            (*self_0).finished_tree = root
        }
        i = i.wrapping_add(1)
    }
    ts_stack_remove_version(
        (*self_0).stack,
        (*pop.contents.offset(0 as libc::c_int as isize)).version,
    );
    ts_stack_halt((*self_0).stack, version);
}
unsafe extern "C" fn ts_parser__do_all_potential_reductions(
    mut self_0: *mut TSParser,
    mut starting_version: StackVersion,
    mut lookahead_symbol: TSSymbol,
) -> bool {
    let mut initial_version_count: uint32_t = ts_stack_version_count((*self_0).stack);
    let mut can_shift_lookahead_symbol: bool = 0 as libc::c_int != 0;
    let mut version: StackVersion = starting_version;
    let mut current_block_33: u64;
    let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    loop {
        let mut version_count: uint32_t = ts_stack_version_count((*self_0).stack);
        if version >= version_count {
            break;
        }
        let mut merged: bool = 0 as libc::c_int != 0;
        let mut i_0: StackVersion = initial_version_count;
        while i_0 < version {
            if ts_stack_merge((*self_0).stack, i_0, version) {
                merged = 1 as libc::c_int != 0;
                break;
            } else {
                i_0 = i_0.wrapping_add(1)
            }
        }
        if !merged {
            let mut state: TSStateId = ts_stack_state((*self_0).stack, version);
            let mut has_shift_action: bool = 0 as libc::c_int != 0;
            (*self_0).reduce_actions.size = 0 as libc::c_int as uint32_t;
            let mut first_symbol: TSSymbol = 0;
            let mut end_symbol: TSSymbol = 0;
            if lookahead_symbol as libc::c_int != 0 as libc::c_int {
                first_symbol = lookahead_symbol;
                end_symbol = (lookahead_symbol as libc::c_int + 1 as libc::c_int) as TSSymbol
            } else {
                first_symbol = 1 as libc::c_int as TSSymbol;
                end_symbol = (*(*self_0).language).token_count as TSSymbol
            }
            let mut symbol: TSSymbol = first_symbol;
            while (symbol as libc::c_int) < end_symbol as libc::c_int {
                let mut entry: TableEntry = TableEntry {
                    actions: 0 as *const TSParseAction,
                    action_count: 0,
                    is_reusable: false,
                };
                ts_language_table_entry((*self_0).language, state, symbol, &mut entry);
                let mut i_1: uint32_t = 0 as libc::c_int as uint32_t;
                while i_1 < entry.action_count {
                    let mut action: TSParseAction = *entry.actions.offset(i_1 as isize);
                    match action.type_0() as libc::c_int {
                        0 | 3 => {
                            if !action.params.c2rust_unnamed.extra()
                                && !action.params.c2rust_unnamed.repetition()
                            {
                                has_shift_action = 1 as libc::c_int != 0
                            }
                        }
                        1 => {
                            if action.params.c2rust_unnamed_0.child_count as libc::c_int
                                > 0 as libc::c_int
                            {
                                ts_reduce_action_set_add(&mut (*self_0).reduce_actions, {
                                    let mut init = ReduceAction {
                                        count: action.params.c2rust_unnamed_0.child_count
                                            as uint32_t,
                                        symbol: action.params.c2rust_unnamed_0.symbol,
                                        dynamic_precedence: action
                                            .params
                                            .c2rust_unnamed_0
                                            .dynamic_precedence
                                            as libc::c_int,
                                        production_id: action.params.c2rust_unnamed_0.production_id
                                            as libc::c_ushort,
                                    };
                                    init
                                });
                            }
                        }
                        _ => {}
                    }
                    i_1 = i_1.wrapping_add(1)
                }
                symbol = symbol.wrapping_add(1)
            }
            let mut reduction_version: StackVersion = -(1 as libc::c_int) as StackVersion;
            let mut i_2: uint32_t = 0 as libc::c_int as uint32_t;
            while i_2 < (*self_0).reduce_actions.size {
                let mut action_0: ReduceAction =
                    *(*self_0).reduce_actions.contents.offset(i_2 as isize);
                reduction_version = ts_parser__reduce(
                    self_0,
                    version,
                    action_0.symbol,
                    action_0.count,
                    action_0.dynamic_precedence,
                    action_0.production_id,
                    1 as libc::c_int != 0,
                    0 as libc::c_int != 0,
                );
                i_2 = i_2.wrapping_add(1)
            }
            if has_shift_action {
                can_shift_lookahead_symbol = 1 as libc::c_int != 0;
                current_block_33 = 13619784596304402172;
            } else if reduction_version != -(1 as libc::c_int) as StackVersion
                && i < MAX_VERSION_COUNT
            {
                ts_stack_renumber_version((*self_0).stack, reduction_version, version);
                current_block_33 = 14916268686031723178;
            } else {
                if lookahead_symbol as libc::c_int != 0 as libc::c_int {
                    ts_stack_remove_version((*self_0).stack, version);
                }
                current_block_33 = 13619784596304402172;
            }
            match current_block_33 {
                14916268686031723178 => {}
                _ => {
                    if version == starting_version {
                        version = version_count
                    } else {
                        version = version.wrapping_add(1)
                    }
                }
            }
        }
        i = i.wrapping_add(1)
    }
    return can_shift_lookahead_symbol;
}
unsafe extern "C" fn ts_parser__handle_error(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut lookahead_symbol: TSSymbol,
) {
    let mut previous_version_count: uint32_t = ts_stack_version_count((*self_0).stack);
    // Perform any reductions that can happen in this state, regardless of the lookahead. After
    // skipping one or more invalid tokens, the parser might find a token that would have allowed
    // a reduction to take place.
    ts_parser__do_all_potential_reductions(self_0, version, 0 as libc::c_int as TSSymbol);
    let mut version_count: uint32_t = ts_stack_version_count((*self_0).stack);
    let mut position: Length = ts_stack_position((*self_0).stack, version);
    // Push a discontinuity onto the stack. Merge all of the stack versions that
    // were created in the previous step.
    let mut did_insert_missing_token: bool = 0 as libc::c_int != 0;
    let mut v: StackVersion = version;
    while v < version_count {
        if !did_insert_missing_token {
            let mut state: TSStateId = ts_stack_state((*self_0).stack, v);
            let mut missing_symbol: TSSymbol = 1 as libc::c_int as TSSymbol;
            while (missing_symbol as libc::c_uint) < (*(*self_0).language).token_count {
                let mut state_after_missing_symbol: TSStateId =
                    ts_language_next_state((*self_0).language, state, missing_symbol);
                if !(state_after_missing_symbol as libc::c_int == 0 as libc::c_int
                    || state_after_missing_symbol as libc::c_int == state as libc::c_int)
                {
                    if ts_language_has_reduce_action(
                        (*self_0).language,
                        state_after_missing_symbol,
                        lookahead_symbol,
                    ) {
                        // In case the parser is currently outside of any included range, the lexer will
                        // snap to the beginning of the next included range. The missing token's padding
                        // must be assigned to position it within the next included range.
                        ts_lexer_reset(&mut (*self_0).lexer, position);
                        ts_lexer_mark_end(&mut (*self_0).lexer);
                        let mut padding: Length =
                            length_sub((*self_0).lexer.token_end_position, position);
                        let mut version_with_missing_tree: StackVersion =
                            ts_stack_copy_version((*self_0).stack, v);
                        let mut missing_tree: Subtree = ts_subtree_new_missing_leaf(
                            &mut (*self_0).tree_pool,
                            missing_symbol,
                            padding,
                            (*self_0).language,
                        );
                        ts_stack_push(
                            (*self_0).stack,
                            version_with_missing_tree,
                            missing_tree,
                            0 as libc::c_int != 0,
                            state_after_missing_symbol,
                        );
                        if ts_parser__do_all_potential_reductions(
                            self_0,
                            version_with_missing_tree,
                            lookahead_symbol,
                        ) {
                            if (*self_0).lexer.logger.log.is_some()
                                || !(*self_0).dot_graph_file.is_null()
                            {
                                snprintf(
                                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                                    1024,
                                    b"recover_with_missing symbol:%s, state:%u\x00" as *const u8
                                        as *const libc::c_char,
                                    ts_language_symbol_name((*self_0).language, missing_symbol),
                                    ts_stack_state((*self_0).stack, version_with_missing_tree)
                                        as libc::c_int,
                                );
                                ts_parser__log(self_0);
                            }
                            did_insert_missing_token = 1 as libc::c_int != 0;
                            break;
                        }
                    }
                }
                missing_symbol = missing_symbol.wrapping_add(1)
            }
        }
        ts_stack_push(
            (*self_0).stack,
            v,
            Subtree {
                ptr: 0 as *const SubtreeHeapData,
            },
            0 as libc::c_int != 0,
            0 as libc::c_int as TSStateId,
        );
        v = if v == version {
            previous_version_count
        } else {
            v.wrapping_add(1 as libc::c_int as libc::c_uint)
        }
    }
    let mut i: libc::c_uint = previous_version_count;
    while i < version_count {
        let mut did_merge: bool = ts_stack_merge((*self_0).stack, version, previous_version_count);
        assert!(did_merge);
        i = i.wrapping_add(1)
    }
    ts_stack_record_summary((*self_0).stack, version, MAX_SUMMARY_DEPTH);
    if !(*self_0).dot_graph_file.is_null() {
        ts_stack_print_dot_graph(
            (*self_0).stack,
            (*self_0).language,
            (*self_0).dot_graph_file,
        );
        fputs(
            b"\n\n\x00" as *const u8 as *const libc::c_char,
            (*self_0).dot_graph_file,
        );
    };
}
unsafe extern "C" fn ts_parser__recover_to_state(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut depth: libc::c_uint,
    mut goal_state: TSStateId,
) -> bool {
    let mut pop: StackSliceArray = ts_stack_pop_count((*self_0).stack, version, depth);
    let mut previous_version: StackVersion = -(1 as libc::c_int) as StackVersion;
    let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    while i < pop.size {
        let mut slice: StackSlice = *pop.contents.offset(i as isize);
        if slice.version == previous_version {
            ts_subtree_array_delete(&mut (*self_0).tree_pool, &mut slice.subtrees);
            let fresh8 = i;
            i = i.wrapping_sub(1);
            array__erase(
                &mut pop as *mut StackSliceArray as *mut VoidArray,
                ::std::mem::size_of::<StackSlice>() as libc::c_ulong,
                fresh8,
            );
        } else if ts_stack_state((*self_0).stack, slice.version) as libc::c_int
            != goal_state as libc::c_int
        {
            ts_stack_halt((*self_0).stack, slice.version);
            ts_subtree_array_delete(&mut (*self_0).tree_pool, &mut slice.subtrees);
            let fresh9 = i;
            i = i.wrapping_sub(1);
            array__erase(
                &mut pop as *mut StackSliceArray as *mut VoidArray,
                ::std::mem::size_of::<StackSlice>() as libc::c_ulong,
                fresh9,
            );
        } else {
            let mut error_trees: SubtreeArray = ts_stack_pop_error((*self_0).stack, slice.version);
            if error_trees.size > 0 as libc::c_int as libc::c_uint {
                assert!(error_trees.size == 1 as libc::c_int as libc::c_uint);
                let mut error_tree: Subtree =
                    *error_trees.contents.offset(0 as libc::c_int as isize);
                let mut error_child_count: uint32_t = ts_subtree_child_count(error_tree);
                if error_child_count > 0 as libc::c_int as libc::c_uint {
                    array__splice(
                        &mut slice.subtrees as *mut SubtreeArray as *mut VoidArray,
                        ::std::mem::size_of::<Subtree>() as libc::c_ulong,
                        0 as libc::c_int as uint32_t,
                        0 as libc::c_int as uint32_t,
                        error_child_count,
                        (*error_tree.ptr).c2rust_unnamed.c2rust_unnamed.children
                            as *const libc::c_void,
                    );
                    let mut j: libc::c_uint = 0 as libc::c_int as libc::c_uint;
                    while j < error_child_count {
                        ts_subtree_retain(*slice.subtrees.contents.offset(j as isize));
                        j = j.wrapping_add(1)
                    }
                }
                ts_subtree_array_delete(&mut (*self_0).tree_pool, &mut error_trees);
            }
            let mut trailing_extras: SubtreeArray =
                ts_subtree_array_remove_trailing_extras(&mut slice.subtrees);
            if slice.subtrees.size > 0 as libc::c_int as libc::c_uint {
                let mut error: Subtree = ts_subtree_new_error_node(
                    &mut (*self_0).tree_pool,
                    &mut slice.subtrees,
                    1 as libc::c_int != 0,
                    (*self_0).language,
                );
                ts_stack_push(
                    (*self_0).stack,
                    slice.version,
                    error,
                    0 as libc::c_int != 0,
                    goal_state,
                );
            } else {
                array__delete(&mut slice.subtrees as *mut SubtreeArray as *mut VoidArray);
            }
            let mut j_0: libc::c_uint = 0 as libc::c_int as libc::c_uint;
            while j_0 < trailing_extras.size {
                let mut tree: Subtree = *trailing_extras.contents.offset(j_0 as isize);
                ts_stack_push(
                    (*self_0).stack,
                    slice.version,
                    tree,
                    0 as libc::c_int != 0,
                    goal_state,
                );
                j_0 = j_0.wrapping_add(1)
            }
            previous_version = slice.version;
            array__delete(&mut trailing_extras as *mut SubtreeArray as *mut VoidArray);
        }
        i = i.wrapping_add(1)
    }
    return previous_version != -(1 as libc::c_int) as StackVersion;
}
unsafe extern "C" fn ts_parser__recover(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut lookahead: Subtree,
) {
    let mut did_recover: bool = 0 as libc::c_int != 0;
    let mut previous_version_count: libc::c_uint = ts_stack_version_count((*self_0).stack);
    let mut position: Length = ts_stack_position((*self_0).stack, version);
    let mut summary: *mut StackSummary = ts_stack_get_summary((*self_0).stack, version);
    let mut node_count_since_error: libc::c_uint =
        ts_stack_node_count_since_error((*self_0).stack, version);
    let mut current_error_cost: libc::c_uint = ts_stack_error_cost((*self_0).stack, version);
    // When the parser is in the error state, there are two strategies for recovering with a
    // given lookahead token:
    // 1. Find a previous state on the stack in which that lookahead token would be valid. Then,
    //    create a new stack version that is in that state again. This entails popping all of the
    //    subtrees that have been pushed onto the stack since that previous state, and wrapping
    //    them in an ERROR node.
    // 2. Wrap the lookahead token in an ERROR node, push that ERROR node onto the stack, and
    //    move on to the next lookahead token, remaining in the error state.
    //
    // First, try the strategy 1. Upon entering the error state, the parser recorded a summary
    // of the previous parse states and their depths. Look at each state in the summary, to see
    // if the current lookahead token would be valid in that state.
    if !summary.is_null() && !ts_subtree_is_error(lookahead) {
        let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
        while i < (*summary).size {
            let mut entry: StackSummaryEntry = *(*summary).contents.offset(i as isize);
            if !(entry.state as libc::c_int == 0 as libc::c_int) {
                if !(entry.position.bytes == position.bytes) {
                    let mut depth: libc::c_uint = entry.depth;
                    if node_count_since_error > 0 as libc::c_int as libc::c_uint {
                        depth = depth.wrapping_add(1)
                    }
                    // Do not recover in ways that create redundant stack versions.
                    let mut would_merge: bool = 0 as libc::c_int != 0;
                    let mut j: libc::c_uint = 0 as libc::c_int as libc::c_uint;
                    while j < previous_version_count {
                        if ts_stack_state((*self_0).stack, j) as libc::c_int
                            == entry.state as libc::c_int
                            && ts_stack_position((*self_0).stack, j).bytes == position.bytes
                        {
                            would_merge = 1 as libc::c_int != 0;
                            break;
                        } else {
                            j = j.wrapping_add(1)
                        }
                    }
                    if !would_merge {
                        // Do not recover if the result would clearly be worse than some existing stack version.
                        let mut new_cost: libc::c_uint = current_error_cost
                            .wrapping_add(
                                entry.depth.wrapping_mul(100 as libc::c_int as libc::c_uint),
                            )
                            .wrapping_add(
                                position
                                    .bytes
                                    .wrapping_sub(entry.position.bytes)
                                    .wrapping_mul(1 as libc::c_int as libc::c_uint),
                            )
                            .wrapping_add(
                                position
                                    .extent
                                    .row
                                    .wrapping_sub(entry.position.extent.row)
                                    .wrapping_mul(30 as libc::c_int as libc::c_uint),
                            );
                        if ts_parser__better_version_exists(
                            self_0,
                            version,
                            0 as libc::c_int != 0,
                            new_cost,
                        ) {
                            break;
                        }
                        // If the current lookahead token is valid in some previous state, recover to that state.
                        // Then stop looking for further recoveries.
                        if ts_language_has_actions(
                            (*self_0).language,
                            entry.state,
                            ts_subtree_symbol(lookahead),
                        ) {
                            if ts_parser__recover_to_state(self_0, version, depth, entry.state) {
                                did_recover = 1 as libc::c_int != 0;
                                if (*self_0).lexer.logger.log.is_some()
                                    || !(*self_0).dot_graph_file.is_null()
                                {
                                    snprintf(
                                        (*self_0).lexer.debug_buffer.as_mut_ptr(),
                                        1024,
                                        b"recover_to_previous state:%u, depth:%u\x00" as *const u8
                                            as *const libc::c_char,
                                        entry.state as libc::c_int,
                                        depth,
                                    );
                                    ts_parser__log(self_0);
                                }
                                if !(*self_0).dot_graph_file.is_null() {
                                    ts_stack_print_dot_graph(
                                        (*self_0).stack,
                                        (*self_0).language,
                                        (*self_0).dot_graph_file,
                                    );
                                    fputs(
                                        b"\n\n\x00" as *const u8 as *const libc::c_char,
                                        (*self_0).dot_graph_file,
                                    );
                                }
                                break;
                            }
                        }
                    }
                }
            }
            i = i.wrapping_add(1)
        }
    }
    // In the process of attemping to recover, some stack versions may have been created
    // and subsequently halted. Remove those versions.
    let mut i_0: libc::c_uint = previous_version_count;
    while i_0 < ts_stack_version_count((*self_0).stack) {
        if !ts_stack_is_active((*self_0).stack, i_0) {
            let fresh10 = i_0;
            i_0 = i_0.wrapping_sub(1);
            ts_stack_remove_version((*self_0).stack, fresh10);
        }
        i_0 = i_0.wrapping_add(1)
    }
    // If strategy 1 succeeded, a new stack version will have been created which is able to handle
    // the current lookahead token. Now, in addition, try strategy 2 described above: skip the
    // current lookahead token by wrapping it in an ERROR node.
    // Don't pursue this additional strategy if there are already too many stack versions.
    if did_recover as libc::c_int != 0
        && ts_stack_version_count((*self_0).stack) > MAX_VERSION_COUNT
    {
        ts_stack_halt((*self_0).stack, version);
        ts_subtree_release(&mut (*self_0).tree_pool, lookahead);
        return;
    }
    // If the parser is still in the error state at the end of the file, just wrap everything
    // in an ERROR node and terminate.
    if ts_subtree_is_eof(lookahead) {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"recover_eof\x00" as *const u8 as *const libc::c_char,
            );
            ts_parser__log(self_0);
        }
        let mut children: SubtreeArray = {
            let mut init = SubtreeArray {
                contents: 0 as *mut Subtree,
                size: 0 as libc::c_int as uint32_t,
                capacity: 0 as libc::c_int as uint32_t,
            };
            init
        };
        let mut parent: Subtree = ts_subtree_new_error_node(
            &mut (*self_0).tree_pool,
            &mut children,
            0 as libc::c_int != 0,
            (*self_0).language,
        );
        ts_stack_push(
            (*self_0).stack,
            version,
            parent,
            0 as libc::c_int != 0,
            1 as libc::c_int as TSStateId,
        );
        ts_parser__accept(self_0, version, lookahead);
        return;
    }
    // Do not recover if the result would clearly be worse than some existing stack version.
    let mut new_cost_0: libc::c_uint = current_error_cost
        .wrapping_add(100 as libc::c_int as libc::c_uint)
        .wrapping_add(
            ts_subtree_total_bytes(lookahead).wrapping_mul(1 as libc::c_int as libc::c_uint),
        )
        .wrapping_add(
            ts_subtree_total_size(lookahead)
                .extent
                .row
                .wrapping_mul(30 as libc::c_int as libc::c_uint),
        );
    if ts_parser__better_version_exists(self_0, version, 0 as libc::c_int != 0, new_cost_0) {
        ts_stack_halt((*self_0).stack, version);
        ts_subtree_release(&mut (*self_0).tree_pool, lookahead);
        return;
    }
    // If the current lookahead token is an extra token, mark it as extra. This means it won't
    // be counted in error cost calculations.
    let mut n: libc::c_uint = 0;
    let mut actions: *const TSParseAction = ts_language_actions(
        (*self_0).language,
        1 as libc::c_int as TSStateId,
        ts_subtree_symbol(lookahead),
        &mut n,
    );
    if n > 0 as libc::c_int as libc::c_uint
        && (*actions.offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)).type_0()
            as libc::c_int
            == TSParseActionTypeShift as libc::c_int
        && (*actions.offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
            .params
            .c2rust_unnamed
            .extra() as libc::c_int
            != 0
    {
        let mut mutable_lookahead: MutableSubtree =
            ts_subtree_make_mut(&mut (*self_0).tree_pool, lookahead);
        ts_subtree_set_extra(&mut mutable_lookahead);
        lookahead = ts_subtree_from_mut(mutable_lookahead)
    }
    // Wrap the lookahead token in an ERROR.
    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
        snprintf(
            (*self_0).lexer.debug_buffer.as_mut_ptr(),
            1024,
            b"skip_token symbol:%s\x00" as *const u8 as *const libc::c_char,
            ts_language_symbol_name((*self_0).language, ts_subtree_symbol(lookahead)),
        );
        ts_parser__log(self_0);
    }
    let mut children_0: SubtreeArray = {
        let mut init = SubtreeArray {
            contents: 0 as *mut Subtree,
            size: 0 as libc::c_int as uint32_t,
            capacity: 0 as libc::c_int as uint32_t,
        };
        init
    };
    array__reserve(
        &mut children_0 as *mut SubtreeArray as *mut VoidArray,
        ::std::mem::size_of::<Subtree>() as libc::c_ulong,
        1 as libc::c_int as uint32_t,
    );
    array__grow(
        &mut children_0 as *mut SubtreeArray as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<Subtree>() as libc::c_ulong,
    );
    let fresh11 = children_0.size;
    children_0.size = children_0.size.wrapping_add(1);
    *children_0.contents.offset(fresh11 as isize) = lookahead;
    let mut error_repeat: MutableSubtree = ts_subtree_new_node(
        &mut (*self_0).tree_pool,
        (-(1 as libc::c_int) as TSSymbol as libc::c_int - 1 as libc::c_int) as TSSymbol,
        &mut children_0,
        0 as libc::c_int as libc::c_uint,
        (*self_0).language,
    );
    // If other tokens have already been skipped, so there is already an ERROR at the top of the
    // stack, then pop that ERROR off the stack and wrap the two ERRORs together into one larger
    // ERROR.
    if node_count_since_error > 0 as libc::c_int as libc::c_uint {
        let mut pop: StackSliceArray =
            ts_stack_pop_count((*self_0).stack, version, 1 as libc::c_int as uint32_t);
        // TODO: Figure out how to make this condition occur.
        // See https://github.com/atom/atom/issues/18450#issuecomment-439579778
        // If multiple stack versions have merged at this point, just pick one of the errors
        // arbitrarily and discard the rest.
        if pop.size > 1 as libc::c_int as libc::c_uint {
            let mut i_1: libc::c_uint = 1 as libc::c_int as libc::c_uint;
            while i_1 < pop.size {
                ts_subtree_array_delete(
                    &mut (*self_0).tree_pool,
                    &mut (*pop.contents.offset(i_1 as isize)).subtrees,
                );
                i_1 = i_1.wrapping_add(1)
            }
            while ts_stack_version_count((*self_0).stack)
                > (*pop.contents.offset(0 as libc::c_int as isize))
                    .version
                    .wrapping_add(1 as libc::c_int as libc::c_uint)
            {
                ts_stack_remove_version(
                    (*self_0).stack,
                    (*pop.contents.offset(0 as libc::c_int as isize))
                        .version
                        .wrapping_add(1 as libc::c_int as libc::c_uint),
                );
            }
        }
        ts_stack_renumber_version(
            (*self_0).stack,
            (*pop.contents.offset(0 as libc::c_int as isize)).version,
            version,
        );
        array__grow(
            &mut (*pop.contents.offset(0 as libc::c_int as isize)).subtrees as *mut SubtreeArray
                as *mut VoidArray,
            1 as libc::c_int as size_t,
            ::std::mem::size_of::<Subtree>() as libc::c_ulong,
        );
        let ref mut fresh12 = (*pop.contents.offset(0 as libc::c_int as isize))
            .subtrees
            .size;
        let fresh13 = *fresh12;
        *fresh12 = (*fresh12).wrapping_add(1);
        *(*pop.contents.offset(0 as libc::c_int as isize))
            .subtrees
            .contents
            .offset(fresh13 as isize) = ts_subtree_from_mut(error_repeat);
        error_repeat = ts_subtree_new_node(
            &mut (*self_0).tree_pool,
            (-(1 as libc::c_int) as TSSymbol as libc::c_int - 1 as libc::c_int) as TSSymbol,
            &mut (*pop.contents.offset(0 as libc::c_int as isize)).subtrees,
            0 as libc::c_int as libc::c_uint,
            (*self_0).language,
        )
    }
    // Push the new ERROR onto the stack.
    ts_stack_push(
        (*self_0).stack,
        version,
        ts_subtree_from_mut(error_repeat),
        0 as libc::c_int != 0,
        0 as libc::c_int as TSStateId,
    );
    if ts_subtree_has_external_tokens(lookahead) {
        ts_stack_set_last_external_token(
            (*self_0).stack,
            version,
            ts_subtree_last_external_token(lookahead),
        );
    };
}
unsafe extern "C" fn ts_parser__advance(
    mut self_0: *mut TSParser,
    mut version: StackVersion,
    mut allow_node_reuse: bool,
) -> bool {
    let mut state: TSStateId = ts_stack_state((*self_0).stack, version);
    let mut position: uint32_t = ts_stack_position((*self_0).stack, version).bytes;
    let mut last_external_token: Subtree = ts_stack_last_external_token((*self_0).stack, version);
    let mut did_reuse: bool = 1 as libc::c_int != 0;
    let mut lookahead: Subtree = Subtree {
        ptr: 0 as *const SubtreeHeapData,
    };
    let mut table_entry: TableEntry = {
        let mut init = TableEntry {
            actions: 0 as *const TSParseAction,
            action_count: 0 as libc::c_int as uint32_t,
            is_reusable: false,
        };
        init
    };
    // If possible, reuse a node from the previous syntax tree.
    if allow_node_reuse {
        lookahead = ts_parser__reuse_node(
            self_0,
            version,
            &mut state,
            position,
            last_external_token,
            &mut table_entry,
        )
    }
    // If no node from the previous syntax tree could be reused, then try to
    // reuse the token previously returned by the lexer.
    if lookahead.ptr.is_null() {
        did_reuse = 0 as libc::c_int != 0;
        lookahead = ts_parser__get_cached_token(
            self_0,
            state,
            position as size_t,
            last_external_token,
            &mut table_entry,
        )
    }
    // Otherwise, re-run the lexer.
    if lookahead.ptr.is_null() {
        lookahead = ts_parser__lex(self_0, version, state);
        if !lookahead.ptr.is_null() {
            ts_parser__set_cached_token(self_0, position as size_t, last_external_token, lookahead);
            ts_language_table_entry(
                (*self_0).language,
                state,
                ts_subtree_symbol(lookahead),
                &mut table_entry,
            );
        } else {
            // When parsing a non-terminal extra, a null lookahead indicates the
            // end of the rule. The reduction is stored in the EOF table entry.
            // After the reduction, the lexer needs to be run again.
            ts_language_table_entry(
                (*self_0).language,
                state,
                0 as libc::c_int as TSSymbol,
                &mut table_entry,
            );
        }
    }
    loop {
        // If a cancellation flag or a timeout was provided, then check every
        // time a fixed number of parse actions has been processed.
        (*self_0).operation_count = (*self_0).operation_count.wrapping_add(1);
        if (*self_0).operation_count == OP_COUNT_PER_TIMEOUT_CHECK {
            (*self_0).operation_count = 0 as libc::c_int as libc::c_uint
        }
        if (*self_0).operation_count == 0 as libc::c_int as libc::c_uint
            && (!(*self_0).cancellation_flag.is_null()
                && atomic_load((*self_0).cancellation_flag) != 0
                || !clock_is_null((*self_0).end_clock)
                    && clock_is_gt(clock_now(), (*self_0).end_clock) as libc::c_int != 0)
        {
            ts_subtree_release(&mut (*self_0).tree_pool, lookahead);
            return 0 as libc::c_int != 0;
        }
        // Process each parse action for the current lookahead token in
        // the current state. If there are multiple actions, then this is
        // an ambiguous state. REDUCE actions always create a new stack
        // version, whereas SHIFT actions update the existing stack version
        // and terminate this loop.
        let mut last_reduction_version: StackVersion = -(1 as libc::c_int) as StackVersion;
        let mut current_block_67: u64;
        let mut i: uint32_t = 0 as libc::c_int as uint32_t;
        while i < table_entry.action_count {
            let mut action: TSParseAction = *table_entry.actions.offset(i as isize);
            match action.type_0() as libc::c_int {
                0 => {
                    if !action.params.c2rust_unnamed.repetition() {
                        let mut next_state: TSStateId = 0;
                        if action.params.c2rust_unnamed.extra() {
                            // TODO: remove when TREE_SITTER_LANGUAGE_VERSION 9 is out.
                            if state as libc::c_int == 0 as libc::c_int {
                                current_block_67 = 15125582407903384992;
                            } else {
                                next_state = state;
                                if (*self_0).lexer.logger.log.is_some()
                                    || !(*self_0).dot_graph_file.is_null()
                                {
                                    snprintf(
                                        (*self_0).lexer.debug_buffer.as_mut_ptr(),
                                        1024,
                                        b"shift_extra\x00" as *const u8 as *const libc::c_char,
                                    );
                                    ts_parser__log(self_0);
                                }
                                current_block_67 = 6717214610478484138;
                            }
                        } else {
                            next_state = action.params.c2rust_unnamed.state;
                            if (*self_0).lexer.logger.log.is_some()
                                || !(*self_0).dot_graph_file.is_null()
                            {
                                snprintf(
                                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                                    1024,
                                    b"shift state:%u\x00" as *const u8 as *const libc::c_char,
                                    next_state as libc::c_int,
                                );
                                ts_parser__log(self_0);
                            }
                            current_block_67 = 6717214610478484138;
                        }
                        match current_block_67 {
                            15125582407903384992 => {}
                            _ => {
                                if ts_subtree_child_count(lookahead)
                                    > 0 as libc::c_int as libc::c_uint
                                {
                                    ts_parser__breakdown_lookahead(
                                        self_0,
                                        &mut lookahead,
                                        state,
                                        &mut (*self_0).reusable_node,
                                    );
                                    next_state = ts_language_next_state(
                                        (*self_0).language,
                                        state,
                                        ts_subtree_symbol(lookahead),
                                    )
                                }
                                ts_parser__shift(
                                    self_0,
                                    version,
                                    next_state,
                                    lookahead,
                                    action.params.c2rust_unnamed.extra(),
                                );
                                if did_reuse {
                                    reusable_node_advance(&mut (*self_0).reusable_node);
                                }
                                return 1 as libc::c_int != 0;
                            }
                        }
                    }
                }
                1 => {
                    let mut is_fragile: bool =
                        table_entry.action_count > 1 as libc::c_int as libc::c_uint;
                    let mut is_extra: bool = lookahead.ptr.is_null();
                    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                        snprintf(
                            (*self_0).lexer.debug_buffer.as_mut_ptr(),
                            1024,
                            b"reduce sym:%s, child_count:%u\x00" as *const u8
                                as *const libc::c_char,
                            ts_language_symbol_name(
                                (*self_0).language,
                                action.params.c2rust_unnamed_0.symbol,
                            ),
                            action.params.c2rust_unnamed_0.child_count as libc::c_int,
                        );
                        ts_parser__log(self_0);
                    }
                    let mut reduction_version: StackVersion = ts_parser__reduce(
                        self_0,
                        version,
                        action.params.c2rust_unnamed_0.symbol,
                        action.params.c2rust_unnamed_0.child_count as uint32_t,
                        action.params.c2rust_unnamed_0.dynamic_precedence as libc::c_int,
                        action.params.c2rust_unnamed_0.production_id as uint16_t,
                        is_fragile,
                        is_extra,
                    );
                    if reduction_version != -(1 as libc::c_int) as StackVersion {
                        last_reduction_version = reduction_version
                    }
                }
                2 => {
                    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                        snprintf(
                            (*self_0).lexer.debug_buffer.as_mut_ptr(),
                            1024,
                            b"accept\x00" as *const u8 as *const libc::c_char,
                        );
                        ts_parser__log(self_0);
                    }
                    ts_parser__accept(self_0, version, lookahead);
                    return 1 as libc::c_int != 0;
                }
                3 => {
                    if ts_subtree_child_count(lookahead) > 0 as libc::c_int as libc::c_uint {
                        ts_parser__breakdown_lookahead(
                            self_0,
                            &mut lookahead,
                            0 as libc::c_int as TSStateId,
                            &mut (*self_0).reusable_node,
                        );
                    }
                    ts_parser__recover(self_0, version, lookahead);
                    if did_reuse {
                        reusable_node_advance(&mut (*self_0).reusable_node);
                    }
                    return 1 as libc::c_int != 0;
                }
                _ => {}
            }
            i = i.wrapping_add(1)
        }
        // If a reduction was performed, then replace the current stack version
        // with one of the stack versions created by a reduction, and continue
        // processing this version of the stack with the same lookahead symbol.
        if last_reduction_version != -(1 as libc::c_int) as StackVersion {
            ts_stack_renumber_version((*self_0).stack, last_reduction_version, version);
            if !(*self_0).dot_graph_file.is_null() {
                ts_stack_print_dot_graph(
                    (*self_0).stack,
                    (*self_0).language,
                    (*self_0).dot_graph_file,
                );
                fputs(
                    b"\n\n\x00" as *const u8 as *const libc::c_char,
                    (*self_0).dot_graph_file,
                );
            }
            state = ts_stack_state((*self_0).stack, version);
            // At the end of a non-terminal extra rule, the lexer will return a
            // null subtree, because the parser needs to perform a fixed reduction
            // regardless of the lookahead node. After performing that reduction,
            // (and completing the non-terminal extra rule) run the lexer again based
            // on the current parse state.
            if lookahead.ptr.is_null() {
                lookahead = ts_parser__lex(self_0, version, state)
            }
            ts_language_table_entry(
                (*self_0).language,
                state,
                ts_subtree_leaf_symbol(lookahead),
                &mut table_entry,
            );
        } else {
            // If there were no parse actions for the current lookahead token, then
            // it is not valid in this state. If the current lookahead token is a
            // keyword, then switch to treating it as the normal word token if that
            // token is valid in this state.
            if ts_subtree_is_keyword(lookahead) as libc::c_int != 0
                && ts_subtree_symbol(lookahead) as libc::c_int
                    != (*(*self_0).language).keyword_capture_token as libc::c_int
            {
                ts_language_table_entry(
                    (*self_0).language,
                    state,
                    (*(*self_0).language).keyword_capture_token,
                    &mut table_entry,
                );
                if table_entry.action_count > 0 as libc::c_int as libc::c_uint {
                    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                        snprintf(
                            (*self_0).lexer.debug_buffer.as_mut_ptr(),
                            1024,
                            b"switch from_keyword:%s, to_word_token:%s\x00" as *const u8
                                as *const libc::c_char,
                            ts_language_symbol_name(
                                (*self_0).language,
                                ts_subtree_symbol(lookahead),
                            ),
                            ts_language_symbol_name(
                                (*self_0).language,
                                (*(*self_0).language).keyword_capture_token,
                            ),
                        );
                        ts_parser__log(self_0);
                    }
                    let mut mutable_lookahead: MutableSubtree =
                        ts_subtree_make_mut(&mut (*self_0).tree_pool, lookahead);
                    ts_subtree_set_symbol(
                        &mut mutable_lookahead,
                        (*(*self_0).language).keyword_capture_token,
                        (*self_0).language,
                    );
                    lookahead = ts_subtree_from_mut(mutable_lookahead);
                    continue;
                }
            }
            // If the current lookahead token is not valid and the parser is
            // already in the error state, restart the error recovery process.
            // TODO - can this be unified with the other `RECOVER` case above?
            if state as libc::c_int == 0 as libc::c_int {
                ts_parser__recover(self_0, version, lookahead);
                return 1 as libc::c_int != 0;
            }
            // If the current lookahead token is not valid and the previous
            // subtree on the stack was reused from an old tree, it isn't actually
            // valid to reuse it. Remove it from the stack, and in its place,
            // push each of its children. Then try again to process the current
            // lookahead.
            if ts_parser__breakdown_top_of_stack(self_0, version) {
                continue;
            }
            // At this point, the current lookahead token is definitely not valid
            // for this parse stack version. Mark this version as paused and continue
            // processing any other stack versions that might exist. If some other
            // version advances successfully, then this version can simply be removed.
            // But if all versions end up paused, then error recovery is needed.
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"detect_error\x00" as *const u8 as *const libc::c_char,
                );
                ts_parser__log(self_0);
            }
            ts_stack_pause((*self_0).stack, version, ts_subtree_leaf_symbol(lookahead));
            ts_subtree_release(&mut (*self_0).tree_pool, lookahead);
            return 1 as libc::c_int != 0;
        }
    }
}
unsafe extern "C" fn ts_parser__condense_stack(mut self_0: *mut TSParser) -> libc::c_uint {
    let mut made_changes: bool = 0 as libc::c_int != 0;
    let mut min_error_cost: libc::c_uint = (2147483647 as libc::c_int as libc::c_uint)
        .wrapping_mul(2 as libc::c_uint)
        .wrapping_add(1 as libc::c_uint);
    let mut i: StackVersion = 0 as libc::c_int as StackVersion;
    while i < ts_stack_version_count((*self_0).stack) {
        // Prune any versions that have been marked for removal.
        if ts_stack_is_halted((*self_0).stack, i) {
            ts_stack_remove_version((*self_0).stack, i);
            i = i.wrapping_sub(1)
        } else {
            // Keep track of the minimum error cost of any stack version so
            // that it can be returned.
            let mut status_i: ErrorStatus = ts_parser__version_status(self_0, i);
            if !status_i.is_in_error && status_i.cost < min_error_cost {
                min_error_cost = status_i.cost
            }
            // Examine each pair of stack versions, removing any versions that
            // are clearly worse than another version. Ensure that the versions
            // are ordered from most promising to least promising.
            let mut j: StackVersion = 0 as libc::c_int as StackVersion;
            while j < i {
                let mut status_j: ErrorStatus = ts_parser__version_status(self_0, j);
                match ts_parser__compare_versions(self_0, status_j, status_i) as libc::c_uint {
                    0 => {
                        made_changes = 1 as libc::c_int != 0;
                        ts_stack_remove_version((*self_0).stack, i);
                        i = i.wrapping_sub(1);
                        j = i
                    }
                    1 | 2 => {
                        if ts_stack_merge((*self_0).stack, j, i) {
                            made_changes = 1 as libc::c_int != 0;
                            i = i.wrapping_sub(1);
                            j = i
                        }
                    }
                    3 => {
                        made_changes = 1 as libc::c_int != 0;
                        if ts_stack_merge((*self_0).stack, j, i) {
                            i = i.wrapping_sub(1);
                            j = i
                        } else {
                            ts_stack_swap_versions((*self_0).stack, i, j);
                        }
                    }
                    4 => {
                        made_changes = 1 as libc::c_int != 0;
                        ts_stack_remove_version((*self_0).stack, j);
                        i = i.wrapping_sub(1);
                        j = j.wrapping_sub(1)
                    }
                    _ => {}
                }
                j = j.wrapping_add(1)
            }
        }
        i = i.wrapping_add(1)
    }
    // Enfore a hard upper bound on the number of stack versions by
    // discarding the least promising versions.
    while ts_stack_version_count((*self_0).stack) > MAX_VERSION_COUNT {
        ts_stack_remove_version((*self_0).stack, MAX_VERSION_COUNT);
        made_changes = 1 as libc::c_int != 0
    }
    // If the best-performing stack version is currently paused, or all
    // versions are paused, then resume the best paused version and begin
    // the error recovery process. Otherwise, remove the paused versions.
    if ts_stack_version_count((*self_0).stack) > 0 as libc::c_int as libc::c_uint {
        let mut has_unpaused_version: bool = 0 as libc::c_int != 0;
        let mut i_0: StackVersion = 0 as libc::c_int as StackVersion;
        let mut n: StackVersion = ts_stack_version_count((*self_0).stack);
        while i_0 < n {
            if ts_stack_is_paused((*self_0).stack, i_0) {
                if !has_unpaused_version && (*self_0).accept_count < MAX_VERSION_COUNT {
                    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                        snprintf(
                            (*self_0).lexer.debug_buffer.as_mut_ptr(),
                            1024,
                            b"resume version:%u\x00" as *const u8 as *const libc::c_char,
                            i_0,
                        );
                        ts_parser__log(self_0);
                    }
                    min_error_cost = ts_stack_error_cost((*self_0).stack, i_0);
                    let mut lookahead_symbol: TSSymbol = ts_stack_resume((*self_0).stack, i_0);
                    ts_parser__handle_error(self_0, i_0, lookahead_symbol);
                    has_unpaused_version = 1 as libc::c_int != 0
                } else {
                    ts_stack_remove_version((*self_0).stack, i_0);
                    i_0 = i_0.wrapping_sub(1);
                    n = n.wrapping_sub(1)
                }
            } else {
                has_unpaused_version = 1 as libc::c_int != 0
            }
            i_0 = i_0.wrapping_add(1)
        }
    }
    if made_changes {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"condense\x00" as *const u8 as *const libc::c_char,
            );
            ts_parser__log(self_0);
        }
        if !(*self_0).dot_graph_file.is_null() {
            ts_stack_print_dot_graph(
                (*self_0).stack,
                (*self_0).language,
                (*self_0).dot_graph_file,
            );
            fputs(
                b"\n\n\x00" as *const u8 as *const libc::c_char,
                (*self_0).dot_graph_file,
            );
        }
    }
    return min_error_cost;
}
unsafe extern "C" fn ts_parser_has_outstanding_parse(mut self_0: *mut TSParser) -> bool {
    return ts_stack_state((*self_0).stack, 0 as libc::c_int as StackVersion) as libc::c_int
        != 1 as libc::c_int
        || ts_stack_node_count_since_error((*self_0).stack, 0 as libc::c_int as StackVersion)
            != 0 as libc::c_int as libc::c_uint;
}
// Parser - Public
#[no_mangle]
pub unsafe extern "C" fn ts_parser_new() -> *mut TSParser {
    let mut self_0: *mut TSParser = ts_calloc(
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<TSParser>() as libc::c_ulong,
    ) as *mut TSParser;
    ts_lexer_init(&mut (*self_0).lexer);
    (*self_0).reduce_actions.size = 0 as libc::c_int as uint32_t;
    (*self_0).reduce_actions.capacity = 0 as libc::c_int as uint32_t;
    (*self_0).reduce_actions.contents = 0 as *mut ReduceAction;
    array__reserve(
        &mut (*self_0).reduce_actions as *mut ReduceActionSet as *mut VoidArray,
        ::std::mem::size_of::<ReduceAction>() as libc::c_ulong,
        4 as libc::c_int as uint32_t,
    );
    (*self_0).tree_pool = ts_subtree_pool_new(32 as libc::c_int as uint32_t);
    (*self_0).stack = ts_stack_new(&mut (*self_0).tree_pool);
    (*self_0).finished_tree = Subtree {
        ptr: 0 as *const SubtreeHeapData,
    };
    (*self_0).reusable_node = reusable_node_new();
    (*self_0).dot_graph_file = 0 as *mut FILE;
    (*self_0).cancellation_flag = 0 as *const size_t;
    (*self_0).timeout_duration = 0 as libc::c_int as TSDuration;
    (*self_0).end_clock = clock_null();
    (*self_0).operation_count = 0 as libc::c_int as libc::c_uint;
    (*self_0).old_tree = Subtree {
        ptr: 0 as *const SubtreeHeapData,
    };
    (*self_0).scratch_tree.ptr = &mut (*self_0).scratch_tree_data;
    (*self_0).included_range_differences = {
        let mut init = TSRangeArray {
            contents: 0 as *mut TSRange,
            size: 0 as libc::c_int as uint32_t,
            capacity: 0 as libc::c_int as uint32_t,
        };
        init
    };
    (*self_0).included_range_difference_index = 0 as libc::c_int as libc::c_uint;
    ts_parser__set_cached_token(
        self_0,
        0 as libc::c_int as size_t,
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        },
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        },
    );
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_delete(mut self_0: *mut TSParser) {
    if self_0.is_null() {
        return;
    }
    ts_parser_set_language(self_0, 0 as *const TSLanguage);
    ts_stack_delete((*self_0).stack);
    if !(*self_0).reduce_actions.contents.is_null() {
        array__delete(&mut (*self_0).reduce_actions as *mut ReduceActionSet as *mut VoidArray);
    }
    if !(*self_0).included_range_differences.contents.is_null() {
        array__delete(
            &mut (*self_0).included_range_differences as *mut TSRangeArray as *mut VoidArray,
        );
    }
    if !(*self_0).old_tree.ptr.is_null() {
        ts_subtree_release(&mut (*self_0).tree_pool, (*self_0).old_tree);
        (*self_0).old_tree = Subtree {
            ptr: 0 as *const SubtreeHeapData,
        }
    }
    ts_lexer_delete(&mut (*self_0).lexer);
    ts_parser__set_cached_token(
        self_0,
        0 as libc::c_int as size_t,
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        },
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        },
    );
    ts_subtree_pool_delete(&mut (*self_0).tree_pool);
    reusable_node_delete(&mut (*self_0).reusable_node);
    ts_free(self_0 as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_language(mut self_0: *const TSParser) -> *const TSLanguage {
    return (*self_0).language;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_set_language(
    mut self_0: *mut TSParser,
    mut language: *const TSLanguage,
) -> bool {
    if !language.is_null() {
        if (*language).version > 11 as libc::c_int as libc::c_uint {
            return 0 as libc::c_int != 0;
        }
        if (*language).version < 9 as libc::c_int as libc::c_uint {
            return 0 as libc::c_int != 0;
        }
    }
    if !(*self_0).external_scanner_payload.is_null()
        && (*(*self_0).language).external_scanner.destroy.is_some()
    {
        (*(*self_0).language)
            .external_scanner
            .destroy
            .expect("non-null function pointer")((*self_0).external_scanner_payload);
    }
    if !language.is_null() && (*language).external_scanner.create.is_some() {
        (*self_0).external_scanner_payload = (*language)
            .external_scanner
            .create
            .expect("non-null function pointer")()
    } else {
        (*self_0).external_scanner_payload = 0 as *mut libc::c_void
    }
    (*self_0).language = language;
    ts_parser_reset(self_0);
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_logger(mut self_0: *const TSParser) -> TSLogger {
    return (*self_0).lexer.logger;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_set_logger(mut self_0: *mut TSParser, mut logger: TSLogger) {
    (*self_0).lexer.logger = logger;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_print_dot_graphs(
    mut self_0: *mut TSParser,
    mut fd: libc::c_int,
) {
    if !(*self_0).dot_graph_file.is_null() {
        fclose((*self_0).dot_graph_file);
    }
    if fd >= 0 as libc::c_int {
        (*self_0).dot_graph_file = fdopen(fd, b"a\x00" as *const u8 as *const libc::c_char)
    } else {
        (*self_0).dot_graph_file = 0 as *mut FILE
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_cancellation_flag(mut self_0: *const TSParser) -> *const size_t {
    return (*self_0).cancellation_flag as *const size_t;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_set_cancellation_flag(
    mut self_0: *mut TSParser,
    mut flag: *const size_t,
) {
    (*self_0).cancellation_flag = flag as *const size_t;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_timeout_micros(mut self_0: *const TSParser) -> uint64_t {
    return duration_to_micros((*self_0).timeout_duration);
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_set_timeout_micros(
    mut self_0: *mut TSParser,
    mut timeout_micros: uint64_t,
) {
    (*self_0).timeout_duration = duration_from_micros(timeout_micros);
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_set_included_ranges(
    mut self_0: *mut TSParser,
    mut ranges: *const TSRange,
    mut count: uint32_t,
) -> bool {
    return ts_lexer_set_included_ranges(&mut (*self_0).lexer, ranges, count);
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_included_ranges(
    mut self_0: *const TSParser,
    mut count: *mut uint32_t,
) -> *const TSRange {
    return ts_lexer_included_ranges(&(*self_0).lexer, count);
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_reset(mut self_0: *mut TSParser) {
    if !(*self_0).language.is_null() && (*(*self_0).language).external_scanner.deserialize.is_some()
    {
        (*(*self_0).language)
            .external_scanner
            .deserialize
            .expect("non-null function pointer")(
            (*self_0).external_scanner_payload,
            0 as *const libc::c_char,
            0 as libc::c_int as libc::c_uint,
        );
    }
    if !(*self_0).old_tree.ptr.is_null() {
        ts_subtree_release(&mut (*self_0).tree_pool, (*self_0).old_tree);
        (*self_0).old_tree = Subtree {
            ptr: 0 as *const SubtreeHeapData,
        }
    }
    reusable_node_clear(&mut (*self_0).reusable_node);
    ts_lexer_reset(&mut (*self_0).lexer, length_zero());
    ts_stack_clear((*self_0).stack);
    ts_parser__set_cached_token(
        self_0,
        0 as libc::c_int as size_t,
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        },
        Subtree {
            ptr: 0 as *const SubtreeHeapData,
        },
    );
    if !(*self_0).finished_tree.ptr.is_null() {
        ts_subtree_release(&mut (*self_0).tree_pool, (*self_0).finished_tree);
        (*self_0).finished_tree = Subtree {
            ptr: 0 as *const SubtreeHeapData,
        }
    }
    (*self_0).accept_count = 0 as libc::c_int as libc::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_parse(
    mut self_0: *mut TSParser,
    mut old_tree: *const TSTree,
    mut input: TSInput,
) -> *mut TSTree {
    if (*self_0).language.is_null() || input.read.is_none() {
        return 0 as *mut TSTree;
    }
    ts_lexer_set_input(&mut (*self_0).lexer, input);
    (*self_0).included_range_differences.size = 0 as libc::c_int as uint32_t;
    (*self_0).included_range_difference_index = 0 as libc::c_int as libc::c_uint;
    if ts_parser_has_outstanding_parse(self_0) {
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"resume_parsing\x00" as *const u8 as *const libc::c_char,
            );
            ts_parser__log(self_0);
        }
    } else if !old_tree.is_null() {
        ts_subtree_retain((*old_tree).root);
        (*self_0).old_tree = (*old_tree).root;
        ts_range_array_get_changed_ranges(
            (*old_tree).included_ranges,
            (*old_tree).included_range_count,
            (*self_0).lexer.included_ranges,
            (*self_0).lexer.included_range_count as libc::c_uint,
            &mut (*self_0).included_range_differences,
        );
        reusable_node_reset(&mut (*self_0).reusable_node, (*old_tree).root);
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"parse_after_edit\x00" as *const u8 as *const libc::c_char,
            );
            ts_parser__log(self_0);
        }
        if !(*self_0).dot_graph_file.is_null() {
            ts_subtree_print_dot_graph(
                (*self_0).old_tree,
                (*self_0).language,
                (*self_0).dot_graph_file,
            );
            fputs(
                b"\n\x00" as *const u8 as *const libc::c_char,
                (*self_0).dot_graph_file,
            );
        }
        let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
        while i < (*self_0).included_range_differences.size {
            let mut range: *mut TSRange = &mut *(*self_0)
                .included_range_differences
                .contents
                .offset(i as isize) as *mut TSRange;
            if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                snprintf(
                    (*self_0).lexer.debug_buffer.as_mut_ptr(),
                    1024,
                    b"different_included_range %u - %u\x00" as *const u8 as *const libc::c_char,
                    (*range).start_byte,
                    (*range).end_byte,
                );
                ts_parser__log(self_0);
            }
            i = i.wrapping_add(1)
        }
    } else {
        reusable_node_clear(&mut (*self_0).reusable_node);
        if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
            snprintf(
                (*self_0).lexer.debug_buffer.as_mut_ptr(),
                1024,
                b"new_parse\x00" as *const u8 as *const libc::c_char,
            );
            ts_parser__log(self_0);
        }
    }
    let mut position: uint32_t = 0 as libc::c_int as uint32_t;
    let mut last_position: uint32_t = 0 as libc::c_int as uint32_t;
    let mut version_count: uint32_t = 0 as libc::c_int as uint32_t;
    (*self_0).operation_count = 0 as libc::c_int as libc::c_uint;
    if (*self_0).timeout_duration != 0 {
        (*self_0).end_clock = clock_after(clock_now(), (*self_0).timeout_duration)
    } else {
        (*self_0).end_clock = clock_null()
    }
    loop {
        let mut version: StackVersion = 0 as libc::c_int as StackVersion;
        loop {
            version_count = ts_stack_version_count((*self_0).stack);
            if !(version < version_count) {
                break;
            }
            let mut allow_node_reuse: bool = version_count == 1 as libc::c_int as libc::c_uint;
            while ts_stack_is_active((*self_0).stack, version) {
                if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
                    snprintf(
                        (*self_0).lexer.debug_buffer.as_mut_ptr(),
                        1024 as usize,
                        b"process version:%d, version_count:%u, state:%d, row:%u, col:%u\x00"
                            as *const u8 as *const libc::c_char,
                        version,
                        ts_stack_version_count((*self_0).stack),
                        ts_stack_state((*self_0).stack, version) as libc::c_int,
                        ts_stack_position((*self_0).stack, version)
                            .extent
                            .row
                            .wrapping_add(1 as libc::c_int as libc::c_uint),
                        ts_stack_position((*self_0).stack, version).extent.column,
                    );
                    ts_parser__log(self_0);
                }
                if !ts_parser__advance(self_0, version, allow_node_reuse) {
                    return 0 as *mut TSTree;
                }
                if !(*self_0).dot_graph_file.is_null() {
                    ts_stack_print_dot_graph(
                        (*self_0).stack,
                        (*self_0).language,
                        (*self_0).dot_graph_file,
                    );
                    fputs(
                        b"\n\n\x00" as *const u8 as *const libc::c_char,
                        (*self_0).dot_graph_file,
                    );
                }
                position = ts_stack_position((*self_0).stack, version).bytes;
                if !(position > last_position
                    || version > 0 as libc::c_int as libc::c_uint && position == last_position)
                {
                    continue;
                }
                last_position = position;
                break;
            }
            version = version.wrapping_add(1)
        }
        let mut min_error_cost: libc::c_uint = ts_parser__condense_stack(self_0);
        if !(*self_0).finished_tree.ptr.is_null()
            && ts_subtree_error_cost((*self_0).finished_tree) < min_error_cost
        {
            break;
        }
        while (*self_0).included_range_difference_index < (*self_0).included_range_differences.size
        {
            let mut range_0: *mut TSRange = &mut *(*self_0)
                .included_range_differences
                .contents
                .offset((*self_0).included_range_difference_index as isize)
                as *mut TSRange;
            if !((*range_0).end_byte <= position) {
                break;
            }
            (*self_0).included_range_difference_index =
                (*self_0).included_range_difference_index.wrapping_add(1)
        }
        if !(version_count != 0 as libc::c_int as libc::c_uint) {
            break;
        }
    }
    ts_subtree_balance(
        (*self_0).finished_tree,
        &mut (*self_0).tree_pool,
        (*self_0).language,
    );
    if (*self_0).lexer.logger.log.is_some() || !(*self_0).dot_graph_file.is_null() {
        snprintf(
            (*self_0).lexer.debug_buffer.as_mut_ptr(),
            1024,
            b"done\x00" as *const u8 as *const libc::c_char,
        );
        ts_parser__log(self_0);
    }
    if !(*self_0).dot_graph_file.is_null() {
        ts_subtree_print_dot_graph(
            (*self_0).finished_tree,
            (*self_0).language,
            (*self_0).dot_graph_file,
        );
        fputs(
            b"\n\x00" as *const u8 as *const libc::c_char,
            (*self_0).dot_graph_file,
        );
    }
    let mut result: *mut TSTree = ts_tree_new(
        (*self_0).finished_tree,
        (*self_0).language,
        (*self_0).lexer.included_ranges,
        (*self_0).lexer.included_range_count as libc::c_uint,
    );
    (*self_0).finished_tree = Subtree {
        ptr: 0 as *const SubtreeHeapData,
    };
    ts_parser_reset(self_0);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_parse_string(
    mut self_0: *mut TSParser,
    mut old_tree: *const TSTree,
    mut string: *const libc::c_char,
    mut length: uint32_t,
) -> *mut TSTree {
    return ts_parser_parse_string_encoding(self_0, old_tree, string, length, TSInputEncodingUTF8);
}
#[no_mangle]
pub unsafe extern "C" fn ts_parser_parse_string_encoding(
    mut self_0: *mut TSParser,
    mut old_tree: *const TSTree,
    mut string: *const libc::c_char,
    mut length: uint32_t,
    mut encoding: TSInputEncoding,
) -> *mut TSTree {
    let mut input: TSStringInput = {
        let mut init = TSStringInput {
            string: string,
            length: length,
        };
        init
    };
    return ts_parser_parse(self_0, old_tree, {
        let mut init = TSInput {
            payload: &mut input as *mut TSStringInput as *mut libc::c_void,
            read: Some(
                ts_string_input_read
                    as unsafe extern "C" fn(
                        _: *mut libc::c_void,
                        _: uint32_t,
                        _: TSPoint,
                        _: *mut uint32_t,
                    ) -> *const libc::c_char,
            ),
            encoding: encoding,
        };
        init
    });
}
