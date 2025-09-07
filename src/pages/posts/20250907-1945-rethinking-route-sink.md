---
layout: ../../layouts/MarkdownPostLayout.astro
title: 'Rethinking `RouteSink`'
pubDate: 2025-09-07T19:45:41.554685740+03:00
description: ''
author: 'Alisa Feistel'
# image:
#    url: ''
#    alt: ''
tags: ["async", "data-structures", "project:ruchei"]
---

## The What

[`ruchei_route::RouteSink`](<https://docs.rs/ruchei-route/0.1.7/ruchei_route/trait.RouteSink.html>)

## The Why

We want a universal interface for "collection of connections" and "ZeroMQ-style router".

## Why is it *not* an extension of `Sink` (yet)

- The interfaces seem very different in usage.
- Default behaviour for `RouteSink` on readying and flushing all routes is to hang forever, if
   implemented over a set of connections.
- Implementing that behaviour otherwise is non-trivial.
- It's sometimes nice to have a blanket impelmentation of this for all `Sink`s.

## But, yes, it's kind of an extension?

There is a [blanket `impl`](<https://docs.rs/ruchei-route/0.1.7/ruchei_route/trait.RouteSink.html#impl-RouteSink%3CRoute,+Msg%3E-for-T>).

## So, what changed?

[`LinkedSlab`](<https://docs.rs/ruchei/0.0.96/src/ruchei/collections/linked_slab.rs.html>) made
thinking of complex wake patterns doable.

See [`ruchei::deal::slab`](<https://docs.rs/ruchei/0.0.96/ruchei/deal/slab/index.html>) and
[`ruchei::multicast::bufferless_slab`](<https://docs.rs/ruchei/0.0.96/ruchei/multicast/bufferless_slab/index.html>)
for examples of its use (note that multicast one has bugs, which were fixed since then, and were
relatively easy to fix).

## What is `Slab`

First, we need to talk about [`Slab`](<https://docs.rs/slab/0.4.11/slab/struct.Slab.html>). I
believe it to be one of the most powerful data structures in the Rust ecosystem. It effectively
gives you a pointer-like access without the associated memory unsafeties. You can implement
[all sorts](<https://github.com/parrrate/ruchei/tree/d511929158d68b09a039c8ccbd58cddbc2de1de2/ruchei-collections>)
of data structures on top of that.

> And it seems to perform well enough?

I'd say that but I don't have concrete enough benchmarks for that claim, so you can ignore it.

> What can I do with a `Slab`?

Things you'd expect from an allocator, plus a bit more.

- [allocate stuff](<https://docs.rs/slab/0.4.11/slab/struct.Slab.html#method.insert>) (you get a
   [`usize`](<https://doc.rust-lang.org/1.89.0/core/primitive.usize.html>) in return)
   - [know in advance where stuff goes](<https://docs.rs/slab/0.4.11/slab/struct.Slab.html#method.vacant_key>)
      (!!)
- [free stuff](<https://docs.rs/slab/0.4.11/slab/struct.Slab.html#method.remove>)
   - [free stuff but pretend it's sane not to expect a value to be there](<https://docs.rs/slab/0.4.11/slab/struct.Slab.html#method.try_remove>)
      (??)
- [dereference stuff](<https://docs.rs/slab/0.4.11/slab/struct.Slab.html#impl-Index%3Cusize%3E-for-Slab%3CT%3E>)
   - [attempt to dereference stuff](<https://docs.rs/slab/0.4.11/slab/struct.Slab.html#method.get>)
      (you might want to do that if you get an index from the user of your own data structure)
- panics if a pointer is dead rather than segfaulting
   - do note that you might accidentally reuse addresses (consider adding an extra check counter)

## Why is `Slab`

Not only do you get a memory safe (compared to pointers) implementation for structures you make,
but you also can expose `Slab`-like interface from the outside (you'll see that in `LinkedSlab`)
either by giving your users `usize` directly or
[some sort of a handle](<https://github.com/parrrate/ruchei/blob/d511929158d68b09a039c8ccbd58cddbc2de1de2/ruchei-collections/src/nodes.rs#L22-L26>).

## What is `LinkedSlab`

`LinkedSlab<T, N>`, in addition to base `Slab<T>` interface gives you `N` doubly-linked lists. If
you remove a node, it's automatically removed from all the doubly linked lists. This creates an
insertion-sorted set overlay on the items of the slab.

## Why is `LinkedSlab`

When working on [`ruchei`](<https://docs.rs/ruchei/0.0.96/ruchei/index.html>), we have a lot of
combinators which are collections of streams that:

- Need to be polled in a reasonable order.
- Need to have their state cleaned up after they're over.
- Have some sort of a flag associated with them or belong to a certain subset.

`LinkedSlab` solves all those.

## What Next

With better tools to implement multi-stream combinators, we believe that it should be reasonable to
assume all `RouteSink`s to come with a valid `poll_ready_any` and `poll_flush_all` implementation.
As such, we can just treat `poll_ready_route` and `poll_flush_route` as optimisation-only tools.

`RouteSink<Route, Msg>: Sink<(Route, Msg)>` seems like a reasonable way forward.

Another addition that I want to see is `DealSink<Route, Msg>: RouteSink<Route, Msg>`, which also
provides `poll_ready_some` to get the next `Route` that we can `start_send` to.

## Is `LinkedSlab` the best we can do?

I think not? [parrrate/ruchei#13](<https://github.com/parrrate/ruchei/issues/13>)

Having actual separate nodes and pointers gives you the following:

- `Waker` can reuse the same pointer.
   - This also includes the notification queue implementation.
- No `Unpin` requirement.
