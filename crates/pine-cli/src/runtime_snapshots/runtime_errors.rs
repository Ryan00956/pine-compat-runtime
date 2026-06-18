pub(crate) const RUNTIME_ERROR_FIXTURES: &[(&str, &str)] = &[
    (
        "tests/fixtures/regressions/array_concat_limit.pine",
        "array.concat cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_join_result_limit.pine",
        "array.join result cannot exceed 40960 characters",
    ),
    (
        "tests/fixtures/regressions/array_push_limit.pine",
        "array.push cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_insert_limit.pine",
        "array.insert cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_unshift_limit.pine",
        "array.unshift cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_sort_unsupported_order.pine",
        "unsupported array.sort order `sideways`",
    ),
    (
        "tests/fixtures/regressions/array_sort_indices_unsupported_order.pine",
        "unsupported array.sort_indices order `sideways`",
    ),
    (
        "tests/fixtures/regressions/array_new_float_negative_size.pine",
        "array.new_float size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_float_size_limit.pine",
        "array.new_float size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_int_negative_size.pine",
        "array.new_int size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_int_size_limit.pine",
        "array.new_int size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_bool_negative_size.pine",
        "array.new_bool size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_bool_size_limit.pine",
        "array.new_bool size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_string_negative_size.pine",
        "array.new_string size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_string_size_limit.pine",
        "array.new_string size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_color_negative_size.pine",
        "array.new_color size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_color_size_limit.pine",
        "array.new_color size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_line_negative_size.pine",
        "array.new_line size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_line_size_limit.pine",
        "array.new_line size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_label_negative_size.pine",
        "array.new_label size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_label_size_limit.pine",
        "array.new_label size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_box_negative_size.pine",
        "array.new_box size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_box_size_limit.pine",
        "array.new_box size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/array_new_table_negative_size.pine",
        "array.new_table size cannot be negative",
    ),
    (
        "tests/fixtures/regressions/array_new_table_size_limit.pine",
        "array.new_table size cannot exceed 100000 elements",
    ),
    (
        "tests/fixtures/regressions/label_new_limit.pine",
        "label count cannot exceed 500",
    ),
    (
        "tests/fixtures/regressions/line_new_limit.pine",
        "line count cannot exceed 500",
    ),
    (
        "tests/fixtures/regressions/box_new_limit.pine",
        "box count cannot exceed 500",
    ),
    (
        "tests/fixtures/regressions/table_new_count_limit.pine",
        "table count cannot exceed 50",
    ),
    (
        "tests/fixtures/regressions/table_new_cell_limit.pine",
        "table cell count cannot exceed 1000",
    ),
    (
        "tests/fixtures/regressions/table_new_positive_dimensions.pine",
        "table dimensions must be positive",
    ),
    (
        "tests/fixtures/regressions/table_clear_coordinate_order.pine",
        "table clear start coordinate cannot exceed end coordinate",
    ),
    (
        "tests/fixtures/regressions/table_clear_coordinate_bounds.pine",
        "table clear coordinate out of bounds `0,0` to `2,0`",
    ),
    (
        "tests/fixtures/regressions/table_merge_coordinate_order.pine",
        "table merge start coordinate cannot exceed end coordinate",
    ),
    (
        "tests/fixtures/regressions/table_merge_coordinate_bounds.pine",
        "table merge coordinate out of bounds `0,0` to `2,0`",
    ),
    (
        "tests/fixtures/regressions/table_merge_overlap.pine",
        "table merge range intersects existing merged cells",
    ),
    (
        "tests/fixtures/regressions/table_cell_coordinate_bounds.pine",
        "table cell coordinate out of bounds `2,0`",
    ),
    (
        "tests/fixtures/regressions/str_match_invalid_regex.pine",
        "str.match invalid regex",
    ),
    (
        "tests/fixtures/regressions/str_repeat_negative_count.pine",
        "str.repeat count cannot be negative: -1",
    ),
    (
        "tests/fixtures/regressions/str_repeat_result_limit.pine",
        "str.repeat result cannot exceed 40960 characters",
    ),
    (
        "tests/fixtures/regressions/str_replace_result_limit.pine",
        "str.replace result cannot exceed 40960 characters",
    ),
    (
        "tests/fixtures/regressions/str_replace_all_result_limit.pine",
        "str.replace_all result cannot exceed 40960 characters",
    ),
    (
        "tests/fixtures/regressions/str_tostring_result_limit.pine",
        "str.tostring result cannot exceed 40960 characters",
    ),
    (
        "tests/fixtures/regressions/str_substring_invalid_range.pine",
        "str.substring end_pos 1 is less than begin_pos 2",
    ),
    (
        "tests/fixtures/regressions/str_substring_begin_out_of_range.pine",
        "str.substring begin_pos 4 is outside string length 3",
    ),
    (
        "tests/fixtures/regressions/str_substring_negative_begin.pine",
        "str.substring begin_pos -1 is outside string length 3",
    ),
    (
        "tests/fixtures/regressions/str_substring_negative_end.pine",
        "str.substring end_pos -1 is less than begin_pos 0",
    ),
    (
        "tests/fixtures/regressions/str_format_unmatched_left_brace.pine",
        "str.format has unmatched `{`",
    ),
    (
        "tests/fixtures/regressions/str_format_unmatched_right_brace.pine",
        "str.format has unmatched `}`",
    ),
    (
        "tests/fixtures/regressions/str_format_timestamp_out_of_range.pine",
        "str.format timestamp is out of range: 9223372036854775807",
    ),
    (
        "tests/fixtures/regressions/str_format_time_placeholder_timestamp_out_of_range.pine",
        "str.format timestamp is out of range: 9223372036854775807",
    ),
    (
        "tests/fixtures/regressions/str_format_result_limit.pine",
        "str.format result cannot exceed 40960 characters",
    ),
    (
        "tests/fixtures/regressions/str_format_time_unsupported_timezone.pine",
        "str.format_time unsupported timezone `America/New_York`",
    ),
    (
        "tests/fixtures/regressions/str_format_time_timestamp_out_of_range.pine",
        "str.format_time timestamp is out of range: 9223372036854775807",
    ),
    (
        "tests/fixtures/regressions/str_format_time_result_limit.pine",
        "str.format_time result cannot exceed 40960 characters",
    ),
];
