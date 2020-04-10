use crate::{
    util::{iswalnum, iswspace, strcmp, strncmp, WrappingOffsetFromExt},
    *,
};

use std::{
    ffi, os,
    ptr::{self, copy_nonoverlapping, write_bytes},
};

static mut PARENT_DONE: TSQueryError = 4_294_967_295 as TSQueryError;
static mut PATTERN_DONE_MARKER: u8 = 255 as os::raw::c_int as u8;
static mut NONE: u16 = 65535 as os::raw::c_int as u16;
static mut WILDCARD_SYMBOL: TSSymbol = 0 as os::raw::c_int as TSSymbol;
static mut NAMED_WILDCARD_SYMBOL: TSSymbol =
    (65535 as os::raw::c_int - 1 as os::raw::c_int) as TSSymbol;
static mut MAX_STATE_COUNT: u16 = 32 as os::raw::c_int as u16;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CaptureListPool {
    pub list: TSQueryCaptureArray,
    pub usage_map: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryCaptureArray {
    pub contents: *mut TSQueryCapture,
    pub size: u32,
    pub capacity: u32,
}

/*
 * TSQueryCursor - A stateful struct used to execute a query on a tree.
 */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryCursor {
    pub query: *const TSQuery,
    pub cursor: TSTreeCursor,
    pub states: TSQueryCursorStates,
    pub finished_states: TsQueryCursorFinishedStated,
    pub capture_list_pool: CaptureListPool,
    pub depth: u32,
    pub start_byte: u32,
    pub end_byte: u32,
    pub next_state_id: u32,
    pub start_point: TSPoint,
    pub end_point: TSPoint,
    pub ascending: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Stream {
    pub input: *const os::raw::c_char,
    pub end: *const os::raw::c_char,
    pub next: i32,
    pub next_size: u8,
}

#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct QueryStep {
    pub symbol: TSSymbol,
    pub field: TSFieldId,
    pub capture_ids: [u16; 4],
    #[bitfield(name = "depth", ty = "u16", bits = "0..=12")]
    #[bitfield(name = "contains_captures", ty = "bool", bits = "13..=13")]
    #[bitfield(name = "is_immediate", ty = "bool", bits = "14..=14")]
    #[bitfield(name = "is_last", ty = "bool", bits = "15..=15")]
    pub depth_contains_captures_is_immediate_is_last: [u8; 2],
}

/*
 * TSQuery - A tree query, compiled from a string of S-expressions. The query
 * itself is immutable. The mutable state used in the process of executing the
 * query is stored in a `TSQueryCursor`.
 */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQuery {
    pub captures: SymbolTable,
    pub predicate_values: SymbolTable,
    pub steps: TSQuerySteps,
    pub pattern_map: TSQueryPatternMap,
    pub predicate_steps: TSQueryPredicateSteps,
    pub predicates_by_pattern: TSQueryPredicatesByPattern,
    pub start_bytes_by_pattern: TSQueryStartBytesByPattern,
    pub language: *const TSLanguage,
    pub max_capture_count: u16,
    pub wildcard_root_pattern_count: u16,
    pub symbol_map: *mut TSSymbol,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryStartBytesByPattern {
    pub contents: *mut u32,
    pub size: u32,
    pub capacity: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Slice {
    pub offset: u32,
    pub length: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SymbolTable {
    pub characters: SymbolTableCharacters,
    pub slices: SymbolTableSlices,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SymbolTableSlices {
    pub contents: *mut Slice,
    pub size: u32,
    pub capacity: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SymbolTableCharacters {
    pub contents: *mut os::raw::c_char,
    pub size: u32,
    pub capacity: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryPredicatesByPattern {
    pub contents: *mut Slice,
    pub size: u32,
    pub capacity: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryPredicateSteps {
    pub contents: *mut TSQueryPredicateStep,
    pub size: u32,
    pub capacity: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryPatternMap {
    pub contents: *mut PatternEntry,
    pub size: u32,
    pub capacity: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PatternEntry {
    pub step_index: u16,
    pub pattern_index: u16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQuerySteps {
    pub contents: *mut QueryStep,
    pub size: u32,
    pub capacity: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TsQueryCursorFinishedStated {
    pub contents: *mut QueryState,
    pub size: u32,
    pub capacity: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct QueryState {
    pub start_depth: u16,
    pub pattern_index: u16,
    pub step_index: u16,
    pub capture_count: u16,
    pub capture_list_id: u16,
    pub consumed_capture_count: u16,
    pub id: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryCursorStates {
    pub contents: *mut QueryState,
    pub size: u32,
    pub capacity: u32,
}

/* *********
 * Stream
 **********/
// Advance to the next unicode code point in the stream.
unsafe extern "C" fn stream_advance(mut self_0: *mut Stream) -> bool {
    (*self_0).input = (*self_0)
        .input
        .offset((*self_0).next_size as os::raw::c_int as isize);
    if (*self_0).input < (*self_0).end {
        let mut size: u32 = ts_decode_utf8(
            (*self_0).input as *const u8,
            (*self_0).end.wrapping_offset_from_((*self_0).input) as os::raw::c_long as u32,
            &mut (*self_0).next,
        );
        if size > 0 as os::raw::c_int as os::raw::c_uint {
            (*self_0).next_size = size as u8;
            return true;
        }
    } else {
        (*self_0).next_size = 0 as os::raw::c_int as u8;
        (*self_0).next = '\u{0}' as i32
    }
    false
}
// Reset the stream to the given input position, represented as a pointer
// into the input string.
unsafe extern "C" fn stream_reset(mut self_0: *mut Stream, mut input: *const os::raw::c_char) {
    (*self_0).input = input;
    (*self_0).next_size = 0 as os::raw::c_int as u8;
    stream_advance(self_0);
}
unsafe extern "C" fn stream_new(mut string: *const os::raw::c_char, mut length: u32) -> Stream {
    let mut self_0 = Stream {
        input: string,
        end: string.offset(length as isize),
        next: 0 as os::raw::c_int,
        next_size: 0,
    };
    stream_advance(&mut self_0);
    self_0
}
unsafe extern "C" fn stream_skip_whitespace(mut stream: *mut Stream) {
    loop {
        if iswspace((*stream).next as wint_t) != 0 {
            stream_advance(stream);
        } else {
            if (*stream).next != ';' as i32 {
                break;
            }
            // skip over comments
            stream_advance(stream);
            #[allow(clippy::while_immutable_condition)]
            while (*stream).next != 0 && (*stream).next != '\n' as i32 {
                if !stream_advance(stream) {
                    break;
                }
            }
        }
    }
}
unsafe extern "C" fn stream_is_ident_start(mut stream: *mut Stream) -> bool {
    iswalnum((*stream).next as wint_t) != 0
        || (*stream).next == '_' as i32
        || (*stream).next == '-' as i32
}
unsafe extern "C" fn stream_scan_identifier(mut stream: *mut Stream) {
    loop {
        stream_advance(stream);
        if !(iswalnum((*stream).next as wint_t) != 0
            || (*stream).next == '_' as i32
            || (*stream).next == '-' as i32
            || (*stream).next == '.' as i32
            || (*stream).next == '?' as i32
            || (*stream).next == '!' as i32)
        {
            break;
        }
    }
}

/* *****************
 * CaptureListPool
 ******************/
unsafe extern "C" fn capture_list_pool_new() -> CaptureListPool {
    CaptureListPool {
        list: TSQueryCaptureArray {
            contents: std::ptr::null_mut::<TSQueryCapture>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        usage_map: 4_294_967_295 as os::raw::c_uint,
    }
}
unsafe extern "C" fn capture_list_pool_reset(mut self_0: *mut CaptureListPool, mut list_size: u16) {
    (*self_0).usage_map = 4_294_967_295 as os::raw::c_uint;
    let mut total_size: u32 =
        (MAX_STATE_COUNT as os::raw::c_int * list_size as os::raw::c_int) as u32;
    array__reserve(
        &mut (*self_0).list as *mut TSQueryCaptureArray as *mut VoidArray,
        ::std::mem::size_of::<TSQueryCapture>(),
        total_size,
    );
    (*self_0).list.size = total_size;
}
unsafe extern "C" fn capture_list_pool_delete(mut self_0: *mut CaptureListPool) {
    array__delete(&mut (*self_0).list as *mut TSQueryCaptureArray as *mut VoidArray);
}
unsafe extern "C" fn capture_list_pool_get(
    mut self_0: *mut CaptureListPool,
    mut id: u16,
) -> *mut TSQueryCapture {
    &mut *(*self_0).list.contents.offset(
        (id as os::raw::c_uint).wrapping_mul(
            (*self_0)
                .list
                .size
                .wrapping_div(MAX_STATE_COUNT as os::raw::c_uint),
        ) as isize,
    ) as *mut TSQueryCapture
}
unsafe extern "C" fn capture_list_pool_is_empty(mut self_0: *const CaptureListPool) -> bool {
    (*self_0).usage_map == 0 as os::raw::c_int as os::raw::c_uint
}
unsafe extern "C" fn capture_list_pool_acquire(mut self_0: *mut CaptureListPool) -> u16 {
    // In the usage_map bitmask, ones represent free lists, and zeros represent
    // lists that are in use. A free list id can quickly be found by counting
    // the leading zeros in the usage map. An id of zero corresponds to the
    // highest-order bit in the bitmask.
    let mut id: u16 = count_leading_zeros((*self_0).usage_map) as u16;
    if id as os::raw::c_int == 32 as os::raw::c_int {
        return NONE;
    }
    (*self_0).usage_map &= !bitmask_for_index(id);
    id
}
unsafe extern "C" fn capture_list_pool_release(mut self_0: *mut CaptureListPool, mut id: u16) {
    (*self_0).usage_map |= bitmask_for_index(id);
}

/* *************
 * SymbolTable
 **************/
unsafe extern "C" fn symbol_table_new() -> SymbolTable {
    SymbolTable {
        characters: SymbolTableCharacters {
            contents: ptr::null_mut(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        slices: SymbolTableSlices {
            contents: std::ptr::null_mut::<Slice>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
    }
}
unsafe extern "C" fn symbol_table_delete(mut self_0: *mut SymbolTable) {
    array__delete(&mut (*self_0).characters as *mut SymbolTableCharacters as *mut VoidArray);
    array__delete(&mut (*self_0).slices as *mut SymbolTableSlices as *mut VoidArray);
}
unsafe extern "C" fn symbol_table_id_for_name(
    mut self_0: *const SymbolTable,
    mut name: *const os::raw::c_char,
    mut length: u32,
) -> os::raw::c_int {
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < (*self_0).slices.size {
        let mut slice: Slice = *(*self_0).slices.contents.offset(i as isize);
        if slice.length == length
            && strncmp(
                &*(*self_0).characters.contents.offset(slice.offset as isize),
                name,
                length as usize,
            ) == 0
        {
            return i as os::raw::c_int;
        }
        i = i.wrapping_add(1)
    }
    -(1 as os::raw::c_int)
}
unsafe extern "C" fn symbol_table_name_for_id(
    mut self_0: *const SymbolTable,
    mut id: u16,
    mut length: *mut u32,
) -> *const os::raw::c_char {
    let mut slice: Slice = *(*self_0).slices.contents.offset(id as isize);
    *length = slice.length;
    &mut *(*self_0).characters.contents.offset(slice.offset as isize) as *mut os::raw::c_char
}
unsafe extern "C" fn symbol_table_insert_name(
    mut self_0: *mut SymbolTable,
    mut name: *const os::raw::c_char,
    mut length: u32,
) -> u16 {
    let mut id: os::raw::c_int = symbol_table_id_for_name(self_0, name, length);
    if id >= 0 as os::raw::c_int {
        return id as u16;
    }
    let mut slice = Slice {
        offset: (*self_0).characters.size,
        length,
    };
    array__grow(
        &mut (*self_0).characters as *mut SymbolTableCharacters as *mut VoidArray,
        length.wrapping_add(1) as usize,
        ::std::mem::size_of::<os::raw::c_char>(),
    );
    write_bytes(
        (*self_0)
            .characters
            .contents
            .offset((*self_0).characters.size as isize),
        0,
        length.wrapping_add(1) as usize,
    );
    (*self_0).characters.size = ((*self_0).characters.size as os::raw::c_uint)
        .wrapping_add(length.wrapping_add(1)) as u32 as u32;
    copy_nonoverlapping(
        name,
        &mut *(*self_0).characters.contents.offset(slice.offset as isize) as *mut os::raw::c_char,
        length as usize,
    );
    *(*self_0)
        .characters
        .contents
        .offset((*self_0).characters.size.wrapping_sub(1) as isize) =
        0 as os::raw::c_int as os::raw::c_char;
    array__grow(
        &mut (*self_0).slices as *mut SymbolTableSlices as *mut VoidArray,
        1 as os::raw::c_int as usize,
        ::std::mem::size_of::<Slice>(),
    );
    let fresh1 = (*self_0).slices.size;
    (*self_0).slices.size = (*self_0).slices.size.wrapping_add(1);
    *(*self_0).slices.contents.offset(fresh1 as isize) = slice;
    (*self_0).slices.size.wrapping_sub(1) as u16
}
unsafe extern "C" fn symbol_table_insert_name_with_escapes(
    mut self_0: *mut SymbolTable,
    mut escaped_name: *const os::raw::c_char,
    mut escaped_length: u32,
) -> u16 {
    let mut slice = Slice {
        offset: (*self_0).characters.size,
        length: 0 as os::raw::c_int as u32,
    };
    array__grow(
        &mut (*self_0).characters as *mut SymbolTableCharacters as *mut VoidArray,
        escaped_length.wrapping_add(1) as usize,
        ::std::mem::size_of::<os::raw::c_char>(),
    );
    write_bytes(
        (*self_0)
            .characters
            .contents
            .offset((*self_0).characters.size as isize),
        0,
        escaped_length.wrapping_add(1) as usize,
    );
    (*self_0).characters.size = ((*self_0).characters.size as os::raw::c_uint)
        .wrapping_add(escaped_length.wrapping_add(1)) as u32 as u32;
    // Copy the contents of the literal into the characters buffer, processing escape
    // sequences like \n and \". This needs to be done before checking if the literal
    // is already present, in order to do the string comparison.
    let mut is_escaped: bool = false;
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < escaped_length {
        let mut src: *const os::raw::c_char =
            &*escaped_name.offset(i as isize) as *const os::raw::c_char;
        let mut dest: *mut os::raw::c_char = &mut *(*self_0)
            .characters
            .contents
            .offset(slice.offset.wrapping_add(slice.length) as isize)
            as *mut os::raw::c_char;
        if is_escaped {
            match *src as os::raw::c_int {
                110 => *dest = '\n' as i32 as os::raw::c_char,
                114 => *dest = '\r' as i32 as os::raw::c_char,
                116 => *dest = '\t' as i32 as os::raw::c_char,
                48 => *dest = '\u{0}' as i32 as os::raw::c_char,
                _ => *dest = *src,
            }
            is_escaped = false;
            slice.length = slice.length.wrapping_add(1)
        } else if *src as os::raw::c_int == '\\' as i32 {
            is_escaped = true
        } else {
            *dest = *src;
            slice.length = slice.length.wrapping_add(1)
        }
        i = i.wrapping_add(1)
    }
    // If the string is already present, remove the redundant content from the characters
    // buffer and return the existing id.
    let mut id: os::raw::c_int = symbol_table_id_for_name(
        self_0,
        &*(*self_0).characters.contents.offset(slice.offset as isize),
        slice.length,
    );
    if id >= 0 as os::raw::c_int {
        (*self_0).characters.size = ((*self_0).characters.size as os::raw::c_uint)
            .wrapping_sub(escaped_length.wrapping_add(1)) as u32
            as u32;
        return id as u16;
    }
    *(*self_0)
        .characters
        .contents
        .offset(slice.offset.wrapping_add(slice.length) as isize) =
        0 as os::raw::c_int as os::raw::c_char;
    array__grow(
        &mut (*self_0).slices as *mut SymbolTableSlices as *mut VoidArray,
        1 as os::raw::c_int as usize,
        ::std::mem::size_of::<Slice>(),
    );
    let fresh2 = (*self_0).slices.size;
    (*self_0).slices.size = (*self_0).slices.size.wrapping_add(1);
    *(*self_0).slices.contents.offset(fresh2 as isize) = slice;
    (*self_0).slices.size.wrapping_sub(1) as u16
}

/* ***********
 * QueryStep
 ************/
unsafe extern "C" fn query_step__new(
    mut symbol: TSSymbol,
    mut depth: u16,
    mut is_immediate: bool,
) -> QueryStep {
    let mut init = QueryStep {
        depth_contains_captures_is_immediate_is_last: [0; 2],
        symbol,
        field: 0 as os::raw::c_int as TSFieldId,
        capture_ids: [NONE, NONE, NONE, NONE],
    };
    init.set_depth(depth);
    init.set_contains_captures(false);
    init.set_is_immediate(is_immediate);
    init.set_is_last(false);
    init
}
unsafe extern "C" fn query_step__add_capture(mut self_0: *mut QueryStep, mut capture_id: u16) {
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < 4 as os::raw::c_int as os::raw::c_uint {
        if (*self_0).capture_ids[i as usize] as os::raw::c_int == NONE as os::raw::c_int {
            (*self_0).capture_ids[i as usize] = capture_id;
            break;
        } else {
            i = i.wrapping_add(1)
        }
    }
}
unsafe extern "C" fn query_step__remove_capture(mut self_0: *mut QueryStep, mut capture_id: u16) {
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < 4 as os::raw::c_int as os::raw::c_uint {
        if (*self_0).capture_ids[i as usize] as os::raw::c_int == capture_id as os::raw::c_int {
            (*self_0).capture_ids[i as usize] = NONE;
            while i.wrapping_add(1) < 4 as os::raw::c_int as os::raw::c_uint {
                if (*self_0).capture_ids[i.wrapping_add(1) as usize] as os::raw::c_int
                    == NONE as os::raw::c_int
                {
                    break;
                }
                (*self_0).capture_ids[i as usize] =
                    (*self_0).capture_ids[i.wrapping_add(1) as usize];
                (*self_0).capture_ids[i.wrapping_add(1) as usize] = NONE;
                i = i.wrapping_add(1)
            }
            break;
        } else {
            i = i.wrapping_add(1)
        }
    }
}

/* ********
 * Query
 *********/
// The `pattern_map` contains a mapping from TSSymbol values to indices in the
// `steps` array. For a given syntax node, the `pattern_map` makes it possible
// to quickly find the starting steps of all of the patterns whose root matches
// that node. Each entry has two fields: a `pattern_index`, which identifies one
// of the patterns in the query, and a `step_index`, which indicates the start
// offset of that pattern's steps within the `steps` array.
//
// The entries are sorted by the patterns' root symbols, and lookups use a
// binary search. This ensures that the cost of this initial lookup step
// scales logarithmically with the number of patterns in the query.
//
// This returns `true` if the symbol is present and `false` otherwise.
// If the symbol is not present `*result` is set to the index where the
// symbol should be inserted.
#[inline]
unsafe extern "C" fn ts_query__pattern_map_search(
    mut self_0: *const TSQuery,
    mut needle: TSSymbol,
    mut result: *mut u32,
) -> bool {
    let mut base_index: u32 = (*self_0).wildcard_root_pattern_count as u32;
    let mut size: u32 = (*self_0).pattern_map.size.wrapping_sub(base_index);
    if size == 0 as os::raw::c_int as os::raw::c_uint {
        *result = base_index;
        return false;
    }
    while size > 1 as os::raw::c_int as os::raw::c_uint {
        let mut half_size: u32 = size.wrapping_div(2);
        let mut mid_index: u32 = base_index.wrapping_add(half_size);
        let mut mid_symbol: TSSymbol = (*(*self_0).steps.contents.offset(
            (*(*self_0).pattern_map.contents.offset(mid_index as isize)).step_index as isize,
        ))
        .symbol;
        if needle as os::raw::c_int > mid_symbol as os::raw::c_int {
            base_index = mid_index
        }
        size = (size as os::raw::c_uint).wrapping_sub(half_size) as u32 as u32
    }
    let mut symbol: TSSymbol = (*(*self_0)
        .steps
        .contents
        .offset((*(*self_0).pattern_map.contents.offset(base_index as isize)).step_index as isize))
    .symbol;
    if needle as os::raw::c_int > symbol as os::raw::c_int {
        base_index = base_index.wrapping_add(1);
        if base_index < (*self_0).pattern_map.size {
            symbol = (*(*self_0).steps.contents.offset(
                (*(*self_0).pattern_map.contents.offset(base_index as isize)).step_index as isize,
            ))
            .symbol
        }
    }
    *result = base_index;
    needle as os::raw::c_int == symbol as os::raw::c_int
}
// Insert a new pattern's start index into the pattern map, maintaining
// the pattern map's ordering invariant.
#[inline]
unsafe extern "C" fn ts_query__pattern_map_insert(
    mut self_0: *mut TSQuery,
    mut symbol: TSSymbol,
    mut start_step_index: u32,
) {
    let mut index: u32 = 0;
    ts_query__pattern_map_search(self_0, symbol, &mut index);
    array__splice(
        &mut (*self_0).pattern_map as *mut TSQueryPatternMap as *mut VoidArray,
        ::std::mem::size_of::<PatternEntry>(),
        index,
        0 as os::raw::c_int as u32,
        1 as os::raw::c_int as u32,
        &mut PatternEntry {
            step_index: start_step_index as u16,
            pattern_index: (*self_0).pattern_map.size as u16,
        } as *mut PatternEntry as *const ffi::c_void,
    );
}
unsafe extern "C" fn ts_query__finalize_steps(mut self_0: *mut TSQuery) {
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < (*self_0).steps.size {
        let mut step: *mut QueryStep =
            &mut *(*self_0).steps.contents.offset(i as isize) as *mut QueryStep;
        let mut depth: u32 = (*step).depth() as u32;
        if (*step).capture_ids[0 as os::raw::c_int as usize] as os::raw::c_int
            != NONE as os::raw::c_int
        {
            (*step).set_contains_captures(true)
        } else {
            (*step).set_contains_captures(false);
            let mut j: os::raw::c_uint = i.wrapping_add(1);
            while j < (*self_0).steps.size {
                let mut s: *mut QueryStep =
                    &mut *(*self_0).steps.contents.offset(j as isize) as *mut QueryStep;
                if (*s).depth() as os::raw::c_int == PATTERN_DONE_MARKER as os::raw::c_int
                    || (*s).depth() as os::raw::c_uint <= depth
                {
                    break;
                }
                if (*s).capture_ids[0 as os::raw::c_int as usize] as os::raw::c_int
                    != NONE as os::raw::c_int
                {
                    (*step).set_contains_captures(true)
                }
                j = j.wrapping_add(1)
            }
        }
        i = i.wrapping_add(1)
    }
}
// Parse a single predicate associated with a pattern, adding it to the
// query's internal `predicate_steps` array. Predicates are arbitrary
// S-expressions associated with a pattern which are meant to be handled at
// a higher level of abstraction, such as the Rust/JavaScript bindings. They
// can contain '@'-prefixed capture names, double-quoted strings, and bare
// symbols, which also represent strings.
unsafe extern "C" fn ts_query__parse_predicate(
    mut self_0: *mut TSQuery,
    mut stream: *mut Stream,
) -> TSQueryError {
    if (*stream).next == ')' as i32 {
        return PARENT_DONE;
    }
    if (*stream).next != '(' as i32 {
        return TSQueryErrorSyntax;
    }
    stream_advance(stream);
    stream_skip_whitespace(stream);
    let mut step_count: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    loop {
        if (*stream).next == ')' as i32 {
            stream_advance(stream);
            stream_skip_whitespace(stream);
            assert!(
                (*self_0)
                    .predicates_by_pattern
                    .size
                    .wrapping_sub(1 as os::raw::c_int as os::raw::c_uint)
                    < (*self_0).predicates_by_pattern.size
            );
            let fresh3 = &mut (*(&mut *(*self_0)
                .predicates_by_pattern
                .contents
                .offset((*self_0).predicates_by_pattern.size.wrapping_sub(1) as isize)
                as *mut Slice))
                .length;
            *fresh3 = (*fresh3).wrapping_add(1);
            array__grow(
                &mut (*self_0).predicate_steps as *mut TSQueryPredicateSteps as *mut VoidArray,
                1 as os::raw::c_int as usize,
                ::std::mem::size_of::<TSQueryPredicateStep>(),
            );
            let fresh4 = (*self_0).predicate_steps.size;
            (*self_0).predicate_steps.size = (*self_0).predicate_steps.size.wrapping_add(1);
            *(*self_0).predicate_steps.contents.offset(fresh4 as isize) = TSQueryPredicateStep {
                type_0: TSQueryPredicateStepTypeDone,
                value_id: 0 as os::raw::c_int as u32,
            };
            break;
        } else {
            // Parse an '@'-prefixed capture name
            if (*stream).next == '@' as i32 {
                stream_advance(stream);
                // Parse the capture name
                if !stream_is_ident_start(stream) {
                    return TSQueryErrorSyntax;
                }
                let mut capture_name: *const os::raw::c_char = (*stream).input;
                stream_scan_identifier(stream);
                let mut length: u32 =
                    (*stream).input.wrapping_offset_from_(capture_name) as os::raw::c_long as u32;
                // Add the capture id to the first step of the pattern
                let mut capture_id: os::raw::c_int =
                    symbol_table_id_for_name(&(*self_0).captures, capture_name, length);
                if capture_id == -(1 as os::raw::c_int) {
                    stream_reset(stream, capture_name);
                    return TSQueryErrorCapture;
                }
                assert!(
                    (*self_0)
                        .predicates_by_pattern
                        .size
                        .wrapping_sub(1 as os::raw::c_int as os::raw::c_uint)
                        < (*self_0).predicates_by_pattern.size
                );
                let fresh5 = &mut (*(&mut *(*self_0)
                    .predicates_by_pattern
                    .contents
                    .offset((*self_0).predicates_by_pattern.size.wrapping_sub(1) as isize)
                    as *mut Slice))
                    .length;
                *fresh5 = (*fresh5).wrapping_add(1);
                array__grow(
                    &mut (*self_0).predicate_steps as *mut TSQueryPredicateSteps as *mut VoidArray,
                    1 as os::raw::c_int as usize,
                    ::std::mem::size_of::<TSQueryPredicateStep>(),
                );
                let fresh6 = (*self_0).predicate_steps.size;
                (*self_0).predicate_steps.size = (*self_0).predicate_steps.size.wrapping_add(1);
                *(*self_0).predicate_steps.contents.offset(fresh6 as isize) = TSQueryPredicateStep {
                    type_0: TSQueryPredicateStepTypeCapture,
                    value_id: capture_id as u32,
                }
            } else if (*stream).next == '\"' as i32 {
                stream_advance(stream);
                // Parse a string literal
                // Parse the string content
                let mut is_escaped: bool = false;
                let mut string_content: *const os::raw::c_char = (*stream).input;
                loop {
                    if is_escaped {
                        is_escaped = false
                    } else if (*stream).next == '\\' as i32 {
                        is_escaped = true
                    } else {
                        if (*stream).next == '\"' as i32 {
                            break;
                        }
                        if (*stream).next == '\n' as i32 {
                            stream_reset(
                                stream,
                                string_content.offset(-(1 as os::raw::c_int as isize)),
                            );
                            return TSQueryErrorSyntax;
                        }
                    }
                    if !stream_advance(stream) {
                        stream_reset(
                            stream,
                            string_content.offset(-(1 as os::raw::c_int as isize)),
                        );
                        return TSQueryErrorSyntax;
                    }
                }
                let mut length_0: u32 =
                    (*stream).input.wrapping_offset_from_(string_content) as os::raw::c_long as u32;
                // Add a step for the node
                let mut id: u16 = symbol_table_insert_name_with_escapes(
                    &mut (*self_0).predicate_values,
                    string_content,
                    length_0,
                );
                assert!(
                    (*self_0)
                        .predicates_by_pattern
                        .size
                        .wrapping_sub(1 as os::raw::c_int as os::raw::c_uint)
                        < (*self_0).predicates_by_pattern.size
                );
                let fresh7 = &mut (*(&mut *(*self_0)
                    .predicates_by_pattern
                    .contents
                    .offset((*self_0).predicates_by_pattern.size.wrapping_sub(1) as isize)
                    as *mut Slice))
                    .length;
                *fresh7 = (*fresh7).wrapping_add(1);
                array__grow(
                    &mut (*self_0).predicate_steps as *mut TSQueryPredicateSteps as *mut VoidArray,
                    1 as os::raw::c_int as usize,
                    ::std::mem::size_of::<TSQueryPredicateStep>(),
                );
                let fresh8 = (*self_0).predicate_steps.size;
                (*self_0).predicate_steps.size = (*self_0).predicate_steps.size.wrapping_add(1);
                *(*self_0).predicate_steps.contents.offset(fresh8 as isize) =
                    TSQueryPredicateStep {
                        type_0: TSQueryPredicateStepTypeString,
                        value_id: id as u32,
                    };
                if (*stream).next != '\"' as i32 {
                    return TSQueryErrorSyntax;
                }
                stream_advance(stream);
            } else if stream_is_ident_start(stream) {
                let mut symbol_start: *const os::raw::c_char = (*stream).input;
                stream_scan_identifier(stream);
                let mut length_1: u32 =
                    (*stream).input.wrapping_offset_from_(symbol_start) as os::raw::c_long as u32;
                let mut id_0: u16 = symbol_table_insert_name(
                    &mut (*self_0).predicate_values,
                    symbol_start,
                    length_1,
                );
                assert!(
                    (*self_0)
                        .predicates_by_pattern
                        .size
                        .wrapping_sub(1 as os::raw::c_int as os::raw::c_uint)
                        < (*self_0).predicates_by_pattern.size
                );
                let fresh9 = &mut (*(&mut *(*self_0)
                    .predicates_by_pattern
                    .contents
                    .offset((*self_0).predicates_by_pattern.size.wrapping_sub(1) as isize)
                    as *mut Slice))
                    .length;
                *fresh9 = (*fresh9).wrapping_add(1);
                array__grow(
                    &mut (*self_0).predicate_steps as *mut TSQueryPredicateSteps as *mut VoidArray,
                    1 as os::raw::c_int as usize,
                    ::std::mem::size_of::<TSQueryPredicateStep>(),
                );
                let fresh10 = (*self_0).predicate_steps.size;
                (*self_0).predicate_steps.size = (*self_0).predicate_steps.size.wrapping_add(1);
                *(*self_0).predicate_steps.contents.offset(fresh10 as isize) =
                    TSQueryPredicateStep {
                        type_0: TSQueryPredicateStepTypeString,
                        value_id: id_0 as u32,
                    }
            } else {
                return TSQueryErrorSyntax;
            }
            step_count = step_count.wrapping_add(1);
            stream_skip_whitespace(stream);
        }
    }
    TSQueryErrorNone
}
// Parse a bare symbol
// Read one S-expression pattern from the stream, and incorporate it into
// the query's internal state machine representation. For nested patterns,
// this function calls itself recursively.
unsafe extern "C" fn ts_query__parse_pattern(
    mut self_0: *mut TSQuery,
    mut stream: *mut Stream,
    mut depth: u32,
    mut capture_count: *mut u32,
    mut is_immediate: bool,
) -> TSQueryError {
    let mut starting_step_index: u16 = (*self_0).steps.size as u16;
    if (*stream).next == 0 as os::raw::c_int {
        return TSQueryErrorSyntax;
    }
    // Finish the parent S-expression
    if (*stream).next == ')' as i32 {
        return PARENT_DONE;
    } else {
        // Parse a parenthesized node expression
        if (*stream).next == '(' as i32 {
            stream_advance(stream);
            stream_skip_whitespace(stream);
            // Parse a nested list, which represents a pattern followed by
            // zero-or-more predicates.
            if (*stream).next == '(' as i32 && depth == 0 as os::raw::c_int as os::raw::c_uint {
                let mut e: TSQueryError = ts_query__parse_pattern(
                    self_0,
                    stream,
                    0 as os::raw::c_int as u32,
                    capture_count,
                    is_immediate,
                );
                if e as u64 != 0 {
                    return e;
                }
                // Parse the predicates.
                stream_skip_whitespace(stream);
                loop {
                    let mut e_0: TSQueryError = ts_query__parse_predicate(self_0, stream);
                    if e_0 as os::raw::c_uint == PARENT_DONE as os::raw::c_uint {
                        stream_advance(stream);
                        stream_skip_whitespace(stream);
                        return TSQueryErrorNone;
                    } else if e_0 as u64 != 0 {
                        return e_0;
                    }
                }
            }
            let mut symbol: TSSymbol = 0;
            // Parse the wildcard symbol
            if (*stream).next == '*' as i32 {
                symbol = if depth > 0 as os::raw::c_int as os::raw::c_uint {
                    NAMED_WILDCARD_SYMBOL as os::raw::c_int
                } else {
                    WILDCARD_SYMBOL as os::raw::c_int
                } as TSSymbol;
                stream_advance(stream);
            } else if stream_is_ident_start(stream) {
                let mut node_name: *const os::raw::c_char = (*stream).input;
                stream_scan_identifier(stream);
                let mut length: u32 =
                    (*stream).input.wrapping_offset_from_(node_name) as os::raw::c_long as u32;
                symbol = ts_language_symbol_for_name((*self_0).language, node_name, length, true);
                if symbol == 0 {
                    stream_reset(stream, node_name);
                    return TSQueryErrorNodeType;
                }
            } else {
                return TSQueryErrorSyntax;
            }
            // Parse a normal node name
            // Add a step for the node.
            array__grow(
                &mut (*self_0).steps as *mut TSQuerySteps as *mut VoidArray,
                1 as os::raw::c_int as usize,
                ::std::mem::size_of::<QueryStep>(),
            );
            let fresh11 = (*self_0).steps.size;
            (*self_0).steps.size = (*self_0).steps.size.wrapping_add(1);
            *(*self_0).steps.contents.offset(fresh11 as isize) =
                query_step__new(symbol, depth as u16, is_immediate);
            // Parse the child patterns
            stream_skip_whitespace(stream);
            let mut child_is_immediate: bool = false;
            let mut child_start_step_index: u16 = (*self_0).steps.size as u16;
            loop {
                if (*stream).next == '.' as i32 {
                    child_is_immediate = true;
                    stream_advance(stream);
                    stream_skip_whitespace(stream);
                }
                let mut e_1: TSQueryError = ts_query__parse_pattern(
                    self_0,
                    stream,
                    depth.wrapping_add(1),
                    capture_count,
                    child_is_immediate,
                );
                if e_1 as os::raw::c_uint == PARENT_DONE as os::raw::c_uint {
                    if child_is_immediate {
                        let fresh12 = &mut (*(*self_0)
                            .steps
                            .contents
                            .offset(child_start_step_index as isize));
                        (*fresh12).set_is_last(true)
                    }
                    stream_advance(stream);
                    break;
                } else {
                    if e_1 as u64 != 0 {
                        return e_1;
                    }
                    child_is_immediate = false
                }
            }
        } else if (*stream).next == '\"' as i32 {
            stream_advance(stream);
            // Parse a double-quoted anonymous leaf node expression
            // Parse the string content
            let mut string_content: *const os::raw::c_char = (*stream).input;
            #[allow(clippy::while_immutable_condition)]
            while (*stream).next != '\"' as i32 {
                if !stream_advance(stream) {
                    stream_reset(
                        stream,
                        string_content.offset(-(1 as os::raw::c_int as isize)),
                    );
                    return TSQueryErrorSyntax;
                }
            }
            let mut length_0: u32 =
                (*stream).input.wrapping_offset_from_(string_content) as os::raw::c_long as u32;
            // Add a step for the node
            let mut symbol_0: TSSymbol =
                ts_language_symbol_for_name((*self_0).language, string_content, length_0, false);
            if symbol_0 == 0 {
                stream_reset(stream, string_content);
                return TSQueryErrorNodeType;
            }
            array__grow(
                &mut (*self_0).steps as *mut TSQuerySteps as *mut VoidArray,
                1 as os::raw::c_int as usize,
                ::std::mem::size_of::<QueryStep>(),
            );
            let fresh13 = (*self_0).steps.size;
            (*self_0).steps.size = (*self_0).steps.size.wrapping_add(1);
            *(*self_0).steps.contents.offset(fresh13 as isize) =
                query_step__new(symbol_0, depth as u16, is_immediate);
            if (*stream).next != '\"' as i32 {
                return TSQueryErrorSyntax;
            }
            stream_advance(stream);
        } else if stream_is_ident_start(stream) {
            // Parse a field-prefixed pattern
            // Parse the field name
            let mut field_name: *const os::raw::c_char = (*stream).input;
            stream_scan_identifier(stream);
            let mut length_1: u32 =
                (*stream).input.wrapping_offset_from_(field_name) as os::raw::c_long as u32;
            stream_skip_whitespace(stream);
            if (*stream).next != ':' as i32 {
                stream_reset(stream, field_name);
                return TSQueryErrorSyntax;
            }
            stream_advance(stream);
            stream_skip_whitespace(stream);
            // Parse the pattern
            let mut step_index: u32 = (*self_0).steps.size;
            let mut e_2: TSQueryError =
                ts_query__parse_pattern(self_0, stream, depth, capture_count, is_immediate);
            if e_2 as os::raw::c_uint == PARENT_DONE as os::raw::c_uint {
                return TSQueryErrorSyntax;
            }
            if e_2 as u64 != 0 {
                return e_2;
            }
            // Add the field name to the first step of the pattern
            let mut field_id: TSFieldId =
                ts_language_field_id_for_name((*self_0).language, field_name, length_1);
            if field_id == 0 {
                (*stream).input = field_name;
                return TSQueryErrorField;
            }
            (*(*self_0).steps.contents.offset(step_index as isize)).field = field_id
        } else if (*stream).next == '*' as i32 {
            stream_advance(stream);
            stream_skip_whitespace(stream);
            // Parse a wildcard pattern
            // Add a step that matches any kind of node
            array__grow(
                &mut (*self_0).steps as *mut TSQuerySteps as *mut VoidArray,
                1 as os::raw::c_int as usize,
                ::std::mem::size_of::<QueryStep>(),
            );
            let fresh14 = (*self_0).steps.size;
            (*self_0).steps.size = (*self_0).steps.size.wrapping_add(1);
            *(*self_0).steps.contents.offset(fresh14 as isize) =
                query_step__new(WILDCARD_SYMBOL, depth as u16, is_immediate)
        } else {
            return TSQueryErrorSyntax;
        }
    }
    stream_skip_whitespace(stream);
    // Parse an '@'-prefixed capture pattern
    #[allow(clippy::while_immutable_condition)]
    while (*stream).next == '@' as i32 {
        stream_advance(stream);
        // Parse the capture name
        if !stream_is_ident_start(stream) {
            return TSQueryErrorSyntax;
        }
        let mut capture_name: *const os::raw::c_char = (*stream).input;
        stream_scan_identifier(stream);
        let mut length_2: u32 =
            (*stream).input.wrapping_offset_from_(capture_name) as os::raw::c_long as u32;
        // Add the capture id to the first step of the pattern
        let mut capture_id: u16 =
            symbol_table_insert_name(&mut (*self_0).captures, capture_name, length_2);
        let mut step: *mut QueryStep = &mut *(*self_0)
            .steps
            .contents
            .offset(starting_step_index as isize)
            as *mut QueryStep;
        query_step__add_capture(step, capture_id);
        *capture_count = (*capture_count).wrapping_add(1);
        stream_skip_whitespace(stream);
    }
    TSQueryErrorNone
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_new(
    mut language: *const TSLanguage,
    mut source: *const os::raw::c_char,
    mut source_len: u32,
    mut error_offset: *mut u32,
    mut error_type: *mut TSQueryError,
) -> *mut TSQuery {
    let mut symbol_map: *mut TSSymbol = std::ptr::null_mut::<TSSymbol>();
    if ts_language_version(language) >= 11 as os::raw::c_int as os::raw::c_uint {
        symbol_map = std::ptr::null_mut::<TSSymbol>()
    } else {
        // Work around the fact that multiple symbols can currently be
        // associated with the same name, due to "simple aliases".
        // In the next language ABI version, this map will be contained
        // in the language's `public_symbol_map` field.
        let mut symbol_count: u32 = ts_language_symbol_count(language);
        symbol_map =
            ts_malloc((::std::mem::size_of::<TSSymbol>()).wrapping_mul(symbol_count as usize))
                as *mut TSSymbol;
        let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
        while i < symbol_count {
            let mut name: *const os::raw::c_char = ts_language_symbol_name(language, i as TSSymbol);
            let symbol_type: TSSymbolType = ts_language_symbol_type(language, i as TSSymbol);
            *symbol_map.offset(i as isize) = i as TSSymbol;
            let mut j: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
            while j < i {
                if ts_language_symbol_type(language, j as TSSymbol) as os::raw::c_uint
                    == symbol_type as os::raw::c_uint
                    && strcmp(name, ts_language_symbol_name(language, j as TSSymbol)) == 0
                {
                    *symbol_map.offset(i as isize) = j as TSSymbol;
                    break;
                }
                j = j.wrapping_add(1)
            }
            i = i.wrapping_add(1)
        }
    }
    let mut self_0: *mut TSQuery = ts_malloc(::std::mem::size_of::<TSQuery>()) as *mut TSQuery;
    *self_0 = TSQuery {
        captures: symbol_table_new(),
        predicate_values: symbol_table_new(),
        steps: TSQuerySteps {
            contents: std::ptr::null_mut::<QueryStep>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        pattern_map: TSQueryPatternMap {
            contents: std::ptr::null_mut::<PatternEntry>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        predicate_steps: TSQueryPredicateSteps {
            contents: std::ptr::null_mut::<TSQueryPredicateStep>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        predicates_by_pattern: TSQueryPredicatesByPattern {
            contents: std::ptr::null_mut::<Slice>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        start_bytes_by_pattern: TSQueryStartBytesByPattern {
            contents: std::ptr::null_mut::<u32>(),
            size: 0,
            capacity: 0,
        },
        language,
        max_capture_count: 0 as os::raw::c_int as u16,
        wildcard_root_pattern_count: 0 as os::raw::c_int as u16,
        symbol_map,
    };
    // Parse all of the S-expressions in the given string.
    let mut stream: Stream = stream_new(source, source_len);
    stream_skip_whitespace(&mut stream);
    let mut start_step_index: u32 = 0;
    while stream.input < stream.end {
        start_step_index = (*self_0).steps.size;
        let mut capture_count: u32 = 0 as os::raw::c_int as u32;
        array__grow(
            &mut (*self_0).start_bytes_by_pattern as *mut TSQueryStartBytesByPattern
                as *mut VoidArray,
            1 as os::raw::c_int as usize,
            ::std::mem::size_of::<u32>(),
        );
        let fresh15 = (*self_0).start_bytes_by_pattern.size;
        (*self_0).start_bytes_by_pattern.size =
            (*self_0).start_bytes_by_pattern.size.wrapping_add(1);
        *(*self_0)
            .start_bytes_by_pattern
            .contents
            .offset(fresh15 as isize) =
            stream.input.wrapping_offset_from_(source) as os::raw::c_long as u32;
        array__grow(
            &mut (*self_0).predicates_by_pattern as *mut TSQueryPredicatesByPattern
                as *mut VoidArray,
            1 as os::raw::c_int as usize,
            ::std::mem::size_of::<Slice>(),
        );
        let fresh16 = (*self_0).predicates_by_pattern.size;
        (*self_0).predicates_by_pattern.size = (*self_0).predicates_by_pattern.size.wrapping_add(1);
        *(*self_0)
            .predicates_by_pattern
            .contents
            .offset(fresh16 as isize) = Slice {
            offset: (*self_0).predicate_steps.size,
            length: 0 as os::raw::c_int as u32,
        };
        *error_type = ts_query__parse_pattern(
            self_0,
            &mut stream,
            0 as os::raw::c_int as u32,
            &mut capture_count,
            false,
        );
        array__grow(
            &mut (*self_0).steps as *mut TSQuerySteps as *mut VoidArray,
            1 as os::raw::c_int as usize,
            ::std::mem::size_of::<QueryStep>(),
        );
        let fresh17 = (*self_0).steps.size;
        (*self_0).steps.size = (*self_0).steps.size.wrapping_add(1);
        *(*self_0).steps.contents.offset(fresh17 as isize) = query_step__new(
            0 as os::raw::c_int as TSSymbol,
            PATTERN_DONE_MARKER as u16,
            false,
        );
        // If any pattern could not be parsed, then report the error information
        // and terminate.
        if *error_type as u64 != 0 {
            *error_offset = stream.input.wrapping_offset_from_(source) as os::raw::c_long as u32;
            ts_query_delete(self_0);
            return std::ptr::null_mut::<TSQuery>();
        }
        // Maintain a map that can look up patterns for a given root symbol.
        ts_query__pattern_map_insert(
            self_0,
            (*(*self_0).steps.contents.offset(start_step_index as isize)).symbol,
            start_step_index,
        );
        if (*(*self_0).steps.contents.offset(start_step_index as isize)).symbol as os::raw::c_int
            == WILDCARD_SYMBOL as os::raw::c_int
        {
            (*self_0).wildcard_root_pattern_count =
                (*self_0).wildcard_root_pattern_count.wrapping_add(1)
        }
        // Keep track of the maximum number of captures in pattern, because
        // that numer determines how much space is needed to store each capture
        // list.
        if capture_count > (*self_0).max_capture_count as os::raw::c_uint {
            (*self_0).max_capture_count = capture_count as u16
        }
    }
    ts_query__finalize_steps(self_0);
    self_0
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_delete(mut self_0: *mut TSQuery) {
    if !self_0.is_null() {
        array__delete(&mut (*self_0).steps as *mut TSQuerySteps as *mut VoidArray);
        array__delete(&mut (*self_0).pattern_map as *mut TSQueryPatternMap as *mut VoidArray);
        array__delete(
            &mut (*self_0).predicate_steps as *mut TSQueryPredicateSteps as *mut VoidArray,
        );
        array__delete(
            &mut (*self_0).predicates_by_pattern as *mut TSQueryPredicatesByPattern
                as *mut VoidArray,
        );
        array__delete(
            &mut (*self_0).start_bytes_by_pattern as *mut TSQueryStartBytesByPattern
                as *mut VoidArray,
        );
        symbol_table_delete(&mut (*self_0).captures);
        symbol_table_delete(&mut (*self_0).predicate_values);
        ts_free((*self_0).symbol_map as *mut ffi::c_void);
        ts_free(self_0 as *mut ffi::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_pattern_count(mut self_0: *const TSQuery) -> u32 {
    (*self_0).predicates_by_pattern.size
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_capture_count(mut self_0: *const TSQuery) -> u32 {
    (*self_0).captures.slices.size
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_string_count(mut self_0: *const TSQuery) -> u32 {
    (*self_0).predicate_values.slices.size
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_capture_name_for_id(
    mut self_0: *const TSQuery,
    mut index: u32,
    mut length: *mut u32,
) -> *const os::raw::c_char {
    symbol_table_name_for_id(&(*self_0).captures, index as u16, length)
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_string_value_for_id(
    mut self_0: *const TSQuery,
    mut index: u32,
    mut length: *mut u32,
) -> *const os::raw::c_char {
    symbol_table_name_for_id(&(*self_0).predicate_values, index as u16, length)
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_predicates_for_pattern(
    mut self_0: *const TSQuery,
    mut pattern_index: u32,
    mut step_count: *mut u32,
) -> *const TSQueryPredicateStep {
    let mut slice: Slice = *(*self_0)
        .predicates_by_pattern
        .contents
        .offset(pattern_index as isize);
    *step_count = slice.length;
    &mut *(*self_0)
        .predicate_steps
        .contents
        .offset(slice.offset as isize) as *mut TSQueryPredicateStep
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_start_byte_for_pattern(
    mut self_0: *const TSQuery,
    mut pattern_index: u32,
) -> u32 {
    *(*self_0)
        .start_bytes_by_pattern
        .contents
        .offset(pattern_index as isize)
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_disable_capture(
    mut self_0: *mut TSQuery,
    mut name: *const os::raw::c_char,
    mut length: u32,
) {
    // Remove capture information for any pattern step that previously
    // captured with the given name.
    let mut id: os::raw::c_int = symbol_table_id_for_name(&(*self_0).captures, name, length);
    if id != -(1 as os::raw::c_int) {
        let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
        while i < (*self_0).steps.size {
            let mut step: *mut QueryStep =
                &mut *(*self_0).steps.contents.offset(i as isize) as *mut QueryStep;
            query_step__remove_capture(step, id as u16);
            i = i.wrapping_add(1)
        }
        ts_query__finalize_steps(self_0);
    };
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_disable_pattern(
    mut self_0: *mut TSQuery,
    mut pattern_index: u32,
) {
    // Remove the given pattern from the pattern map. Its steps will still
    // be in the `steps` array, but they will never be read.
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < (*self_0).pattern_map.size {
        let mut pattern: *mut PatternEntry =
            &mut *(*self_0).pattern_map.contents.offset(i as isize) as *mut PatternEntry;
        if (*pattern).pattern_index as os::raw::c_uint == pattern_index {
            array__erase(
                &mut (*self_0).pattern_map as *mut TSQueryPatternMap as *mut VoidArray,
                ::std::mem::size_of::<PatternEntry>(),
                i,
            );
            i = i.wrapping_sub(1)
        }
        i = i.wrapping_add(1)
    }
}

/* **************
 * QueryCursor
 ***************/
#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_new() -> *mut TSQueryCursor {
    let mut self_0: *mut TSQueryCursor =
        ts_malloc(::std::mem::size_of::<TSQueryCursor>()) as *mut TSQueryCursor;
    *self_0 = TSQueryCursor {
        query: std::ptr::null::<TSQuery>(),
        cursor: TSTreeCursor {
            tree: ptr::null(),
            id: ptr::null(),
            context: [0; 2],
        },
        states: TSQueryCursorStates {
            contents: std::ptr::null_mut::<QueryState>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        finished_states: TsQueryCursorFinishedStated {
            contents: std::ptr::null_mut::<QueryState>(),
            size: 0 as os::raw::c_int as u32,
            capacity: 0 as os::raw::c_int as u32,
        },
        capture_list_pool: capture_list_pool_new(),
        depth: 0,
        start_byte: 0 as os::raw::c_int as u32,
        end_byte: 4_294_967_295 as os::raw::c_uint,
        next_state_id: 0,
        start_point: TSPoint {
            row: 0 as os::raw::c_int as u32,
            column: 0 as os::raw::c_int as u32,
        },
        end_point: TSPoint {
            row: 4_294_967_295 as os::raw::c_uint,
            column: 4_294_967_295 as os::raw::c_uint,
        },
        ascending: false,
    };
    array__reserve(
        &mut (*self_0).states as *mut TSQueryCursorStates as *mut VoidArray,
        ::std::mem::size_of::<QueryState>(),
        MAX_STATE_COUNT as u32,
    );
    array__reserve(
        &mut (*self_0).finished_states as *mut TsQueryCursorFinishedStated as *mut VoidArray,
        ::std::mem::size_of::<QueryState>(),
        MAX_STATE_COUNT as u32,
    );
    self_0
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_delete(mut self_0: *mut TSQueryCursor) {
    array__delete(&mut (*self_0).states as *mut TSQueryCursorStates as *mut VoidArray);
    array__delete(
        &mut (*self_0).finished_states as *mut TsQueryCursorFinishedStated as *mut VoidArray,
    );
    ts_tree_cursor_delete(&mut (*self_0).cursor);
    capture_list_pool_delete(&mut (*self_0).capture_list_pool);
    ts_free(self_0 as *mut ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_exec(
    mut self_0: *mut TSQueryCursor,
    mut query: *const TSQuery,
    mut node: TSNode,
) {
    (*self_0).states.size = 0 as os::raw::c_int as u32;
    (*self_0).finished_states.size = 0 as os::raw::c_int as u32;
    ts_tree_cursor_reset(&mut (*self_0).cursor, node);
    capture_list_pool_reset(&mut (*self_0).capture_list_pool, (*query).max_capture_count);
    (*self_0).next_state_id = 0 as os::raw::c_int as u32;
    (*self_0).depth = 0 as os::raw::c_int as u32;
    (*self_0).ascending = false;
    (*self_0).query = query;
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_set_byte_range(
    mut self_0: *mut TSQueryCursor,
    mut start_byte: u32,
    mut end_byte: u32,
) {
    if end_byte == 0 as os::raw::c_int as os::raw::c_uint {
        start_byte = 0 as os::raw::c_int as u32;
        end_byte = 4_294_967_295 as os::raw::c_uint
    }
    (*self_0).start_byte = start_byte;
    (*self_0).end_byte = end_byte;
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_set_point_range(
    mut self_0: *mut TSQueryCursor,
    mut start_point: TSPoint,
    mut end_point: TSPoint,
) {
    if end_point.row == 0 as os::raw::c_int as os::raw::c_uint
        && end_point.column == 0 as os::raw::c_int as os::raw::c_uint
    {
        start_point = TSPoint {
            row: 0 as os::raw::c_int as u32,
            column: 0 as os::raw::c_int as u32,
        };
        end_point = TSPoint {
            row: 4_294_967_295 as os::raw::c_uint,
            column: 4_294_967_295 as os::raw::c_uint,
        }
    }
    (*self_0).start_point = start_point;
    (*self_0).end_point = end_point;
}
// Search through all of the in-progress states, and find the captured
// node that occurs earliest in the document.
unsafe extern "C" fn ts_query_cursor__first_in_progress_capture(
    mut self_0: *mut TSQueryCursor,
    mut state_index: *mut u32,
    mut byte_offset: *mut u32,
    mut pattern_index: *mut u32,
) -> bool {
    let mut result: bool = false;
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < (*self_0).states.size {
        let mut state: *const QueryState =
            &mut *(*self_0).states.contents.offset(i as isize) as *mut QueryState;
        if (*state).capture_count as os::raw::c_int > 0 as os::raw::c_int {
            let mut captures: *const TSQueryCapture =
                capture_list_pool_get(&mut (*self_0).capture_list_pool, (*state).capture_list_id);
            let mut capture_byte: u32 =
                ts_node_start_byte((*captures.offset(0 as os::raw::c_int as isize)).node);
            if !result
                || capture_byte < *byte_offset
                || capture_byte == *byte_offset
                    && ((*state).pattern_index as os::raw::c_uint) < *pattern_index
            {
                result = true;
                *state_index = i;
                *byte_offset = capture_byte;
                *pattern_index = (*state).pattern_index as u32
            }
        }
        i = i.wrapping_add(1)
    }
    result
}
unsafe extern "C" fn ts_query__cursor_add_state(
    mut self_0: *mut TSQueryCursor,
    mut pattern: *const PatternEntry,
) -> bool {
    let mut list_id: u32 = capture_list_pool_acquire(&mut (*self_0).capture_list_pool) as u32;
    // If there are no capture lists left in the pool, then terminate whichever
    // state has captured the earliest node in the document, and steal its
    // capture list.
    if list_id == NONE as os::raw::c_uint {
        let mut state_index: u32 = 0;
        let mut byte_offset: u32 = 0;
        let mut pattern_index: u32 = 0;
        if ts_query_cursor__first_in_progress_capture(
            self_0,
            &mut state_index,
            &mut byte_offset,
            &mut pattern_index,
        ) {
            list_id =
                (*(*self_0).states.contents.offset(state_index as isize)).capture_list_id as u32;
            array__erase(
                &mut (*self_0).states as *mut TSQueryCursorStates as *mut VoidArray,
                ::std::mem::size_of::<QueryState>(),
                state_index,
            );
        } else {
            return false;
        }
    }
    array__grow(
        &mut (*self_0).states as *mut TSQueryCursorStates as *mut VoidArray,
        1 as os::raw::c_int as usize,
        ::std::mem::size_of::<QueryState>(),
    );
    let fresh18 = (*self_0).states.size;
    (*self_0).states.size = (*self_0).states.size.wrapping_add(1);
    *(*self_0).states.contents.offset(fresh18 as isize) = QueryState {
        start_depth: (*self_0).depth as u16,
        pattern_index: (*pattern).pattern_index,
        step_index: (*pattern).step_index,
        capture_count: 0 as os::raw::c_int as u16,
        capture_list_id: list_id as u16,
        consumed_capture_count: 0 as os::raw::c_int as u16,
        id: 0,
    };
    true
}
unsafe extern "C" fn ts_query__cursor_copy_state(
    mut self_0: *mut TSQueryCursor,
    mut state: *const QueryState,
) -> *mut QueryState {
    let mut new_list_id: u32 = capture_list_pool_acquire(&mut (*self_0).capture_list_pool) as u32;
    if new_list_id == NONE as os::raw::c_uint {
        return std::ptr::null_mut::<QueryState>();
    }
    array__grow(
        &mut (*self_0).states as *mut TSQueryCursorStates as *mut VoidArray,
        1 as os::raw::c_int as usize,
        ::std::mem::size_of::<QueryState>(),
    );
    let fresh19 = (*self_0).states.size;
    (*self_0).states.size = (*self_0).states.size.wrapping_add(1);
    *(*self_0).states.contents.offset(fresh19 as isize) = *state;
    assert!(
        (*self_0)
            .states
            .size
            .wrapping_sub(1 as os::raw::c_int as os::raw::c_uint)
            < (*self_0).states.size
    );
    let mut new_state: *mut QueryState = &mut *(*self_0)
        .states
        .contents
        .offset((*self_0).states.size.wrapping_sub(1) as isize)
        as *mut QueryState;
    (*new_state).capture_list_id = new_list_id as u16;
    let mut old_captures: *mut TSQueryCapture =
        capture_list_pool_get(&mut (*self_0).capture_list_pool, (*state).capture_list_id);
    let mut new_captures: *mut TSQueryCapture =
        capture_list_pool_get(&mut (*self_0).capture_list_pool, new_list_id as u16);
    copy_nonoverlapping(old_captures, new_captures, (*state).capture_count as usize);
    new_state
}
// Walk the tree, processing patterns until at least one pattern finishes,
// If one or more patterns finish, return `true` and store their states in the
// `finished_states` array. Multiple patterns can finish on the same node. If
// there are no more matches, return `false`.
#[inline]
unsafe extern "C" fn ts_query_cursor__advance(mut self_0: *mut TSQueryCursor) -> bool {
    loop {
        if (*self_0).ascending {
            // When leaving a node, remove any unfinished states whose next step
            // needed to match something within that node.
            let mut deleted_count: u32 = 0 as os::raw::c_int as u32;
            let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
            let mut n: os::raw::c_uint = (*self_0).states.size;
            while i < n {
                let mut state: *mut QueryState =
                    &mut *(*self_0).states.contents.offset(i as isize) as *mut QueryState;
                let mut step: *mut QueryStep = &mut *(*(*self_0).query)
                    .steps
                    .contents
                    .offset((*state).step_index as isize)
                    as *mut QueryStep;
                if ((*state).start_depth as u32).wrapping_add((*step).depth() as u32)
                    > (*self_0).depth
                {
                    capture_list_pool_release(
                        &mut (*self_0).capture_list_pool,
                        (*state).capture_list_id,
                    );
                    deleted_count = deleted_count.wrapping_add(1)
                } else if deleted_count > 0 as os::raw::c_int as os::raw::c_uint {
                    *(*self_0)
                        .states
                        .contents
                        .offset(i.wrapping_sub(deleted_count) as isize) = *state
                }
                i = i.wrapping_add(1)
            }
            (*self_0).states.size = ((*self_0).states.size as os::raw::c_uint)
                .wrapping_sub(deleted_count) as u32 as u32;
            if ts_tree_cursor_goto_next_sibling(&mut (*self_0).cursor) {
                (*self_0).ascending = false
            } else if ts_tree_cursor_goto_parent(&mut (*self_0).cursor) {
                (*self_0).depth = (*self_0).depth.wrapping_sub(1)
            } else {
                return (*self_0).finished_states.size > 0 as os::raw::c_int as os::raw::c_uint;
            }
        } else {
            let mut has_later_siblings: bool = false;
            let mut can_have_later_siblings_with_this_field: bool = false;
            let mut field_id: TSFieldId = ts_tree_cursor_current_status(
                &(*self_0).cursor,
                &mut has_later_siblings,
                &mut can_have_later_siblings_with_this_field,
            );
            let mut node: TSNode = ts_tree_cursor_current_node(&(*self_0).cursor);
            let mut symbol: TSSymbol = ts_node_symbol(node);
            let mut is_named: bool = ts_node_is_named(node);
            if symbol as os::raw::c_int != -(1 as os::raw::c_int) as TSSymbol as os::raw::c_int
                && !(*(*self_0).query).symbol_map.is_null()
            {
                symbol = *(*(*self_0).query).symbol_map.offset(symbol as isize)
            }
            // If this node is before the selected range, then avoid descending
            // into it.
            if ts_node_end_byte(node) <= (*self_0).start_byte
                || point_lte(ts_node_end_point(node), (*self_0).start_point) as os::raw::c_int != 0
            {
                if !ts_tree_cursor_goto_next_sibling(&mut (*self_0).cursor) {
                    (*self_0).ascending = true
                }
            } else {
                // If this node is after the selected range, then stop walking.
                if (*self_0).end_byte <= ts_node_start_byte(node)
                    || point_lte((*self_0).end_point, ts_node_start_point(node)) as os::raw::c_int
                        != 0
                {
                    return false;
                }
                // Add new states for any patterns whose root node is a wildcard.
                let mut i_0: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
                while i_0 < (*(*self_0).query).wildcard_root_pattern_count as os::raw::c_uint {
                    let mut pattern: *mut PatternEntry =
                        &mut *(*(*self_0).query).pattern_map.contents.offset(i_0 as isize)
                            as *mut PatternEntry;
                    let mut step_0: *mut QueryStep = &mut *(*(*self_0).query)
                        .steps
                        .contents
                        .offset((*pattern).step_index as isize)
                        as *mut QueryStep;
                    // If this node matches the first step of the pattern, then add a new
                    // state at the start of this pattern.
                    if ((*step_0).field as os::raw::c_int == 0
                        || field_id as os::raw::c_int == (*step_0).field as os::raw::c_int)
                        && !ts_query__cursor_add_state(self_0, pattern)
                    {
                        break;
                    }
                    i_0 = i_0.wrapping_add(1)
                }
                // Add new states for any patterns whose root node matches this node.
                let mut i_1: os::raw::c_uint = 0;
                if ts_query__pattern_map_search((*self_0).query, symbol, &mut i_1) {
                    let mut pattern_0: *mut PatternEntry =
                        &mut *(*(*self_0).query).pattern_map.contents.offset(i_1 as isize)
                            as *mut PatternEntry;
                    let mut step_1: *mut QueryStep = &mut *(*(*self_0).query)
                        .steps
                        .contents
                        .offset((*pattern_0).step_index as isize)
                        as *mut QueryStep;
                    loop
                    // If this node matches the first step of the pattern, then add a new
                    // state at the start of this pattern.
                    {
                        if !((*step_1).field as os::raw::c_int != 0
                            && field_id as os::raw::c_int != (*step_1).field as os::raw::c_int)
                        {
                            if !ts_query__cursor_add_state(self_0, pattern_0) {
                                break;
                            }
                            // Advance to the next pattern whose root node matches this node.
                            i_1 = i_1.wrapping_add(1);
                            if i_1 == (*(*self_0).query).pattern_map.size {
                                break;
                            }
                            pattern_0 =
                                &mut *(*(*self_0).query).pattern_map.contents.offset(i_1 as isize)
                                    as *mut PatternEntry;
                            step_1 = &mut *(*(*self_0).query)
                                .steps
                                .contents
                                .offset((*pattern_0).step_index as isize)
                                as *mut QueryStep
                        }
                        if (*step_1).symbol as os::raw::c_int != symbol as os::raw::c_int {
                            break;
                        }
                    }
                }
                // Update all of the in-progress states with current node.
                let mut i_2: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
                let mut n_0: os::raw::c_uint = (*self_0).states.size;
                while i_2 < n_0 {
                    let mut state_0: *mut QueryState =
                        &mut *(*self_0).states.contents.offset(i_2 as isize) as *mut QueryState;
                    let mut step_2: *mut QueryStep = &mut *(*(*self_0).query)
                        .steps
                        .contents
                        .offset((*state_0).step_index as isize)
                        as *mut QueryStep;
                    // Check that the node matches all of the criteria for the next
                    // step of the pattern.
                    if ((*state_0).start_depth as u32).wrapping_add((*step_2).depth() as u32)
                        == (*self_0).depth
                    {
                        // Determine if this node matches this step of the pattern, and also
                        // if this node can have later siblings that match this step of the
                        // pattern.
                        let mut node_does_match: bool = (*step_2).symbol as os::raw::c_int
                            == symbol as os::raw::c_int
                            || (*step_2).symbol as os::raw::c_int
                                == WILDCARD_SYMBOL as os::raw::c_int
                            || (*step_2).symbol as os::raw::c_int
                                == NAMED_WILDCARD_SYMBOL as os::raw::c_int
                                && is_named as os::raw::c_int != 0;
                        let mut later_sibling_can_match: bool = has_later_siblings;
                        if (*step_2).is_immediate() as os::raw::c_int != 0
                            && is_named as os::raw::c_int != 0
                        {
                            later_sibling_can_match = false
                        }
                        if (*step_2).is_last() as os::raw::c_int != 0
                            && has_later_siblings as os::raw::c_int != 0
                        {
                            node_does_match = false
                        }
                        if (*step_2).field != 0 {
                            if (*step_2).field as os::raw::c_int == field_id as os::raw::c_int {
                                if !can_have_later_siblings_with_this_field {
                                    later_sibling_can_match = false
                                }
                            } else {
                                node_does_match = false
                            }
                        }
                        if !node_does_match {
                            if !later_sibling_can_match {
                                capture_list_pool_release(
                                    &mut (*self_0).capture_list_pool,
                                    (*state_0).capture_list_id,
                                );
                                array__erase(
                                    &mut (*self_0).states as *mut TSQueryCursorStates
                                        as *mut VoidArray,
                                    ::std::mem::size_of::<QueryState>(),
                                    i_2,
                                );
                                i_2 = i_2.wrapping_sub(1);
                                n_0 = n_0.wrapping_sub(1)
                            }
                        } else {
                            // Some patterns can match their root node in multiple ways,
                            // capturing different children. If this pattern step could match
                            // later children within the same parent, then this query state
                            // cannot simply be updated in place. It must be split into two
                            // states: one that matches this node, and one which skips over
                            // this node, to preserve the possibility of matching later
                            // siblings.
                            let mut next_state: *mut QueryState = state_0;
                            if (*step_2).depth() as os::raw::c_int > 0 as os::raw::c_int
                                && (*step_2).contains_captures() as os::raw::c_int != 0
                                && later_sibling_can_match as os::raw::c_int != 0
                            {
                                let mut copy: *mut QueryState =
                                    ts_query__cursor_copy_state(self_0, state_0);
                                if !copy.is_null() {
                                    next_state = copy
                                }
                            }
                            // If the current node is captured in this pattern, add it to the
                            // capture list.
                            let mut j: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
                            while j < 4 as os::raw::c_int as os::raw::c_uint {
                                let mut capture_id: u16 = (*step_2).capture_ids[j as usize];
                                if (*step_2).capture_ids[j as usize] as os::raw::c_int
                                    == NONE as os::raw::c_int
                                {
                                    break;
                                }
                                let mut capture_list: *mut TSQueryCapture = capture_list_pool_get(
                                    &mut (*self_0).capture_list_pool,
                                    (*next_state).capture_list_id,
                                );
                                let fresh20 = (*next_state).capture_count;
                                (*next_state).capture_count =
                                    (*next_state).capture_count.wrapping_add(1);
                                *capture_list.offset(fresh20 as isize) = TSQueryCapture {
                                    node,
                                    index: capture_id as u32,
                                };
                                j = j.wrapping_add(1)
                            }
                            // If the pattern is now done, then remove it from the list of
                            // in-progress states, and add it to the list of finished states.
                            (*next_state).step_index = (*next_state).step_index.wrapping_add(1);
                            let mut next_step: *mut QueryStep =
                                step_2.offset(1 as os::raw::c_int as isize);
                            if (*next_step).depth() as os::raw::c_int
                                == PATTERN_DONE_MARKER as os::raw::c_int
                            {
                                let fresh21 = (*self_0).next_state_id;
                                (*self_0).next_state_id = (*self_0).next_state_id.wrapping_add(1);
                                (*next_state).id = fresh21;
                                array__grow(
                                    &mut (*self_0).finished_states
                                        as *mut TsQueryCursorFinishedStated
                                        as *mut VoidArray,
                                    1 as os::raw::c_int as usize,
                                    ::std::mem::size_of::<QueryState>(),
                                );
                                let fresh22 = (*self_0).finished_states.size;
                                (*self_0).finished_states.size =
                                    (*self_0).finished_states.size.wrapping_add(1);
                                *(*self_0).finished_states.contents.offset(fresh22 as isize) =
                                    *next_state;
                                if next_state == state_0 {
                                    array__erase(
                                        &mut (*self_0).states as *mut TSQueryCursorStates
                                            as *mut VoidArray,
                                        ::std::mem::size_of::<QueryState>(),
                                        i_2,
                                    );
                                    i_2 = i_2.wrapping_sub(1);
                                    n_0 = n_0.wrapping_sub(1)
                                } else {
                                    (*self_0).states.size = (*self_0).states.size.wrapping_sub(1)
                                }
                            }
                        }
                    }
                    i_2 = i_2.wrapping_add(1)
                }
                // Continue descending if possible.
                if ts_tree_cursor_goto_first_child(&mut (*self_0).cursor) {
                    (*self_0).depth = (*self_0).depth.wrapping_add(1)
                } else {
                    (*self_0).ascending = true
                }
            }
        }
        if (*self_0).finished_states.size != 0 as os::raw::c_int as os::raw::c_uint {
            break;
        }
    }
    true
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_next_match(
    mut self_0: *mut TSQueryCursor,
    mut match_0: *mut TSQueryMatch,
) -> bool {
    if (*self_0).finished_states.size == 0 as os::raw::c_int as os::raw::c_uint
        && !ts_query_cursor__advance(self_0)
    {
        return false;
    }
    let mut state: *mut QueryState = &mut *(*self_0)
        .finished_states
        .contents
        .offset(0 as os::raw::c_int as isize)
        as *mut QueryState;
    (*match_0).id = (*state).id;
    (*match_0).pattern_index = (*state).pattern_index;
    (*match_0).capture_count = (*state).capture_count;
    (*match_0).captures =
        capture_list_pool_get(&mut (*self_0).capture_list_pool, (*state).capture_list_id);
    capture_list_pool_release(&mut (*self_0).capture_list_pool, (*state).capture_list_id);
    array__erase(
        &mut (*self_0).finished_states as *mut TsQueryCursorFinishedStated as *mut VoidArray,
        ::std::mem::size_of::<QueryState>(),
        0 as os::raw::c_int as u32,
    );
    true
}
#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_remove_match(
    mut self_0: *mut TSQueryCursor,
    mut match_id: u32,
) {
    let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
    while i < (*self_0).finished_states.size {
        let mut state: *const QueryState =
            &mut *(*self_0).finished_states.contents.offset(i as isize) as *mut QueryState;
        if (*state).id == match_id {
            capture_list_pool_release(&mut (*self_0).capture_list_pool, (*state).capture_list_id);
            array__erase(
                &mut (*self_0).finished_states as *mut TsQueryCursorFinishedStated
                    as *mut VoidArray,
                ::std::mem::size_of::<QueryState>(),
                i,
            );
            return;
        }
        i = i.wrapping_add(1)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ts_query_cursor_next_capture(
    mut self_0: *mut TSQueryCursor,
    mut match_0: *mut TSQueryMatch,
    mut capture_index: *mut u32,
) -> bool {
    loop {
        // The goal here is to return captures in order, even though they may not
        // be discovered in order, because patterns can overlap. If there are any
        // finished patterns, then try to find one that contains a capture that
        // is *definitely* before any capture in an *unfinished* pattern.
        if (*self_0).finished_states.size > 0 as os::raw::c_int as os::raw::c_uint {
            // First, identify the position of the earliest capture in an unfinished
            // match. For a finished capture to be returned, it must be *before*
            // this position.
            let mut first_unfinished_capture_byte: u32 = 4_294_967_295 as os::raw::c_uint;
            let mut first_unfinished_pattern_index: u32 = 4_294_967_295 as os::raw::c_uint;
            let mut first_unfinished_state_index: u32 = 0;
            ts_query_cursor__first_in_progress_capture(
                self_0,
                &mut first_unfinished_state_index,
                &mut first_unfinished_capture_byte,
                &mut first_unfinished_pattern_index,
            );
            // Find the earliest capture in a finished match.
            let mut first_finished_state_index: os::raw::c_int = -(1 as os::raw::c_int);
            let mut first_finished_capture_byte: u32 = first_unfinished_capture_byte;
            let mut first_finished_pattern_index: u32 = first_unfinished_pattern_index;
            let mut i: os::raw::c_uint = 0 as os::raw::c_int as os::raw::c_uint;
            while i < (*self_0).finished_states.size {
                let mut state: *const QueryState =
                    &mut *(*self_0).finished_states.contents.offset(i as isize) as *mut QueryState;
                if (*state).capture_count as os::raw::c_int
                    > (*state).consumed_capture_count as os::raw::c_int
                {
                    let mut captures: *const TSQueryCapture = capture_list_pool_get(
                        &mut (*self_0).capture_list_pool,
                        (*state).capture_list_id,
                    );
                    let mut capture_byte: u32 = ts_node_start_byte(
                        (*captures.offset((*state).consumed_capture_count as isize)).node,
                    );
                    if capture_byte < first_finished_capture_byte
                        || capture_byte == first_finished_capture_byte
                            && ((*state).pattern_index as os::raw::c_uint)
                                < first_finished_pattern_index
                    {
                        first_finished_state_index = i as os::raw::c_int;
                        first_finished_capture_byte = capture_byte;
                        first_finished_pattern_index = (*state).pattern_index as u32
                    }
                } else {
                    capture_list_pool_release(
                        &mut (*self_0).capture_list_pool,
                        (*state).capture_list_id,
                    );
                    array__erase(
                        &mut (*self_0).finished_states as *mut TsQueryCursorFinishedStated
                            as *mut VoidArray,
                        ::std::mem::size_of::<QueryState>(),
                        i,
                    );
                    i = i.wrapping_sub(1)
                }
                i = i.wrapping_add(1)
            }
            // If there is finished capture that is clearly before any unfinished
            // capture, then return its match, and its capture index. Internally
            // record the fact that the capture has been 'consumed'.
            if first_finished_state_index != -(1 as os::raw::c_int) {
                let mut state_0: *mut QueryState = &mut *(*self_0)
                    .finished_states
                    .contents
                    .offset(first_finished_state_index as isize)
                    as *mut QueryState;
                (*match_0).id = (*state_0).id;
                (*match_0).pattern_index = (*state_0).pattern_index;
                (*match_0).capture_count = (*state_0).capture_count;
                (*match_0).captures = capture_list_pool_get(
                    &mut (*self_0).capture_list_pool,
                    (*state_0).capture_list_id,
                );
                *capture_index = (*state_0).consumed_capture_count as u32;
                (*state_0).consumed_capture_count =
                    (*state_0).consumed_capture_count.wrapping_add(1);
                return true;
            }
            if capture_list_pool_is_empty(&(*self_0).capture_list_pool) {
                capture_list_pool_release(
                    &mut (*self_0).capture_list_pool,
                    (*(*self_0)
                        .states
                        .contents
                        .offset(first_unfinished_state_index as isize))
                    .capture_list_id,
                );
                array__erase(
                    &mut (*self_0).states as *mut TSQueryCursorStates as *mut VoidArray,
                    ::std::mem::size_of::<QueryState>(),
                    first_unfinished_state_index,
                );
            }
        }
        // If there are no finished matches that are ready to be returned, then
        // continue finding more matches.
        if !ts_query_cursor__advance(self_0) {
            return false;
        }
    }
}
