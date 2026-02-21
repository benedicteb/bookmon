---
name: rust-crates
description: Search crates.io for Rust packages and browse their documentation on docs.rs. Use this skill when you need to find crates, compare alternatives, check versions, or read crate API docs.
---

## What I do

Search for Rust crates on crates.io via its JSON API and browse crate documentation hosted on docs.rs.

## When to use me

Use this skill when you need to:

- Find a crate for a specific purpose (e.g. "what's a good crate for CSV parsing?")
- Check the latest version of a crate
- Read a crate's description, repository link, or download stats
- Browse a crate's API documentation (structs, traits, functions, modules)
- Compare crate alternatives by popularity
- Look up a crate's feature flags or dependencies

## Searching for crates

The crates.io website requires JavaScript and cannot be fetched directly. Use the **JSON API** instead.

### Search by keyword

```
webfetch("https://crates.io/api/v1/crates?q=SEARCH_TERM&per_page=10")
```

The response is JSON with a `crates` array. Each entry includes:

| Field | Description |
|---|---|
| `name` | Crate name |
| `description` | Short description |
| `max_version` | Latest published version |
| `downloads` | All-time download count |
| `recent_downloads` | Downloads in the last 90 days |
| `documentation` | Link to docs (usually docs.rs) |
| `repository` | Source code repository URL |
| `homepage` | Project homepage |
| `exact_match` | Whether the name exactly matches the query |

### Get a specific crate's info

```
webfetch("https://crates.io/api/v1/crates/CRATE_NAME")
```

Returns detailed info about a single crate. Note: this response can be very large because it includes all versions. Prefer the search endpoint with `?q=exact_name&per_page=1` for quick lookups.

### Sort and filter

You can add query parameters to refine search results:

| Parameter | Values | Example |
|---|---|---|
| `q` | Search query string | `q=json+parser` |
| `per_page` | Results per page (max 100) | `per_page=5` |
| `sort` | `relevance`, `downloads`, `recent-downloads`, `recent-updates`, `new` | `sort=downloads` |

Example — find the most downloaded async HTTP clients:

```
webfetch("https://crates.io/api/v1/crates?q=async+http+client&per_page=5&sort=downloads")
```

## Reading crate documentation

Crate API documentation is hosted on **docs.rs**. Use `webfetch` to read it.

### URL patterns

All URLs are rooted at `https://docs.rs/`.

| What you're looking for | URL pattern | Example |
|---|---|---|
| Crate root / overview | `docs.rs/{crate}/latest/{crate}/` | `docs.rs/serde/latest/serde/` |
| Module | `docs.rs/{crate}/latest/{crate}/{module}/` | `docs.rs/tokio/latest/tokio/sync/` |
| Struct | `docs.rs/{crate}/latest/{crate}/struct.{Name}.html` | `docs.rs/serde/latest/serde/struct.Deserializer.html` |
| Enum | `docs.rs/{crate}/latest/{crate}/enum.{Name}.html` | `docs.rs/clap/latest/clap/enum.ColorChoice.html` |
| Trait | `docs.rs/{crate}/latest/{crate}/trait.{Name}.html` | `docs.rs/serde/latest/serde/trait.Serialize.html` |
| Function | `docs.rs/{crate}/latest/{crate}/fn.{name}.html` | `docs.rs/tokio/latest/tokio/fn.spawn.html` |
| Macro | `docs.rs/{crate}/latest/{crate}/macro.{name}.html` | `docs.rs/tokio/latest/tokio/macro.select.html` |
| Attribute macro | `docs.rs/{crate}/latest/{crate}/attr.{name}.html` | `docs.rs/tokio/latest/tokio/attr.main.html` |
| Derive macro | `docs.rs/{crate}/latest/{crate}/derive.{Name}.html` | `docs.rs/serde/latest/serde/derive.Serialize.html` |
| Type alias | `docs.rs/{crate}/latest/{crate}/type.{Name}.html` | `docs.rs/reqwest/latest/reqwest/type.Result.html` |
| Feature flags | `docs.rs/crate/{crate}/latest/features` | `docs.rs/crate/tokio/latest/features` |

### Nested modules

For items in nested modules, add the module path:

- `docs.rs/tokio/latest/tokio/sync/mpsc/struct.Sender.html`
- `docs.rs/reqwest/latest/reqwest/header/struct.HeaderMap.html`

### Specific version

Replace `latest` with a version number:

- `docs.rs/serde/1.0.200/serde/trait.Serialize.html`

### Crate metadata page

For build info, dependencies, and feature flags:

```
webfetch("https://docs.rs/crate/CRATE_NAME/latest")
```

## Search strategy

1. **Finding a crate:** Search the crates.io API with keywords. Sort by `downloads` for established crates or `recent-downloads` for trending ones.
2. **Quick version check:** Search with `?q=crate_name&per_page=1` and read the `max_version` field.
3. **Reading docs:** Construct the docs.rs URL from the crate name and item path, then fetch it.
4. **Comparing alternatives:** Search with a broad term, fetch the top 5 results, and compare by downloads and descriptions.

## Example workflow

To find and evaluate a YAML parsing crate:

```
# 1. Search for crates
webfetch("https://crates.io/api/v1/crates?q=yaml&per_page=5&sort=downloads")

# 2. Read the top result's docs
webfetch("https://docs.rs/serde_yaml/latest/serde_yaml/")

# 3. Check feature flags
webfetch("https://docs.rs/crate/serde_yaml/latest/features")
```
