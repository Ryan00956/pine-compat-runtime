# Diagnostic Codes

Diagnostic codes are part of the public developer experience. Messages can be
improved over time, but codes should remain stable once published.

## Lexing

- `E_LEX_CHAR`: unexpected character.
- `E_LEX_COLOR`: invalid color literal.
- `E_LEX_FLOAT`: invalid float literal.
- `E_LEX_INT`: invalid integer literal.
- `E_LEX_STRING`: unterminated or invalid string literal.
- `E_LEX_VERSION`: invalid version directive.

## Parsing

- `E_PARSE_DECL`: invalid declaration.
- `E_PARSE_EXPECTED`: expected token was missing.
- `E_PARSE_EXPR`: expected expression.
- `E_PARSE_EXPORT`: invalid export declaration.
- `E_PARSE_IMPORT`: invalid import declaration.
- `E_PARSE_LIBRARY`: invalid library declaration.
- `E_PARSE_NAME`: invalid qualified name.
- `E_PARSE_SWITCH`: invalid switch expression.
- `E_PARSE_SWITCH_BLOCK`: statement-block switch arms are not supported.
- `E_PARSE_TYPE`: invalid user-defined type declaration.
- `E_PARSE_WHILE_EXPR`: while expressions are not supported.

## Semantic Analysis

- `E_ASSIGN_TYPE`: reassignment type mismatch.
- `E_BRANCH_TYPE`: ternary branch type mismatch.
- `E_CALL_ARG_NAME`: unknown named argument.
- `E_CALL_ARG_TYPE`: argument type does not satisfy the built-in signature.
- `E_CALL_ARG_VALUE`: argument type is valid, but the value is outside the
  supported range.
- `E_CALL_ARITY`: wrong number of call arguments.
- `E_CALL_TARGET`: invalid function call target.
- `E_CONDITION_TYPE`: condition expression is not bool.
- `E_FUNCTION_ARG_DUPLICATE`: user-defined function argument was provided more
  than once.
- `E_FUNCTION_ARG_NAME`: unknown user-defined function named argument.
- `E_FUNCTION_ARG_ORDER`: positional argument followed a named argument in a
  user-defined function call.
- `E_FUNCTION_ARITY`: wrong number of user-defined function arguments.
- `E_FUNCTION_DUPLICATE`: user-defined function name was declared more than
  once.
- `E_FUNCTION_NAME`: user-defined function name conflicts with an existing
  symbol or built-in.
- `E_FUNCTION_PARAM`: user-defined function parameter list is invalid.
- `E_FUNCTION_RETURN`: user-defined function block does not end with an
  expression.
- `E_IMPORT_CYCLE`: import dependency graph contains a cycle.
- `E_IMPORT_DUPLICATE_ALIAS`: root imports reuse the same alias.
- `E_IMPORT_DUPLICATE_EXPORT`: a library exports the same name more than once.
- `E_IMPORT_INVALID_LIBRARY`: host-provided library source does not contain
  exactly one library declaration.
- `E_IMPORT_MISSING_LIBRARY`: an import key has no host-provided source.
- `E_IMPORT_PRIVATE_SYMBOL`: root code accessed a non-exported library symbol.
- `E_IMPORT_UNKNOWN_EXPORT`: root code accessed an export that the library does
  not declare.
- `E_IMPORT_UNSUPPORTED_EXECUTION`: imported symbol resolution succeeded, but
  executable imports are not enabled in the current subset.
- `E_RECURSIVE_FUNCTION`: recursive user-defined function call is not supported.
- `E_OPERATOR_TYPE`: operator does not accept the operand types.
- `E_TUPLE_ARITY`: tuple assignment target count does not match value count.
- `E_TUPLE_TYPE`: tuple assignment value is not a tuple.
- `E_UNKNOWN_SYMBOL`: symbol cannot be resolved.
- `E_UNSUPPORTED_FEATURE`: recognized feature is outside the current supported
  subset.
