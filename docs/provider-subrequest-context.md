# Neutral provider subrequest admission

`PageExecutionContext` is a non-serialized capability created only by
`ProviderRegistry` for one page attempt. Its fields are private and it has no
public constructor. Context-aware adapters call `run_subrequest` with an
operation factory for each HTTP request, including every retried request. The
factory is invoked only after admission; constructing or spawning network work
before passing it to the context defeats that boundary and is unsupported.

The additive `SearchProvider::execute_page_with_context` method defaults to
admitting the existing `execute_page` once. This preserves legacy provider
implementations, but establishes only **page-level** admission for those
implementations. It does not prove spacing between hidden HTTP requests. A
multi-request adapter must override the method and route every request through
the context. This context is not a sandbox against a trusted adapter bypassing
the API or spawning detached work.

Factories must be short and nonblocking; returned futures must cooperate with
the asynchronous executor. Timeouts cannot preempt blocking synchronous work or
a future that never yields. Holding the admission guard through factory
invocation is not permission to perform blocking I/O inside that factory.

## Rate groups and deadlines

Trusted host code may use `register_with_rate_group` to share one rate limiter
between distinct provider IDs, such as two explicitly configured NCBI adapters.
Groups belong to one registry; different registries/processes are not jointly
throttled. Group selection is not part of `SearchRequest`, an agent query or a
remote caller's authority. Group IDs are bounded non-secret configuration labels.
Do not derive them from credentials, tenant identifiers or query text.

The effective group floor is the monotonic maximum of its member manifest and
admitted execution-request intervals for the registry lifetime. Denied, timed-out
before admission, and cache-only requests do not raise it. Later weaker settings
cannot reduce that floor; reconstructing a registry requires trusted host
configuration. Separate groups and ordinary ungrouped registrations remain
independent. Sharing a limiter does not merge cache namespaces or provider IDs.

The limiter holds its scheduling mutex through waiting and synchronous operation
factory invocation. It records the time after that invocation, not a future
reservation. A delayed wakeup therefore cannot
release queued operations in a burst. A cancelled waiter does not leave a
future reservation. Admission time is the operation-factory invocation boundary,
not proof of when remote packets reached a provider.

Mutex waits, rate sleeps and operation futures all fall within the page-attempt
deadline and the remaining overall execution deadline. Unlike the legacy
implementation, rate waits now count against the page timeout as well as the
overall timeout. Dropping the execution future cancels pending admission and
owned futures; it cannot recall an already-dispatched remote request.

## Finite work and retries

At most 32 admitted subrequests may execute for a page, cumulatively across that
page's retries. Exhaustion returns `BudgetExceeded` with
`kind: subrequests_per_page`; it does not silently return a partial page. Retry
counts remain controlled by the existing execution policy. A page-level retry
may repeat its earlier successful subrequests, each of which must be admitted
again. No nested connector retry engine is authorized by this API.

If a provider's `Retry-After` exceeds the configured retry-delay maximum, return
the original retryable error instead of retrying early. If an otherwise allowed
delay cannot fit in the overall deadline, the existing total-budget failure
applies. This is a deliberate conservative correction to the previous behavior
that capped a provider-requested delay.

`max_response_bytes` remains a per-response transport limit. The context does
not read bodies or reinterpret that setting as a cumulative page-byte limit.
Transport adapters retain bounded body accumulation and parser-specific limits.
Cache hits consume no subrequest admission. Public receipts and serialized
schemas are unchanged; existing page counts are not HTTP-subrequest counts.

## Evidence boundary

Paused-clock tests use synthetic operation futures with no DNS, sockets or
provider access. They establish only tested local admission, cancellation,
deadline, retry and cache behavior. No existing two-call adapter is promoted to
per-HTTP throttling until it is migrated and tested. This change does not grant
live-call authority, current provider-policy approval, cross-process throttling,
downstream compatibility, authenticated remote capability or release promotion.
