# deno-simple-module-loader

The [`deno_core`](https://crates.io/crates/deno_core) crate, part of the
[Deno](https://deno.land) Javascript/Typescript runtime, provides a high-level
interface over the V8 Javascript engine, with an event loop abstraction that
integrates with Rust `async` code. This includes facilities for handling ES
modules and dynamic imports.

However, `deno_core` does not by itself resolve and fetch modules. Instead it
provides a
[`ModuleLoader`](https://docs.rs/deno_core/latest/deno_core/trait.ModuleLoader.html)
trait, alongside with two implementations:
[`NoopModuleLoader`](https://docs.rs/deno_core/latest/deno_core/struct.NoopModuleLoader.html),
which fails when importing modules, and
[`FsModuleLoader`](https://docs.rs/deno_core/latest/deno_core/struct.FsModuleLoader.html),
which is limited to modules in the filesystem. `deno_core` does not provide an
implementation of `ModuleLoader` which performs network requests, and the
implementation in the Deno program is tightly coupled with Deno's network fetch
infrastructure and with its Typescript compilation pipeline, so it can't be
easily copied.

This crate provides a simple implementation of `ModuleLoader` that does load
modules from the network, built on top of `reqwest`.

Things it supports or aims to support:

- HTTP(S) imports
- Local filesystem imports
- Data URLs
- JSON modules (i.e. import assertions will just work) (pending)

Things it doesn't plan to support (but hey, file an issue if it bugs you):

- Any support at all for non-URL import specifiers (i.e. bare imports, like
  `"lodash"`), URL specifiers which are not supported in browsers (like Deno's
  `npm:` URL support), or any other mapping from the import specifier to the
  actual fetched URL (like import maps).
- Blob URLs
- Any support for transpiling modules (i.e. Typescript)
- Custom network, authentication or TLS settings. (Though if there's significant
  demand, I might reconsider.)
- Running with a futures executor other than `tokio`.

`deno-simple-module-loader` doesn't work with versions of `deno_core` lower than
0.133.0 (corresponding to Deno 1.21.2), since that version changed the
`ModuleSource` API to make it possible to load WebAssembly modules in the future
(https://github.com/denoland/deno/pull/14487).

This is a work in progress. No guarantees, yadda yadda.
