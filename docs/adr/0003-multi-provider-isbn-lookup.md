# 0003 - Multi-Provider ISBN Lookup

## Status

Accepted

## Context

When adding books, users should be able to look up book details by ISBN to avoid manual data entry. Different providers have different coverage — OpenLibrary has broad international coverage, while Bibsok specializes in Norwegian books.

## Decision

Use a `BookProvider` trait with a `ProviderManager` that tries multiple providers in sequence. The first provider to return a result wins. Errors from individual providers are logged but don't stop the lookup — the manager continues to the next provider. Currently OpenLibrary is tried first, then Bibsok.

Bibsok uses HTML scraping (via `scraper` crate) since it doesn't have a public JSON API. OpenLibrary uses its REST API.

## Consequences

- **Easier:** Better coverage across different book catalogues. New providers can be added by implementing the `BookProvider` trait. Graceful degradation if one provider is down.
- **Harder:** Bibsok provider is fragile due to HTML scraping — any site redesign will break it. Live API tests are brittle and marked with `#[ignore]`. Each provider has its own data format quirks that must be normalized into `BookLookupDTO`.
