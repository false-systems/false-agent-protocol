# false-agent-protocol

`false-agent-protocol` is the small wire contract connecting independently
built False Systems products:

- Teko owns specifications, work, obligations, gates, receipts, and closure.
- Toimija owns current repository points and repository capabilities.
- Kisko owns worker runs, actions, tool outcomes, transcripts, and run evidence.

Protocol `1.0` provides strict `Content-Length` framing, JSON-RPC envelopes,
`work.v1`, `worker.v1`, typed references, canonical digests, and compatibility
fixtures. It performs no storage, Git inspection, process execution, provider
calls, or product behaviour.

```bash
cargo build
cargo test
```
