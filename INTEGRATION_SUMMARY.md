# Theater MCP Server Integration

## Overview

This document summarizes our approach to integrating the Theater MCP Server with the Theater project using direct type references instead of creating duplicate type definitions.

## Problem Statement

The original implementation of the Theater MCP Server defined its own versions of types that already existed in the Theater project. This led to:

1. Type duplication between projects
2. Manual conversion between similar types
3. Potential for inconsistencies when Theater types evolve
4. Extra boilerplate code for serialization/deserialization

## Solution

Our solution is to refactor the Theater MCP Server to directly use the types from the Theater project, eliminating duplication and ensuring type consistency between the projects.

## Implementation Details

### 1. Theater Client

We've created a new implementation of the `TheaterClient` that:

- Uses `TheaterId` from the Theater project instead of string IDs
- Uses `ManagementCommand` and `ManagementResponse` directly
- Properly handles pattern matching on response types
- Eliminates string-based conversions and manual JSON parsing

### 2. Resource Implementation

We've updated the resource implementations to:

- Use Theater's native types for actors and states
- Handle conversion between URI paths and Theater IDs
- Properly serialize/deserialize state data

### 3. Tool Implementation

We've updated the tool implementations to:

- Use Theater's native types for actor operations
- Properly register tools with the MCP server
- Handle input/output conversion where needed

## Compatibility Strategy

To maintain backward compatibility, we've:

1. Kept both implementations (original and new)
2. Added type conversion utilities
3. Made imports explicit and clear
4. Created tests to ensure both implementations produce equivalent results

## Benefits

1. **Type Safety**: Using Theater's types directly ensures type-safe operations
2. **Consistency**: Changes to Theater's types will be automatically reflected
3. **Maintainability**: Easier to keep the projects in sync as Theater evolves
4. **Reduced Duplication**: Eliminates redundant type definitions
5. **Better Error Handling**: More specific error types and better pattern matching

## Next Steps

Please refer to the `ADOPTION_PLAN.md` document for details on our phased approach to adopting the new implementation.

## Implementation Guidelines

When working with Theater types in the MCP server:

1. Import types directly from the Theater crate
2. Use `TheaterId` instead of string IDs when possible
3. Use the provided extension trait (`TheaterIdExt`) for conversions
4. Handle potential errors properly when converting between types
5. Use pattern matching on Rust enums rather than relying on JSON field access

## Testing

We've added tests to verify that both implementations produce equivalent results. These tests:

1. Compare API compatibility
2. Ensure equivalent behavior
3. Verify error handling consistency

## Conclusion

This integration approach ensures that the Theater MCP Server and Theater project remain tightly synchronized, reducing maintenance overhead and improving type safety.
