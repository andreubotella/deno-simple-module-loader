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

Things it aims to support (doesn't yet, because there is still no code at all):

- HTTP(S) imports
- Local filesystem imports
- Data URLs
- JSON modules (i.e. import assertions will just work)

Things it doesn't plan to support (but hey, file an issue if it bugs you):

- Any support at all for "bare" import specifiers (i.e. `"lodash"`, which aren't
  absolute or relative URLs). This means no support for import maps, or for
  Node.js's module resolution algorithm.
- Blob URLs
- Any support for transpiling modules (i.e. Typescript)
- Custom network, authentication or TLS settings. (Though if there's significant
  demand, I might reconsider.)
- Running with a futures executor other than `tokio`.

This is a work in progress. No guarantees, yadda yadda.
