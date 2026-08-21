# false-agent-protocol

`false-agent-protocol` is the small wire contract connecting independently
built False Systems products:

- Teko owns specifications, work, obligations, gates, receipts, and closure.
- Toimija owns current repository points and repository capabilities.
- A `worker.v1` producer owns its runs, actions, tool outcomes, transcripts,
  and run evidence; the protocol does not choose that producer.

Protocol `1.0` provides strict `Content-Length` framing, JSON-RPC envelopes,
`work.v1`, `worker.v1`, typed references, canonical digests, and compatibility
fixtures. It performs no storage, Git inspection, process execution, provider
calls, or product behaviour.

The crate is released independently. Products consume the tag, never another
product's source tree:

```toml
false-agent-protocol = { version = "0.1.1", git = "https://github.com/false-systems/false-agent-protocol.git", tag = "v0.1.1" }
```

```bash
cargo build
cargo test
```
