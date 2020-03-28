pub use crate::{
    get_changed_ranges::*, language::*, lexer::*, node::*, parser::*, query::*, stack::*,
    subtree::*, tree::*, tree_cursor::*,
};

use libc::{
    calloc, clock_gettime, exit, fprintf, free, malloc, memcpy, memmove, memset, realloc, timespec,
    FILE,
};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

pub const TREE_SITTER_LANGUAGE_VERSION: usize = 11;

pub const TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION: usize = 9;

extern "C" {
    #[no_mangle]
    pub fn __assert_fail(
        __assertion: *const libc::c_char,
        __file: *const libc::c_char,
        __line: libc::c_uint,
        __function: *const libc::c_char,
    ) -> !;
    #[no_mangle]
    pub fn __ctype_b_loc() -> *mut *const libc::c_ushort;
    #[no_mangle]
    pub static mut stderr: *mut FILE;
    #[no_mangle]
    pub fn iswspace(__wc: wint_t) -> libc::c_int;
    #[no_mangle]
    pub fn iswalnum(__wc: wint_t) -> libc::c_int;
}

pub type size_t = libc::c_ulong;
pub type __uint8_t = libc::c_uchar;
pub type __int16_t = libc::c_short;
pub type __uint16_t = libc::c_ushort;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
pub type __int64_t = libc::c_long;
pub type int64_t = __int64_t;

pub type __uint64_t = libc::c_ulong;
pub type __time_t = libc::c_long;
pub type __clockid_t = libc::c_int;
pub type __syscall_slong_t = libc::c_long;

pub type UnicodeDecodeFunction =
    Option<unsafe extern "C" fn(_: *const uint8_t, _: uint32_t, _: *mut int32_t) -> uint32_t>;

pub type UChar32 = int32_t;
pub static mut LENGTH_UNDEFINED: Length = {
    Length {
        bytes: 0 as libc::c_int as uint32_t,
        extent: {
            TSPoint {
                row: 0 as libc::c_int as uint32_t,
                column: 1 as libc::c_int as uint32_t,
            }
        },
    }
};

pub static mut TS_DECODE_ERROR: int32_t = -(1 as libc::c_int);

pub static mut LENGTH_MAX: Length = {
    Length {
        bytes: 4_294_967_295 as libc::c_uint,
        extent: {
            TSPoint {
                row: 4_294_967_295 as libc::c_uint,
                column: 4_294_967_295 as libc::c_uint,
            }
        },
    }
};

pub type clockid_t = __clockid_t;
pub type uint64_t = __uint64_t;

pub type TSDuration = uint64_t;
// POSIX with monotonic clock support (Linux)
// * Represent a time as a monotonic (seconds, nanoseconds) pair.
// * Represent a duration as a number of microseconds.
//
// On these platforms, parse timeouts will correspond accurately to
// real time, regardless of what other processes are running.
pub type TSClock = timespec;

pub type TSLogType = libc::c_uint;
pub const TSLogTypeLex: TSLogType = 1;
pub const TSLogTypeParse: TSLogType = 0;

pub type CTypeRetType = libc::c_uint;
pub const _ISalnum: CTypeRetType = 8;
pub const _ISpunct: CTypeRetType = 4;
pub const _IScntrl: CTypeRetType = 2;
pub const _ISblank: CTypeRetType = 1;
pub const _ISgraph: CTypeRetType = 32768;
pub const _ISprint: CTypeRetType = 16384;
pub const _ISspace: CTypeRetType = 8192;
pub const _ISxdigit: CTypeRetType = 4096;
pub const _ISdigit: CTypeRetType = 2048;
pub const _ISalpha: CTypeRetType = 1024;
pub const _ISlower: CTypeRetType = 512;
pub const _ISupper: CTypeRetType = 256;

pub type wint_t = libc::c_uint;

pub static mut TS_TREE_STATE_NONE: TSStateId =
    (32767 as libc::c_int * 2 as libc::c_int + 1 as libc::c_int) as TSStateId;

pub type int16_t = __int16_t;
pub type int32_t = __int32_t;
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type TSSymbol = uint16_t;
pub type TSFieldId = uint16_t;
pub type TSSymbolType = libc::c_uint;

pub const TSSymbolTypeAuxiliary: TSSymbolType = 2;
pub const TSSymbolTypeAnonymous: TSSymbolType = 1;
pub const TSSymbolTypeRegular: TSSymbolType = 0;

pub type TSInputEncoding = libc::c_uint;
pub const TSInputEncodingUTF16: TSInputEncoding = 1;
pub const TSInputEncodingUTF8: TSInputEncoding = 0;

pub const IteratorDiffers: IteratorComparison = 0;
pub const IteratorMayDiffer: IteratorComparison = 1;
pub const IteratorMatches: IteratorComparison = 2;
pub type IteratorComparison = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryCapture {
    pub node: TSNode,
    pub index: uint32_t,
}

pub type TSQueryError = libc::c_uint;
pub const TSQueryErrorCapture: TSQueryError = 4;
pub const TSQueryErrorField: TSQueryError = 3;
pub const TSQueryErrorNodeType: TSQueryError = 2;
pub const TSQueryErrorSyntax: TSQueryError = 1;
pub const TSQueryErrorNone: TSQueryError = 0;

pub type TSQueryPredicateStepType = libc::c_uint;
pub const TSQueryPredicateStepTypeString: TSQueryPredicateStepType = 2;
pub const TSQueryPredicateStepTypeCapture: TSQueryPredicateStepType = 1;
pub const TSQueryPredicateStepTypeDone: TSQueryPredicateStepType = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryPredicateStep {
    pub type_0: TSQueryPredicateStepType,
    pub value_id: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryMatch {
    pub id: uint32_t,
    pub pattern_index: uint16_t,
    pub capture_count: uint16_t,
    pub captures: *const TSQueryCapture,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Iterator_0 {
    pub cursor: TreeCursor,
    pub language: *const TSLanguage,
    pub visible_depth: libc::c_uint,
    pub in_padding: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Lexer {
    pub data: TSLexer,
    pub current_position: Length,
    pub token_start_position: Length,
    pub token_end_position: Length,
    pub included_ranges: *mut TSRange,
    pub included_range_count: size_t,
    pub current_included_range_index: size_t,
    pub chunk: *const libc::c_char,
    pub chunk_start: uint32_t,
    pub chunk_size: uint32_t,
    pub lookahead_size: uint32_t,
    pub input: TSInput,
    pub logger: TSLogger,
    pub debug_buffer: [libc::c_char; 1024],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLogger {
    pub payload: *mut libc::c_void,
    pub log: Option<
        unsafe extern "C" fn(_: *mut libc::c_void, _: TSLogType, _: *const libc::c_char) -> (),
    >,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSInput {
    pub payload: *mut libc::c_void,
    pub read: Option<
        unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: uint32_t,
            _: TSPoint,
            _: *mut uint32_t,
        ) -> *const libc::c_char,
    >,
    pub encoding: TSInputEncoding,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TableEntry {
    pub actions: *const TSParseAction,
    pub action_count: uint32_t,
    pub is_reusable: bool,
}

pub type StackVersion = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReduceActionSet {
    pub contents: *mut ReduceAction,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReduceAction {
    pub count: uint32_t,
    pub symbol: TSSymbol,
    pub dynamic_precedence: libc::c_int,
    pub production_id: libc::c_ushort,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct U32Array {
    pub contents: *mut uint32_t,
    pub size: uint32_t,
    pub capacity: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLanguage {
    pub version: uint32_t,
    pub symbol_count: uint32_t,
    pub alias_count: uint32_t,
    pub token_count: uint32_t,
    pub external_token_count: uint32_t,
    pub symbol_names: *mut *const libc::c_char,
    pub symbol_metadata: *const TSSymbolMetadata,
    pub parse_table: *const uint16_t,
    pub parse_actions: *const TSParseActionEntry,
    pub lex_modes: *const TSLexMode,
    pub alias_sequences: *const TSSymbol,
    pub max_alias_sequence_length: uint16_t,
    pub lex_fn: Option<unsafe extern "C" fn(_: *mut TSLexer, _: TSStateId) -> bool>,
    pub keyword_lex_fn: Option<unsafe extern "C" fn(_: *mut TSLexer, _: TSStateId) -> bool>,
    pub keyword_capture_token: TSSymbol,
    pub external_scanner: TSLanguageExternalScanner,
    pub field_count: uint32_t,
    pub field_map_slices: *const TSFieldMapSlice,
    pub field_map_entries: *const TSFieldMapEntry,
    pub field_names: *mut *const libc::c_char,
    pub large_state_count: uint32_t,
    pub small_parse_table: *const uint16_t,
    pub small_parse_table_map: *const uint32_t,
    pub public_symbol_map: *const TSSymbol,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSFieldMapEntry {
    pub field_id: TSFieldId,
    pub child_index: uint8_t,
    pub inherited: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSFieldMapSlice {
    pub index: uint16_t,
    pub length: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLanguageExternalScanner {
    pub states: *const bool,
    pub symbol_map: *const TSSymbol,
    pub create: Option<unsafe extern "C" fn() -> *mut libc::c_void>,
    pub destroy: Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>,
    pub scan:
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: *mut TSLexer, _: *const bool) -> bool>,
    pub serialize:
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: *mut libc::c_char) -> libc::c_uint>,
    pub deserialize: Option<
        unsafe extern "C" fn(_: *mut libc::c_void, _: *const libc::c_char, _: libc::c_uint) -> (),
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLexer {
    pub lookahead: int32_t,
    pub result_symbol: TSSymbol,
    pub advance: Option<unsafe extern "C" fn(_: *mut TSLexer, _: bool) -> ()>,
    pub mark_end: Option<unsafe extern "C" fn(_: *mut TSLexer) -> ()>,
    pub get_column: Option<unsafe extern "C" fn(_: *mut TSLexer) -> uint32_t>,
    pub is_at_included_range_start: Option<unsafe extern "C" fn(_: *const TSLexer) -> bool>,
    pub eof: Option<unsafe extern "C" fn(_: *const TSLexer) -> bool>,
}
pub type TSStateId = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLexMode {
    pub lex_state: uint16_t,
    pub external_lex_state: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union TSParseActionEntry {
    pub action: TSParseAction,
    pub c2rust_unnamed: TSParseActionEntryContent,
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct TSParseActionEntryContent {
    pub count: uint8_t,
    #[bitfield(name = "reusable", ty = "bool", bits = "0..=0")]
    pub reusable: [u8; 1],
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct TSParseAction {
    pub params: TSParseActionParams,
    #[bitfield(name = "type_0", ty = "TSParseActionType", bits = "0..=3")]
    pub type_0: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 1],
}
pub type TSParseActionType = libc::c_uint;
pub const TSParseActionTypeRecover: TSParseActionType = 3;
pub const TSParseActionTypeAccept: TSParseActionType = 2;
pub const TSParseActionTypeReduce: TSParseActionType = 1;
pub const TSParseActionTypeShift: TSParseActionType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union TSParseActionParams {
    pub c2rust_unnamed: TSParseActionParamsState,
    pub c2rust_unnamed_0: TSParseActionParamsSymbol,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSParseActionParamsSymbol {
    pub symbol: TSSymbol,
    pub dynamic_precedence: int16_t,
    pub child_count: uint8_t,
    pub production_id: uint8_t,
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct TSParseActionParamsState {
    pub state: TSStateId,
    #[bitfield(name = "extra", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "repetition", ty = "bool", bits = "1..=1")]
    pub extra_repetition: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 1],
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct TSSymbolMetadata {
    #[bitfield(name = "visible", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "named", ty = "bool", bits = "1..=1")]
    pub visible_named: [u8; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSTree {
    pub root: Subtree,
    pub language: *const TSLanguage,
    pub parent_cache: *mut ParentCacheEntry,
    pub parent_cache_start: uint32_t,
    pub parent_cache_size: uint32_t,
    pub included_ranges: *mut TSRange,
    pub included_range_count: libc::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSRange {
    pub start_point: TSPoint,
    pub end_point: TSPoint,
    pub start_byte: uint32_t,
    pub end_byte: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSPoint {
    pub row: uint32_t,
    pub column: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParentCacheEntry {
    pub child: *const Subtree,
    pub parent: *const Subtree,
    pub position: Length,
    pub alias_symbol: TSSymbol,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Length {
    pub bytes: uint32_t,
    pub extent: TSPoint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union Subtree {
    pub data: SubtreeInlineData,
    pub ptr: *const SubtreeHeapData,
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct SubtreeHeapData {
    pub ref_count: uint32_t,
    pub padding: Length,
    pub size: Length,
    pub lookahead_bytes: uint32_t,
    pub error_cost: uint32_t,
    pub child_count: uint32_t,
    pub symbol: TSSymbol,
    pub parse_state: TSStateId,
    #[bitfield(name = "visible", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "named", ty = "bool", bits = "1..=1")]
    #[bitfield(name = "extra", ty = "bool", bits = "2..=2")]
    #[bitfield(name = "fragile_left", ty = "bool", bits = "3..=3")]
    #[bitfield(name = "fragile_right", ty = "bool", bits = "4..=4")]
    #[bitfield(name = "has_changes", ty = "bool", bits = "5..=5")]
    #[bitfield(name = "has_external_tokens", ty = "bool", bits = "6..=6")]
    #[bitfield(name = "is_missing", ty = "bool", bits = "7..=7")]
    #[bitfield(name = "is_keyword", ty = "bool", bits = "8..=8")]
    pub visible_named_extra_fragile_left_fragile_right_has_changes_has_external_tokens_is_missing_is_keyword:
        [u8; 2],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 2],
    pub c2rust_unnamed: SubtreeHeapDataContent,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union SubtreeHeapDataContent {
    pub c2rust_unnamed: SubtreeHeapDataContentData,
    pub external_scanner_state: ExternalScannerState,
    pub lookahead_char: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExternalScannerState {
    pub c2rust_unnamed: ExternalScannerStateData,
    pub length: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union ExternalScannerStateData {
    pub long_data: *mut libc::c_char,
    pub short_data: [libc::c_char; 24],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SubtreeHeapDataContentData {
    pub children: *mut Subtree,
    pub visible_child_count: uint32_t,
    pub named_child_count: uint32_t,
    pub node_count: uint32_t,
    pub repeat_depth: uint32_t,
    pub dynamic_precedence: int32_t,
    pub production_id: uint16_t,
    pub first_leaf: SubtreeHeapDataContentDataFirstLeaf,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SubtreeHeapDataContentDataFirstLeaf {
    pub symbol: TSSymbol,
    pub parse_state: TSStateId,
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct SubtreeInlineData {
    #[bitfield(name = "is_inline", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "visible", ty = "bool", bits = "1..=1")]
    #[bitfield(name = "named", ty = "bool", bits = "2..=2")]
    #[bitfield(name = "extra", ty = "bool", bits = "3..=3")]
    #[bitfield(name = "has_changes", ty = "bool", bits = "4..=4")]
    #[bitfield(name = "is_missing", ty = "bool", bits = "5..=5")]
    #[bitfield(name = "is_keyword", ty = "bool", bits = "6..=6")]
    pub is_inline_visible_named_extra_has_changes_is_missing_is_keyword: [u8; 1],
    pub symbol: uint8_t,
    pub padding_bytes: uint8_t,
    pub size_bytes: uint8_t,
    pub padding_columns: uint8_t,
    #[bitfield(name = "padding_rows", ty = "uint8_t", bits = "0..=3")]
    #[bitfield(name = "lookahead_bytes", ty = "uint8_t", bits = "4..=7")]
    pub padding_rows_lookahead_bytes: [u8; 1],
    pub parse_state: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSInputEdit {
    pub start_byte: uint32_t,
    pub old_end_byte: uint32_t,
    pub new_end_byte: uint32_t,
    pub start_point: TSPoint,
    pub old_end_point: TSPoint,
    pub new_end_point: TSPoint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSNode {
    pub context: [uint32_t; 4],
    pub id: *const libc::c_void,
    pub tree: *const TSTree,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TreeCursorEntry {
    pub subtree: *const Subtree,
    pub position: Length,
    pub child_index: uint32_t,
    pub structural_child_index: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SubtreeArray {
    pub contents: *mut Subtree,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SubtreePool {
    pub free_trees: MutableSubtreeArray,
    pub tree_stack: MutableSubtreeArray,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MutableSubtreeArray {
    pub contents: *mut MutableSubtree,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union MutableSubtree {
    pub data: SubtreeInlineData,
    pub ptr: *mut SubtreeHeapData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VoidArray {
    pub contents: *mut libc::c_void,
    pub size: uint32_t,
    pub capacity: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSRangeArray {
    pub contents: *mut TSRange,
    pub size: uint32_t,
    pub capacity: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSTreeCursor {
    pub tree: *const libc::c_void,
    pub id: *const libc::c_void,
    pub context: [uint32_t; 2],
}

// Private
#[inline]
pub unsafe extern "C" fn ts_malloc(mut size: size_t) -> *mut libc::c_void {
    let mut result: *mut libc::c_void = malloc(size as usize);
    if size > 0 as libc::c_int as libc::c_ulong && result.is_null() {
        fprintf(
            stderr,
            b"tree-sitter failed to allocate %lu bytes\x00" as *const u8 as *const libc::c_char,
            size,
        );
        exit(1 as libc::c_int);
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn ts_calloc(mut count: size_t, mut size: size_t) -> *mut libc::c_void {
    let mut result: *mut libc::c_void = calloc(count as usize, size as usize);
    if count > 0 as libc::c_int as libc::c_ulong && result.is_null() {
        fprintf(
            stderr,
            b"tree-sitter failed to allocate %lu bytes\x00" as *const u8 as *const libc::c_char,
            count.wrapping_mul(size),
        );
        exit(1 as libc::c_int);
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn ts_free(mut buffer: *mut libc::c_void) {
    free(buffer);
}
#[inline]
pub unsafe extern "C" fn ts_subtree_padding(mut self_0: Subtree) -> Length {
    if self_0.data.is_inline() {
        return Length {
            bytes: self_0.data.padding_bytes as uint32_t,
            extent: {
                TSPoint {
                    row: self_0.data.padding_rows() as uint32_t,
                    column: self_0.data.padding_columns as uint32_t,
                }
            },
        };
    } else {
        return (*self_0.ptr).padding;
    };
}
#[inline]
pub unsafe extern "C" fn length_zero() -> Length {
    Length {
        bytes: 0 as libc::c_int as uint32_t,
        extent: {
            TSPoint {
                row: 0 as libc::c_int as uint32_t,
                column: 0 as libc::c_int as uint32_t,
            }
        },
    }
}
#[inline]
pub unsafe extern "C" fn point_sub(mut a: TSPoint, mut b: TSPoint) -> TSPoint {
    if a.row > b.row {
        return point__new(a.row.wrapping_sub(b.row), a.column);
    } else {
        return point__new(
            0 as libc::c_int as libc::c_uint,
            a.column.wrapping_sub(b.column),
        );
    };
}

#[inline]
pub unsafe extern "C" fn ts_subtree_symbol(mut self_0: Subtree) -> TSSymbol {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.symbol as libc::c_int
    } else {
        (*self_0.ptr).symbol as libc::c_int
    } as TSSymbol;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_size(mut self_0: Subtree) -> Length {
    if self_0.data.is_inline() {
        return Length {
            bytes: self_0.data.size_bytes as uint32_t,
            extent: {
                TSPoint {
                    row: 0 as libc::c_int as uint32_t,
                    column: self_0.data.size_bytes as uint32_t,
                }
            },
        };
    } else {
        return (*self_0.ptr).size;
    }
}

#[inline]
pub unsafe extern "C" fn point_add(mut a: TSPoint, mut b: TSPoint) -> TSPoint {
    if b.row > 0 as libc::c_int as libc::c_uint {
        return point__new(a.row.wrapping_add(b.row), b.column);
    } else {
        return point__new(a.row, a.column.wrapping_add(b.column));
    };
}
#[inline]
pub unsafe extern "C" fn point__new(mut row: libc::c_uint, mut column: libc::c_uint) -> TSPoint {
    TSPoint {
        row: row,
        column: column,
    }
}
#[inline]
pub unsafe extern "C" fn ts_subtree_named(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.named() as libc::c_int
    } else {
        (*self_0.ptr).named() as libc::c_int
    } != 0;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_missing(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.is_missing() as libc::c_int
    } else {
        (*self_0.ptr).is_missing() as libc::c_int
    } != 0;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_extra(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.extra() as libc::c_int
    } else {
        (*self_0.ptr).extra() as libc::c_int
    } != 0;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_has_changes(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.has_changes() as libc::c_int
    } else {
        (*self_0.ptr).has_changes() as libc::c_int
    } != 0;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_error_cost(mut self_0: Subtree) -> uint32_t {
    if ts_subtree_missing(self_0) {
        return (110 as libc::c_int + 500 as libc::c_int) as uint32_t;
    } else {
        return if self_0.data.is_inline() as libc::c_int != 0 {
            0 as libc::c_int as libc::c_uint
        } else {
            (*self_0.ptr).error_cost
        };
    }
}
pub unsafe extern "C" fn ts_subtree_visible(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.visible() as libc::c_int
    } else {
        (*self_0.ptr).visible() as libc::c_int
    } != 0;
}
pub unsafe extern "C" fn ts_subtree_child_count(mut self_0: Subtree) -> uint32_t {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        0 as libc::c_int as libc::c_uint
    } else {
        (*self_0.ptr).child_count
    };
}
pub unsafe extern "C" fn length_add(mut len1: Length, mut len2: Length) -> Length {
    let mut result: Length = Length {
        bytes: 0,
        extent: TSPoint { row: 0, column: 0 },
    };
    result.bytes = len1.bytes.wrapping_add(len2.bytes);
    result.extent = point_add(len1.extent, len2.extent);
    return result;
}
pub unsafe extern "C" fn ts_subtree_total_bytes(mut self_0: Subtree) -> uint32_t {
    return ts_subtree_total_size(self_0).bytes;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_total_size(mut self_0: Subtree) -> Length {
    return length_add(ts_subtree_padding(self_0), ts_subtree_size(self_0));
}
#[inline]
pub unsafe extern "C" fn point_lt(mut a: TSPoint, mut b: TSPoint) -> bool {
    return a.row < b.row || a.row == b.row && a.column < b.column;
}
pub unsafe extern "C" fn point_lte(mut a: TSPoint, mut b: TSPoint) -> bool {
    return a.row < b.row || a.row == b.row && a.column <= b.column;
}

pub unsafe extern "C" fn ts_language_alias_sequence(
    mut self_0: *const TSLanguage,
    mut production_id: uint32_t,
) -> *const TSSymbol {
    return if production_id > 0 as libc::c_int as libc::c_uint {
        (*self_0).alias_sequences.offset(
            production_id.wrapping_mul((*self_0).max_alias_sequence_length as libc::c_uint)
                as isize,
        )
    } else {
        std::ptr::null::<TSSymbol>()
    };
}
#[inline]
pub unsafe extern "C" fn ts_language_field_map(
    mut self_0: *const TSLanguage,
    mut production_id: uint32_t,
    mut start: *mut *const TSFieldMapEntry,
    mut end: *mut *const TSFieldMapEntry,
) {
    if (*self_0).version < 10 as libc::c_int as libc::c_uint
        || (*self_0).field_count == 0 as libc::c_int as libc::c_uint
    {
        *start = std::ptr::null::<TSFieldMapEntry>();
        *end = std::ptr::null::<TSFieldMapEntry>();
        return;
    }
    let mut slice: TSFieldMapSlice = *(*self_0).field_map_slices.offset(production_id as isize);
    *start = &*(*self_0).field_map_entries.offset(slice.index as isize) as *const TSFieldMapEntry;
    *end = (&*(*self_0).field_map_entries.offset(slice.index as isize) as *const TSFieldMapEntry)
        .offset(slice.length as libc::c_int as isize);
}

//TSTreeCursor
#[inline]
pub unsafe extern "C" fn ts_realloc(
    mut buffer: *mut libc::c_void,
    mut size: size_t,
) -> *mut libc::c_void {
    let mut result: *mut libc::c_void = realloc(buffer, size as usize);
    if size > 0 as libc::c_int as libc::c_ulong && result.is_null() {
        fprintf(
            stderr,
            b"tree-sitter failed to reallocate %lu bytes\x00" as *const u8 as *const libc::c_char,
            size,
        );
        exit(1 as libc::c_int);
    }
    return result;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_visible_child_count(mut self_0: Subtree) -> uint32_t {
    if ts_subtree_child_count(self_0) > 0 as libc::c_int as libc::c_uint {
        return (*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .visible_child_count;
    } else {
        return 0 as libc::c_int as uint32_t;
    };
}

#[inline]
pub unsafe extern "C" fn array__splice(
    mut self_0: *mut VoidArray,
    mut element_size: size_t,
    mut index: uint32_t,
    mut old_count: uint32_t,
    mut new_count: uint32_t,
    mut elements: *const libc::c_void,
) {
    let mut new_size: uint32_t = (*self_0)
        .size
        .wrapping_add(new_count)
        .wrapping_sub(old_count);
    let mut old_end: uint32_t = index.wrapping_add(old_count);
    let mut new_end: uint32_t = index.wrapping_add(new_count);
    if old_end <= (*self_0).size {
    } else {
        __assert_fail(b"old_end <= self->size\x00" as *const u8 as
                          *const libc::c_char,
                      b"lib/src/./array.h\x00" as *const u8 as
                          *const libc::c_char,
                      124 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"void array__splice(VoidArray *, size_t, uint32_t, uint32_t, uint32_t, const void *)\x00")).as_ptr());
    }
    array__reserve(self_0, element_size, new_size);
    let mut contents: *mut libc::c_char = (*self_0).contents as *mut libc::c_char;
    if (*self_0).size > old_end {
        memmove(
            contents.offset((new_end as libc::c_ulong).wrapping_mul(element_size) as isize)
                as *mut libc::c_void,
            contents.offset((old_end as libc::c_ulong).wrapping_mul(element_size) as isize)
                as *const libc::c_void,
            ((*self_0).size.wrapping_sub(old_end) as libc::c_ulong).wrapping_mul(element_size)
                as usize,
        );
    }
    if new_count > 0 as libc::c_int as libc::c_uint {
        if !elements.is_null() {
            memcpy(
                contents.offset((index as libc::c_ulong).wrapping_mul(element_size) as isize)
                    as *mut libc::c_void,
                elements,
                (new_count as libc::c_ulong).wrapping_mul(element_size) as usize,
            );
        } else {
            memset(
                contents.offset((index as libc::c_ulong).wrapping_mul(element_size) as isize)
                    as *mut libc::c_void,
                0 as libc::c_int,
                (new_count as libc::c_ulong).wrapping_mul(element_size) as usize,
            );
        }
    }
    (*self_0).size = ((*self_0).size as libc::c_uint)
        .wrapping_add(new_count.wrapping_sub(old_count)) as uint32_t
        as uint32_t;
}

#[inline]
pub unsafe extern "C" fn array__delete(mut self_0: *mut VoidArray) {
    ts_free((*self_0).contents);
    (*self_0).contents = 0 as *mut libc::c_void;
    (*self_0).size = 0 as libc::c_int as uint32_t;
    (*self_0).capacity = 0 as libc::c_int as uint32_t;
}
#[inline]
pub unsafe extern "C" fn array__reserve(
    mut self_0: *mut VoidArray,
    mut element_size: size_t,
    mut new_capacity: uint32_t,
) {
    if new_capacity > (*self_0).capacity {
        if !(*self_0).contents.is_null() {
            (*self_0).contents = ts_realloc(
                (*self_0).contents,
                (new_capacity as libc::c_ulong).wrapping_mul(element_size),
            )
        } else {
            (*self_0).contents = ts_calloc(new_capacity as size_t, element_size)
        }
        (*self_0).capacity = new_capacity
    };
}
#[inline]
pub unsafe extern "C" fn array__grow(
    mut self_0: *mut VoidArray,
    mut count: size_t,
    mut element_size: size_t,
) {
    let mut new_size: size_t = ((*self_0).size as libc::c_ulong).wrapping_add(count);
    if new_size > (*self_0).capacity as libc::c_ulong {
        let mut new_capacity: size_t = (*self_0)
            .capacity
            .wrapping_mul(2 as libc::c_int as libc::c_uint)
            as size_t;
        if new_capacity < 8 as libc::c_int as libc::c_ulong {
            new_capacity = 8 as libc::c_int as size_t
        }
        if new_capacity < new_size {
            new_capacity = new_size
        }
        array__reserve(self_0, element_size, new_capacity as uint32_t);
    };
}

// Subtree

#[inline]
pub unsafe extern "C" fn atomic_inc(p: *const uint32_t) -> uint32_t {
    (&*(p as *const AtomicU32))
        .fetch_add(1, Ordering::SeqCst)
        .wrapping_add(1)
}

#[inline]
pub unsafe extern "C" fn atomic_dec(mut p: *mut uint32_t) -> uint32_t {
    (&*(p as *const AtomicU32))
        .fetch_sub(1, Ordering::SeqCst)
        .wrapping_sub(1)
}

#[inline]
pub unsafe extern "C" fn length_sub(mut len1: Length, mut len2: Length) -> Length {
    let mut result: Length = Length {
        bytes: 0,
        extent: TSPoint { row: 0, column: 0 },
    };
    result.bytes = len1.bytes.wrapping_sub(len2.bytes);
    result.extent = point_sub(len1.extent, len2.extent);
    return result;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_has_external_tokens(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        0 as libc::c_int
    } else {
        (*self_0.ptr).has_external_tokens() as libc::c_int
    } != 0;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_repeat_depth(mut self_0: Subtree) -> uint32_t {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        0 as libc::c_int as libc::c_uint
    } else {
        (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.repeat_depth
    };
}

pub unsafe extern "C" fn ts_subtree_fragile_right(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        0 as libc::c_int
    } else {
        (*self_0.ptr).fragile_right() as libc::c_int
    } != 0;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_fragile_left(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        0 as libc::c_int
    } else {
        (*self_0.ptr).fragile_left() as libc::c_int
    } != 0;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_leaf_parse_state(mut self_0: Subtree) -> TSStateId {
    if self_0.data.is_inline() {
        return self_0.data.parse_state;
    }
    if (*self_0.ptr).child_count == 0 as libc::c_int as libc::c_uint {
        return (*self_0.ptr).parse_state;
    }
    return (*self_0.ptr)
        .c2rust_unnamed
        .c2rust_unnamed
        .first_leaf
        .parse_state;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_leaf_symbol(mut self_0: Subtree) -> TSSymbol {
    if self_0.data.is_inline() {
        return self_0.data.symbol as TSSymbol;
    }
    if (*self_0.ptr).child_count == 0 as libc::c_int as libc::c_uint {
        return (*self_0.ptr).symbol;
    }
    return (*self_0.ptr)
        .c2rust_unnamed
        .c2rust_unnamed
        .first_leaf
        .symbol;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_parse_state(mut self_0: Subtree) -> TSStateId {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.parse_state as libc::c_int
    } else {
        (*self_0.ptr).parse_state as libc::c_int
    } as TSStateId;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_node_count(mut self_0: Subtree) -> uint32_t {
    return if self_0.data.is_inline() as libc::c_int != 0
        || (*self_0.ptr).child_count == 0 as libc::c_int as libc::c_uint
    {
        1 as libc::c_int as libc::c_uint
    } else {
        (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.node_count
    };
}
#[inline]
pub unsafe extern "C" fn ts_subtree_dynamic_precedence(mut self_0: Subtree) -> int32_t {
    return if self_0.data.is_inline() as libc::c_int != 0
        || (*self_0.ptr).child_count == 0 as libc::c_int as libc::c_uint
    {
        0 as libc::c_int
    } else {
        (*self_0.ptr)
            .c2rust_unnamed
            .c2rust_unnamed
            .dynamic_precedence
    };
}

#[inline]
pub unsafe extern "C" fn ts_subtree_lookahead_bytes(mut self_0: Subtree) -> uint32_t {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.lookahead_bytes() as libc::c_uint
    } else {
        (*self_0.ptr).lookahead_bytes
    };
}

#[inline]
pub unsafe extern "C" fn ts_subtree_production_id(mut self_0: Subtree) -> uint16_t {
    if ts_subtree_child_count(self_0) > 0 as libc::c_int as libc::c_uint {
        return (*self_0.ptr).c2rust_unnamed.c2rust_unnamed.production_id;
    } else {
        return 0 as libc::c_int as uint16_t;
    };
}

#[inline]
pub unsafe extern "C" fn ts_subtree_is_error(mut self_0: Subtree) -> bool {
    return ts_subtree_symbol(self_0) as libc::c_int
        == -(1 as libc::c_int) as TSSymbol as libc::c_int;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_from_mut(mut self_0: MutableSubtree) -> Subtree {
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
    result.data = self_0.data;
    return result;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_to_mut_unsafe(mut self_0: Subtree) -> MutableSubtree {
    let mut result: MutableSubtree = MutableSubtree {
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
    result.data = self_0.data;
    return result;
}

// Stack
#[inline]
pub unsafe extern "C" fn array__erase(
    mut self_0: *mut VoidArray,
    mut element_size: size_t,
    mut index: uint32_t,
) {
    if index < (*self_0).size {
    } else {
        __assert_fail(
            b"index < self->size\x00" as *const u8 as *const libc::c_char,
            b"lib/src/./array.h\x00" as *const u8 as *const libc::c_char,
            84 as libc::c_int as libc::c_uint,
            (*::std::mem::transmute::<&[u8; 49], &[libc::c_char; 49]>(
                b"void array__erase(VoidArray *, size_t, uint32_t)\x00",
            ))
            .as_ptr(),
        );
    }
    let mut contents: *mut libc::c_char = (*self_0).contents as *mut libc::c_char;
    memmove(
        contents.offset((index as libc::c_ulong).wrapping_mul(element_size) as isize)
            as *mut libc::c_void,
        contents.offset(
            (index.wrapping_add(1 as libc::c_int as libc::c_uint) as libc::c_ulong)
                .wrapping_mul(element_size) as isize,
        ) as *const libc::c_void,
        ((*self_0)
            .size
            .wrapping_sub(index)
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_ulong)
            .wrapping_mul(element_size) as usize,
    );
    (*self_0).size = (*self_0).size.wrapping_sub(1);
}

#[inline]
pub unsafe extern "C" fn ts_toggle_allocation_recording(mut _value: bool) -> bool {
    return 0 as libc::c_int != 0;
}

// Query
#[inline]
pub unsafe extern "C" fn count_leading_zeros(mut x: uint32_t) -> uint32_t {
    if x == 0 as libc::c_int as libc::c_uint {
        return 32 as libc::c_int as uint32_t;
    }
    return x.leading_zeros() as i32 as uint32_t;
}

#[inline]
pub unsafe extern "C" fn bitmask_for_index(mut id: uint16_t) -> uint32_t {
    return (1 as libc::c_uint) << 31 as libc::c_int - id as libc::c_int;
}

#[inline]
pub unsafe extern "C" fn ts_decode_utf8(
    mut string: *const uint8_t,
    mut length: uint32_t,
    mut code_point: *mut int32_t,
) -> uint32_t {
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    let fresh0 = i;
    i = i.wrapping_add(1);
    *code_point = *string.offset(fresh0 as isize) as int32_t;
    if !(*code_point & 0x80 as libc::c_int == 0 as libc::c_int) {
        let mut __t: uint8_t = 0 as libc::c_int as uint8_t;
        if !(i != length
            && (if *code_point >= 0xe0 as libc::c_int {
                ((if *code_point < 0xf0 as libc::c_int {
                    *code_point &= 0xf as libc::c_int;
                    __t = *string.offset(i as isize);
                    ((*::std::mem::transmute::<&[u8; 17], &[libc::c_char; 17]>(
                        b" 000000000000\x1000\x00",
                    ))[*code_point as usize] as libc::c_int
                        & (1 as libc::c_int) << (__t as libc::c_int >> 5 as libc::c_int)
                        != 0
                        && {
                            __t = (__t as libc::c_int & 0x3f as libc::c_int) as uint8_t;
                            (1 as libc::c_int) != 0
                        }) as libc::c_int
                } else {
                    *code_point -= 0xf0 as libc::c_int;
                    (*code_point <= 4 as libc::c_int
                        && {
                            __t = *string.offset(i as isize);
                            ((*::std::mem::transmute::<&[u8; 17],
                                                                &[libc::c_char; 17]>(b"\x00\x00\x00\x00\x00\x00\x00\x00\x1e\x0f\x0f\x0f\x00\x00\x00\x00\x00"))[(__t
                                                                                                                                                                    as
                                                                                                                                                                    libc::c_int
                                                                                                                                                                    >>
                                                                                                                                                                    4
                                                                                                                                                                        as
                                                                                                                                                                        libc::c_int)
                                                                                                                                                                   as
                                                                                                                                                                   usize]
                                          as libc::c_int &
                                          (1 as libc::c_int) << *code_point)
                                         != 0
                        }
                        && {
                            *code_point = *code_point << 6 as libc::c_int
                                | __t as libc::c_int & 0x3f as libc::c_int;
                            i = i.wrapping_add(1);
                            (i) != length
                        }
                        && {
                            __t = (*string.offset(i as isize) as libc::c_int - 0x80 as libc::c_int)
                                as uint8_t;
                            (__t as libc::c_int) <= 0x3f as libc::c_int
                        }) as libc::c_int
                }) != 0
                    && {
                        *code_point = *code_point << 6 as libc::c_int | __t as libc::c_int;
                        i = i.wrapping_add(1);
                        (i) != length
                    }) as libc::c_int
            } else {
                (*code_point >= 0xc2 as libc::c_int && {
                    *code_point &= 0x1f as libc::c_int;
                    (1 as libc::c_int) != 0
                }) as libc::c_int
            }) != 0
            && {
                __t = (*string.offset(i as isize) as libc::c_int - 0x80 as libc::c_int) as uint8_t;
                (__t as libc::c_int) <= 0x3f as libc::c_int
            }
            && {
                *code_point = *code_point << 6 as libc::c_int | __t as libc::c_int;
                i = i.wrapping_add(1);
                (1 as libc::c_int) != 0
            })
        {
            *code_point = -(1 as libc::c_int)
        }
    }
    return i;
}

// Parser

#[inline]
pub unsafe extern "C" fn clock_now() -> TSClock {
    let mut result: TSClock = TSClock {
        tv_sec: 0,
        tv_nsec: 0,
    };
    clock_gettime(libc::CLOCK_MONOTONIC, &mut result);
    return result;
}
#[inline]
pub unsafe extern "C" fn clock_null() -> TSClock {
    return {
        let mut init = timespec {
            tv_sec: 0 as libc::c_int as __time_t,
            tv_nsec: 0 as libc::c_int as __syscall_slong_t,
        };
        init
    };
}
#[inline]
pub unsafe extern "C" fn clock_after(mut base: TSClock, mut duration: TSDuration) -> TSClock {
    let mut result: TSClock = base;
    result.tv_sec = (result.tv_sec as libc::c_ulong)
        .wrapping_add(duration.wrapping_div(1000000 as libc::c_int as libc::c_ulong))
        as __time_t as __time_t;
    result.tv_nsec = (result.tv_nsec as libc::c_ulong).wrapping_add(
        duration
            .wrapping_rem(1000000 as libc::c_int as libc::c_ulong)
            .wrapping_mul(1000 as libc::c_int as libc::c_ulong),
    ) as __syscall_slong_t as __syscall_slong_t;
    return result;
}
#[inline]
pub unsafe extern "C" fn clock_is_null(mut self_0: TSClock) -> bool {
    return self_0.tv_sec == 0;
}
#[inline]
pub unsafe extern "C" fn clock_is_gt(mut self_0: TSClock, mut other: TSClock) -> bool {
    if self_0.tv_sec > other.tv_sec {
        return 1 as libc::c_int != 0;
    }
    if self_0.tv_sec < other.tv_sec {
        return 0 as libc::c_int != 0;
    }
    return self_0.tv_nsec > other.tv_nsec;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_is_keyword(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        self_0.data.is_keyword() as libc::c_int
    } else {
        (*self_0.ptr).is_keyword() as libc::c_int
    } != 0;
}

#[inline]
pub unsafe extern "C" fn ts_subtree_set_extra(mut self_0: *mut MutableSubtree) {
    if (*self_0).data.is_inline() {
        (*self_0).data.set_extra(1 as libc::c_int != 0)
    } else {
        (*(*self_0).ptr).set_extra(1 as libc::c_int != 0)
    };
}

#[inline]
pub unsafe extern "C" fn ts_subtree_is_eof(mut self_0: Subtree) -> bool {
    return ts_subtree_symbol(self_0) as libc::c_int == 0 as libc::c_int;
}
#[inline]
pub unsafe extern "C" fn ts_subtree_is_fragile(mut self_0: Subtree) -> bool {
    return if self_0.data.is_inline() as libc::c_int != 0 {
        0 as libc::c_int
    } else {
        ((*self_0.ptr).fragile_left() as libc::c_int != 0
            || (*self_0.ptr).fragile_right() as libc::c_int != 0) as libc::c_int
    } != 0;
}

#[inline]
pub unsafe extern "C" fn ts_language_actions(
    mut self_0: *const TSLanguage,
    mut state: TSStateId,
    mut symbol: TSSymbol,
    mut count: *mut uint32_t,
) -> *const TSParseAction {
    let mut entry: TableEntry = TableEntry {
        actions: std::ptr::null::<TSParseAction>(),
        action_count: 0,
        is_reusable: false,
    };
    ts_language_table_entry(self_0, state, symbol, &mut entry);
    *count = entry.action_count;
    return entry.actions;
}
#[inline]
pub unsafe extern "C" fn ts_language_has_actions(
    mut self_0: *const TSLanguage,
    mut state: TSStateId,
    mut symbol: TSSymbol,
) -> bool {
    let mut entry: TableEntry = TableEntry {
        actions: std::ptr::null::<TSParseAction>(),
        action_count: 0,
        is_reusable: false,
    };
    ts_language_table_entry(self_0, state, symbol, &mut entry);
    return entry.action_count > 0 as libc::c_int as libc::c_uint;
}
#[inline]
pub unsafe extern "C" fn ts_language_has_reduce_action(
    mut self_0: *const TSLanguage,
    mut state: TSStateId,
    mut symbol: TSSymbol,
) -> bool {
    let mut entry: TableEntry = TableEntry {
        actions: std::ptr::null::<TSParseAction>(),
        action_count: 0,
        is_reusable: false,
    };
    ts_language_table_entry(self_0, state, symbol, &mut entry);
    return entry.action_count > 0 as libc::c_int as libc::c_uint
        && (*entry.actions.offset(0 as libc::c_int as isize)).type_0() as libc::c_int
            == TSParseActionTypeReduce as libc::c_int;
}
#[inline]
pub unsafe extern "C" fn ts_language_lookup(
    mut self_0: *const TSLanguage,
    mut state: TSStateId,
    mut symbol: TSSymbol,
) -> uint16_t {
    if (*self_0).version >= 11 as libc::c_int as libc::c_uint
        && state as libc::c_uint >= (*self_0).large_state_count
    {
        let mut index: uint32_t = *(*self_0)
            .small_parse_table_map
            .offset((state as libc::c_uint).wrapping_sub((*self_0).large_state_count) as isize);
        let mut data: *const uint16_t =
            &*(*self_0).small_parse_table.offset(index as isize) as *const uint16_t;
        let fresh0 = data;
        data = data.offset(1);
        let mut section_count: uint16_t = *fresh0;
        let mut i: libc::c_uint = 0 as libc::c_int as libc::c_uint;
        while i < section_count as libc::c_uint {
            let fresh1 = data;
            data = data.offset(1);
            let mut section_value: uint16_t = *fresh1;
            let fresh2 = data;
            data = data.offset(1);
            let mut symbol_count: uint16_t = *fresh2;
            let mut i_0: libc::c_uint = 0 as libc::c_int as libc::c_uint;
            while i_0 < symbol_count as libc::c_uint {
                let fresh3 = data;
                data = data.offset(1);
                if *fresh3 as libc::c_int == symbol as libc::c_int {
                    return section_value;
                }
                i_0 = i_0.wrapping_add(1)
            }
            i = i.wrapping_add(1)
        }
        return 0 as libc::c_int as uint16_t;
    } else {
        return *(*self_0).parse_table.offset(
            (state as libc::c_uint)
                .wrapping_mul((*self_0).symbol_count)
                .wrapping_add(symbol as libc::c_uint) as isize,
        );
    };
}
#[inline]
pub unsafe extern "C" fn ts_language_next_state(
    mut self_0: *const TSLanguage,
    mut state: TSStateId,
    mut symbol: TSSymbol,
) -> TSStateId {
    if symbol as libc::c_int == -(1 as libc::c_int) as TSSymbol as libc::c_int
        || symbol as libc::c_int
            == -(1 as libc::c_int) as TSSymbol as libc::c_int - 1 as libc::c_int
    {
        return 0 as libc::c_int as TSStateId;
    } else if (symbol as libc::c_uint) < (*self_0).token_count {
        let mut count: uint32_t = 0;
        let mut actions: *const TSParseAction =
            ts_language_actions(self_0, state, symbol, &mut count);
        if count > 0 as libc::c_int as libc::c_uint {
            let mut action: TSParseAction =
                *actions.offset(count.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize);
            if action.type_0() as libc::c_int == TSParseActionTypeShift as libc::c_int {
                return if action.params.c2rust_unnamed.extra() as libc::c_int != 0 {
                    state as libc::c_int
                } else {
                    action.params.c2rust_unnamed.state as libc::c_int
                } as TSStateId;
            }
        }
        return 0 as libc::c_int as TSStateId;
    } else {
        return ts_language_lookup(self_0, state, symbol);
    };
}
#[inline]
pub unsafe extern "C" fn ts_language_enabled_external_tokens(
    mut self_0: *const TSLanguage,
    mut external_scanner_state: libc::c_uint,
) -> *const bool {
    if external_scanner_state == 0 as libc::c_int as libc::c_uint {
        return std::ptr::null::<bool>();
    } else {
        return (*self_0).external_scanner.states.offset(
            (*self_0)
                .external_token_count
                .wrapping_mul(external_scanner_state) as isize,
        );
    };
}
#[inline]
pub unsafe extern "C" fn ts_reduce_action_set_add(
    mut self_0: *mut ReduceActionSet,
    mut new_action: ReduceAction,
) {
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    while i < (*self_0).size {
        let mut action: ReduceAction = *(*self_0).contents.offset(i as isize);
        if action.symbol as libc::c_int == new_action.symbol as libc::c_int
            && action.count == new_action.count
        {
            return;
        }
        i = i.wrapping_add(1)
    }
    array__grow(
        self_0 as *mut VoidArray,
        1 as libc::c_int as size_t,
        ::std::mem::size_of::<ReduceAction>() as libc::c_ulong,
    );
    let fresh4 = (*self_0).size;
    (*self_0).size = (*self_0).size.wrapping_add(1);
    *(*self_0).contents.offset(fresh4 as isize) = new_action;
}

#[inline]
pub unsafe extern "C" fn duration_from_micros(mut micros: uint64_t) -> TSDuration {
    return micros;
}

#[inline]
pub unsafe extern "C" fn duration_to_micros(mut self_0: TSDuration) -> uint64_t {
    return self_0;
}

#[inline]
pub unsafe extern "C" fn atomic_load(mut p: *const size_t) -> size_t {
    (&*(p as *const AtomicUsize)).load(Ordering::SeqCst) as size_t
}

// Lexer

#[inline]
pub unsafe extern "C" fn ts_decode_utf16(
    mut string: *const uint8_t,
    mut length: uint32_t,
    mut code_point: *mut int32_t,
) -> uint32_t {
    let mut i: uint32_t = 0 as libc::c_int as uint32_t;
    let fresh1 = i;
    i = i.wrapping_add(1);

    assert_eq!(string.align_offset(std::mem::align_of::<uint16_t>()), 0);
    #[allow(clippy::cast_ptr_alignment)]
    let string = string as *mut uint16_t;

    *code_point = *(string as *mut uint16_t).offset(fresh1 as isize) as int32_t;
    if *code_point as libc::c_uint & 0xfffffc00 as libc::c_uint
        == 0xd800 as libc::c_int as libc::c_uint
    {
        let mut __c2: uint16_t = 0;
        if i != length && {
            __c2 = *(string as *mut uint16_t).offset(i as isize);
            (__c2 as libc::c_uint & 0xfffffc00 as libc::c_uint)
                == 0xdc00 as libc::c_int as libc::c_uint
        } {
            i = i.wrapping_add(1);
            *code_point = (*code_point << 10 as libc::c_ulong) + __c2 as UChar32
                - (((0xd800 as libc::c_int) << 10 as libc::c_ulong) + 0xdc00 as libc::c_int
                    - 0x10000 as libc::c_int)
        }
    }
    return i.wrapping_mul(2 as libc::c_int as libc::c_uint);
}

#[inline]
pub unsafe extern "C" fn length_is_undefined(mut length: Length) -> bool {
    return length.bytes == 0 as libc::c_int as libc::c_uint
        && length.extent.column != 0 as libc::c_int as libc::c_uint;
}

// get_changed_ranges
#[inline]
pub unsafe extern "C" fn length_min(mut len1: Length, mut len2: Length) -> Length {
    return if len1.bytes < len2.bytes { len1 } else { len2 };
}
