---
name: rust-std-docs
description: Search and browse the Rust standard library documentation at doc.rust-lang.org. Use this skill when you need to look up types, traits, functions, macros, or modules from the Rust std library.
---

## What I do

Look up Rust standard library documentation by fetching pages from https://doc.rust-lang.org/stable/std/.

## When to use me

Use this skill when you need to:

- Look up the API of a standard library type, trait, function, or macro
- Check method signatures or available trait implementations
- Find which module a type lives in
- Read documentation examples for standard library items
- Verify behavior of std functions before using them in code

## How to search

The Rust standard library docs follow a predictable URL structure. Use the `webfetch` tool to fetch the relevant page.

### URL patterns

All URLs are rooted at `https://doc.rust-lang.org/stable/std/`.

| What you're looking for | URL pattern | Example |
|---|---|---|
| Module overview | `std/{module}/index.html` | `std/collections/index.html` |
| Struct | `std/{module}/struct.{Name}.html` | `std/collections/struct.HashMap.html` |
| Enum | `std/{module}/enum.{Name}.html` | `std/option/enum.Option.html` |
| Trait | `std/{module}/trait.{Name}.html` | `std/iter/trait.Iterator.html` |
| Function | `std/{module}/fn.{name}.html` | `std/mem/fn.replace.html` |
| Macro | `std/macro.{name}.html` | `std/macro.vec.html` |
| Primitive type | `std/primitive.{name}.html` | `std/primitive.str.html` |
| Constant | `std/{module}/constant.{NAME}.html` | `std/f64/constant.PI.html` |
| Type alias | `std/{module}/type.{Name}.html` | `std/io/type.Result.html` |

### Nested modules

For items in nested modules, add the module path:

- `std/sync/atomic/struct.AtomicBool.html`
- `std/collections/hash_map/struct.HashMap.html`
- `std/io/struct.BufReader.html`

### Common lookups

Here are the most frequently needed pages:

- **Collections:** `std/collections/index.html` — HashMap, BTreeMap, HashSet, VecDeque, etc.
- **String:** `std/string/struct.String.html`
- **Vec:** `std/vec/struct.Vec.html`
- **Option:** `std/option/enum.Option.html`
- **Result:** `std/result/enum.Result.html`
- **Iterator:** `std/iter/trait.Iterator.html`
- **Error trait:** `std/error/trait.Error.html`
- **fmt::Display:** `std/fmt/trait.Display.html`
- **Read/Write:** `std/io/trait.Read.html`, `std/io/trait.Write.html`
- **Path/PathBuf:** `std/path/struct.PathBuf.html`, `std/path/struct.Path.html`
- **File:** `std/fs/struct.File.html`
- **fs module:** `std/fs/index.html`
- **HashMap:** `std/collections/struct.HashMap.html`
- **Arc:** `std/sync/struct.Arc.html`
- **Mutex:** `std/sync/struct.Mutex.html`

### Search strategy

1. If you know the exact type/trait/function name, construct the URL directly using the patterns above.
2. If you're unsure which module it's in, start with the std index: `std/index.html`.
3. If looking for a specific module's contents, fetch `std/{module}/index.html`.
4. Fetch the page using `webfetch` and read the relevant sections.

### Example

To look up how `HashMap::entry` works:

```
webfetch("https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html")
```

Then search the returned content for the `entry` method.
