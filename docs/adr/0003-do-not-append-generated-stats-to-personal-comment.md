# Do not write generated stats into personal comments

Java's default `append-winrate-to-comment` merges engine winrate, score, and playouts into the same node comment that serializes as SGF `C`. Next already accepted `SGF-05`: generated information must not overwrite or serialize into the personal-comment field. Ticket 20 abandons that Java write rather than reopening `SGF-05` or adding a second generated-text channel beside deferred `ANA-08`.

Opening a Java file treats the entire existing `C` as personal comment; the user may delete leftover stats. Engine numbers stay visible in the analysis pane. Next-saved files do not carry a generated winrate sentence in `C`.

## Considered options

- **Equivalent write-into-`C`.** Other SGF viewers would keep seeing stats in comments. That reopens Accepted `SGF-05`.
- **Persist generated stats outside `C` now.** Overlaps deferred `ANA-08` structured analysis-header exchange.
