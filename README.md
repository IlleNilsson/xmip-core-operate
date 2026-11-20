# xmip-core-operate

What every operator surface calls, implemented once (ADR-0058, amendment
2026-11-16): the C ABI's operating exports, `include/xmip_operate.h` in
[xmip-core-abi](https://github.com/IlleNilsson/xmip-core-abi), over
[xmip-core-runtime](https://github.com/IlleNilsson/xmip-core-runtime), in
the native library the command line, the PowerShell module, the GUIs, the
designer's language server and the C, C++, Java and Python Event bindings
load — `xmip_core_operate.dll`, `libxmip_core_operate.so` or
`libxmip_core_operate.dylib`; and on .NET the Operate model, `Xmip.Surface`
and its Relay ([.NET](#net)). The header and its bindings are abi's: a
definition has no verb (ADR-0058, clause 1). The runtime never depends on
this repository; it keeps its own end of the C ABI, the loader that opens a
module's library (ADR-0057).

## Rust

| Item | What it is |
| --- | --- |
| `ffi` | Every export of `xmip_operate.h`, and the one folder of the crate that may hold unsafe code (ADR-0050, amendment 2026-11-16): the operator's table, starting and validating a node, the rules a surface calls instead of keeping its own, a publication read, a program's audit record, `cluster.xmip` read, edited and sliced, Events, the technology catalogue, System Processes, a node's Subscriptions, Dead Message Queue and failed Messages, a node reached and served at its operate listener, the security roles, a person's certificate, a host's secure listener, a node's certificate and the cluster's authority, the database scripts and every Report. Each reads what a surface handed over, calls the crate that owns the rule and writes the answer where it was told; none decides. `resident` pins the library as Windows loads it, and `build.rs` links it `-z nodelete` elsewhere, so the threads it starts outlive an unload (section 1) |
| `operator` | The operator a surface's table stands for: the snapshot it answers from, what it last handed out, and what this process published — `start_published`, a node's plan, the first three startup phases, which `xmip_start_v1` publishes |
| `design` | The designer's view of `cluster.xmip`, artifact by artifact (`views`), an Xmip Application's routes (`routes`), a filter's rows and its text (`filter`), every edit (`edit`, `view_edit`), what an editor suggests, looks up and renames by (`keys`, `shape`, `written`, `references`, `offered`), and each node's slice, signed by the cluster's authority: the JSON section 10 hands the language server and the Operation Desktop, in memory only (ADR-0064; ADR-0031 clause 2). What a document means — its reading, binding and slicing — is xmip-core-configure's |
| `wire` | The header's integers for the crates' enums, in both directions, converted here and nowhere else |
| `surface` | What every screen, command and cmdlet reads, once in Rust (ADR-0052 clause 1; ADR-0014, amendment 2026-11-17): the `OperatorSurface` trait, over a published snapshot (`snapshot`, followed by the runtime's directory watch) or a node's operate listener here or on another machine (`listener`, over `runtime::plane::Remote`), chosen by the line over the host's document (`choice`, read by `configure::document`, in the words `choice::table` holds, which also writes the `cli.toml` `xmip-service --install` leaves beside the programs, stating the node it installed); the scope tree a publication is (`scope_index`, `scope_tree`), its rows (`scope_item`), what an argument selects (`scope_selection`), the six figures (`figures`), the English (`english`), the run and a node's declaration (`run_header`, `node_capability`), a validation's verdict (`configuration_verdict`), the three noun queries and the one order they sort by (`subscription_query`, `event_subscription_query`, `dead_message_query`, `list_order`), the Messages that failed (`failed_message_list`) and what came of an act (`acted`). The command line calls it Rust to Rust |
| `audit_read`, `authority`, `certificate`, `database`, `module_probe` | What an export answers by, outside the boundary: the audit read back by its one reader, the cluster's authority's acts, a person's request signed, the database scripts written, and a probe's answer as its JSON — each the one the command line calls and the export forwards to |

**The designer** (`design`) is a view of the cluster's file's own sections,
artifact by artifact: `views` answers one view per kind (`kinds`) —
Cluster, Node, Party, Module, Receive Port, Receive Location, Send Port,
Send Location, Send Port Group, Prepare, Promote, Demote, Route,
Transformation, Work Process — with each entry's values (`field`), what may
be set on it (`offered`, the runtime handing in its `[tuning]` declaration
and the role words through `Views::offering`), an Application's routes as a
graph (`routes`), a filter's rows and groups as a view of its compiled
expression (`filter`; the filter itself is one line of Xmip's expression
language, `xmip-core-path`'s `expression`, compiled as the Application is
read, ADR-0066), and every compiled unit the file's modules declare. `edit`
makes an Application's edits to the text in place, and `view_edit` the
designer's — `add-element` adding one element to an array of unnamed
tables, a Party's identity — writing `start = true` on each Receive
Location, Send Port and Send Location it creates, and refusing one that
would leave a node or an Application that read unable to. A cluster's
shared top declares no roles: the editor's keys (`keys`) offer `roles`
under a node's own table alone.

**Two builds** (ADR-0014, amendment 2026-10-14), as the runtime's. `node`,
the default, is every export over a node's runtime. `remote` alone
(`--no-default-features --features remote`) is the phone's, over the
runtime's `plane::remote`: the rules, a publication read, a program's audit
record, the security roles, the operate listener's client and a person's
request; `Build-XmipService -Remote` builds it for this machine, Android and
iOS.

`cargo build` leaves the library in `target/debug`, and `cargo test` runs
the exports' and the designer's tests over the runtime's fixtures (its
`test-support` feature).

The phone's library is `cargo build --release --target aarch64-linux-android --no-default-features --features remote --lib`, with the Android NDK's API-level-26 clang as linker, `CC` and its `llvm-ar` as `AR` for the target, set on the command line or in the environment and never in this repository. It exports the rules, the publication reader, the audit record, the security roles, a person's certificate request and the `xmip_remote_*` client, and needs no CMake: `aws-lc-sys` builds with the NDK's clang. A .NET surface's `RuntimeRules.Load` asks for every export of the full library, so as it stands it refuses the remote one: section 17's five serve exports, sections 10, 12 and 13, and the per-process lists and acts of sections 11, 14, 15 and 16 are not in it.

The library stays in the process once loaded (`xmip_operate.h` section 1): its own threads, the audit keeper and an Event listener, outlive every call, so on Windows it pins itself as it loads (`ffi/resident.rs`) and on ELF systems `build.rs` links it `-z nodelete`; `FreeLibrary` and `dlclose` release a reference and never unmap it (ADR-0027, amendment 2026-09-30). Every enum that crosses `xmip_operate.h` as an `int` is converted in one file, `src/wire.rs`, and every string is lent and read in one, `ffi/text.rs`.

### What each export does

Every export runs its body within `abi::guarded`, the boundary's one panic guard, so a panic answers `XMIP_E_PANIC` and never aborts the surface's process (specification section 6). `xmip_start_v1` is the runtime's `start::start`, the first three phases alone, its plan published for the operator's table (`operator`): it validates and plans a node against the technologies the runtime carries and says in every record that nothing runs, since a surface's process links no technology (whether it may run a node at all is open problem 20); `xmip_validate_v1`, the runtime's `start::validate`, reads a node's configuration, its bindings with the Application sections they bind, or a cluster's `cluster.xmip` node by node. Section 7 — the rules a surface calls instead of keeping its own — is `src/ffi/rule.rs` and `src/ffi/rule/node.rs`, forwarding each rule to the crate that owns it (`node::Scope`, its containment, parts, node and stage, `node::wildcard`, the one wildcard, as `xmip_scope_matches_v1`, `observe::Health`, `observe::Standing`, `observe::Counted`, `observe::capability`, `node::Stage`, `node::Capability`), one call per export and no rule of its own; section 8, `src/ffi/publication.rs`, reads a publication by `observe::Publication`, `src/ffi/from_to.rs` says where the work at a scope of it came from and went by `observe::topology::from_to` (`xmip_publication_from_to_v1`; ADR-0052, amendment 2026-11-11), `src/ffi/layout.rs` where each thing it draws beneath a focus stands on the map by `observe::topology::layout`, by role and never by traffic or a name (`xmip_publication_layout_v1`; ADR-0052 and ADR-0056, amendments 2026-11-13), and `src/ffi/topology.rs` says what a topology kind, origin, pattern or relation is called (ADR-0027 and ADR-0052, amendments); section 7's `xmip_role_purpose_v1` says what a node's roles are for, by `node::NodeRole::purpose`. Section 9, `src/ffi/audit.rs`, forwards `xmip_audit_v1` to `xmip-core-audit`'s `ProgramAudit` (ADR-0062), and `xmip_audit_read_v1` to its reader and `AuditQuery`, answering the header's JSON (ADR-0062, amendment 2026-09-29) — with each writer's audit chain walked over the file as it is now where the query asks `verify` (ADR-0070 clause 5); `ffi/rule.rs` forwards whether what a hidden run made is shown, `observe::run::shown`, as `xmip_run_shown_v1` (ADR-0028, amendment 2026-09-30). Section 10 is the designer's, over the cluster's one `cluster.xmip`: `src/design.rs` reads the JSON a surface hands over into the designer's types and `src/ffi/design.rs` holds `xmip_cluster_views_v1` (the file as one view per artifact kind, an Xmip Application's routes among them), `xmip_filter_structure_v1`, `xmip_filter_text_v1`, `xmip_cluster_edit_v1` and `xmip_cluster_slices_v1`, each node's document by `configure::slice`, a node's own document refused (ADR-0064, amendment 2026-10-03; ADR-0031, amendment 2026-10-05); the views say `document`, `cluster` or `node`, by `configure::document_kind`, so the Operation Desktop offers no node's file for editing. Section 11 is Events (ADR-0065): `src/ffi/event.rs` holds the six exports, each a thin forwarder into `xmip-core-cluster`'s process hub (`delivery::hub::Hub::process`, taken with `xmip-core-cluster`'s `delivery` under the `node` feature), and `src/ffi/event/crossing.rs` reads and writes the header's Event and filter and says who subscribes: no argument names a Party, and the subscriber is the account the operating system says the process runs as, proven by the operate plane's gate and resolved to the one Party whose `peer-credentials` identity names it by the runtime's one rule, `plane::caller::subscriber` (the Essentials' nodes call it too), through the Parties every node serving in the process lends it as it starts (the runtime's `parties::Parties::serve`, `Parties::serving`), a caller no Party holds, or more than one holds — each named, whichever node lent first (`Parties::holding`) — refused `XMIP_E_AUTH` with the sentence and audited in the program's audit with who asked and what for (`xmip-core-cluster`'s `delivery::subscriber::refused`; ADR-0019, amendment 2026-11-11); `src/ffi/event/subscription.rs` holds what an operator lists and does of Event subscriptions — `xmip_event_subscriptions_v1` (the hub's standing, as JSON), `xmip_event_subscription_act_v1` (pause, resume or remove) and `xmip_publication_event_subscriptions_v1` (what a read publication carries) — each forwarding to the cluster's hub (ADR-0065, amendments 2026-09-29 and 2026-09-30). Section 12, `src/ffi/catalogue.rs`, answers `xmip_technology_catalogue_v1` from the runtime's `catalogue`: the technologies a node in this process loaded, each carried with its settings declaration as it loaded (ADR-0064, amendment 2026-09-26). Section 13, `src/ffi/process.rs`, forwards `xmip_process_declare_v1`, `xmip_process_declarations_v1` and `xmip_process_undeclare_v1` to the runtime's `declared` (ADR-0053): the one writer of a System Process's declaration file, its directory (`XMIP_PROCESS_DIRECTORY`, else `xmip/process` under the system's temporary directory), `declared::standing`, every declaration in a directory, each read by `node::Standing::read`, and the one remover, `declared::undeclare`, which takes away only a file named `xmip-*.toml` and is what a dropped `declared::Declared` calls, what a caller across the boundary handed its file calls where its process ends, and what a listing calls for a process it found gone; what a declaration says and its text stay `xmip-core-node`'s, which opens no file (ADR-0058, amendment 2026-11-13). `xmip-service` and the Essentials' roll, cluster and nodes declare through it too. Section 14 is a node's Subscriptions (ADR-0013, amendment 2026-09-30): `src/ffi/subscription.rs` holds `xmip_subscriptions_v1` (every node's in this process, as JSON, each with the Send Ports it reaches, and none of a node that routes by none, `Pickup::routing_none`) and `xmip_publication_subscriptions_v1`, each forwarding to the pickup; a pause or resume, like a Replay, a Retry and a Dismiss, is asked at a node's door through section 17's `xmip_remote_act_v1`, written to Xmip Storage and applied by the node it names (the runtime's `act_watch`, `applied`; ADR-0014, amendment 2026-11-13), and a snapshot is never acted on. Section 25, `src/ffi/report.rs`, answers `xmip_report_v1` from the runtime's `report` (ADR-0072, amendment 2026-11-10): `list`, every Report in `xmip-core-report`'s one list, each saying whether it presents data that already exists or what a Quest probes first, with each Quest's last probe read back from the audit through `AuditQuery` (action `quest.probe`, the newest finished record of each Report), or the one Report `report` names, a name that is none refused in `xmip-core-report`'s one sentence, and every Report listed all the same where the audit cannot be read, the answer saying why (`unread`); and `open`, a Report's Quest probing a node's configuration given by path — a node's own, or a cluster's `cluster.xmip` sliced for the node named by `configure::slice`, told apart by `configure::document_kind` — composed into an `Installation` whose every Location's wire is undeclared (ADR-0072 Open 1) and whose data domains are kept where `storage::keeping` says, the one decision `storage::plan` starts a node by, each table's storage read by `xmip-core-persist`'s `database::technology`, so a word that names no technology is refused as a node starting is; a Report that cannot be opened is refused by `report::answer::probe_of` before the file is read. Each probe whose answer the caller has room for is recorded through `ProgramAudit` under the program the surface names, with how long it took, as "The Regulation Report's Quest probed node X for GDPR in 3 ms and changed nothing."; a call asking its length alone is answered and not recorded. `report::carry` adds a module's Report to the list. Section 26, `src/ffi/module_probe.rs`, answers `xmip_module_probe_v1` from the runtime's loader (`runtime::ffi::probed`, behind the runtime's `loader` feature): the library opened as a node opens one, its descriptor judged by `abi::validate_module_abi`, its lines heard through the host table's log, and nothing of it left loaded (ADR-0012, amendment 2026-11-17). Section 27, `src/ffi/document.rs` over `src/document.rs`, answers `xmip_document_read_v1`, `xmip_document_write_v1` and `xmip_document_set_v1` from `configure::document`, the runtime's one reader, turning its tables into JSON and back and nothing else — in a remote build too, so a phone reads its own document (ADR-0031, amendment 2026-11-17); it answers through `src/ffi/text.rs`'s `answer`, the text shape section 10 takes too. Section 28 is what every surface shares, once, in `surface` (ADR-0052, amendment 2026-11-18): `src/ffi/scope_index.rs` (28.1) lends a publication's scope tree as a handle — built, drilled, measured, selected and labelled by `surface::scope_index`, its records handed as places in the publication, never copied; `src/ffi/surface.rs` (28.2) opens the surface `surface::choice` decides for a host's document — a snapshot, a node here or on another machine — and reads it, its reading's publication and tree handed only where it changed and nothing made where it did not, follows it (`xmip_surface_wait_v1`) and asks its acts; `src/ffi/words.rs` (28.3) says the sentences, the list queries, what a status means and the boundary's limits; and `src/ffi/written.rs`, a node's build only, writes what is signed and upgraded to a file (`xmip_operator_sign_file_v1`, `xmip_authority_name_v1`, `xmip_database_write_v1`). The command line calls `surface` Rust to Rust; .NET calls these exports.

## Toolchain

`rust-toolchain.toml` pins the toolchain for the whole estate. rustup reads it
automatically and installs what is missing. Do not change it here — raising it
is one deliberate change across every repository.

## Shared governance

Repository-specific licensing remains explicit in [LICENSE](LICENSE).
Contribution, security, support, issue and pull-request defaults are inherited
from [IlleNilsson/.github](https://github.com/IlleNilsson/.github) when they are
not overridden locally.

## Verification

The included workflow is manual-only and calls the versioned shared workflow at
`IlleNilsson/.github@v1`. It does not run on pushes, pull requests or a
schedule.

The ordered stages are formatting, semantic analysis, linting, compilation and
linking, and test execution. Packaging and publishing are not configured.

## .NET

`dotnet/` is the .NET half: `Xmip.Operate`, the Operate model every
operator surface calls the runtime through, and `Xmip.Surface` and
`Xmip.Surface.Relay` over it, each with its tests. All of it calls through
the declarations `Xmip.Abi` binds in
[xmip-core-abi](https://github.com/IlleNilsson/xmip-core-abi): the header is
that repository's and so is its binding; what operates is this one's
(ADR-0058, clause 1; ADR-0052, amendment 2026-11-16). The folder carries its
own `global.json`, `nuget.config` and `Directory.Build.props`, as every
.NET repository does, and the cli, the PowerShell module and the GUI
reference `Xmip.Surface` by project path inside the composed estate.

### The Operate model

`dotnet/Xmip.Operate` compiles under C# 15's updated memory safety rules, a
preview, as the binding does: no type is `unsafe`; an `unsafe` block stands
only around a dereference, a call through a function pointer or a framework
call the runtime marks unsafe (`GCHandle.Free`, an uninitialized
`stackalloc`); and a member whose safety rests on its caller is marked
`unsafe` itself. Xmip's own fixed answers are unions, so a switch over one
names every case: `SecureSurfaceAccepted` (`Accepted`, `Refused`,
`Stopped`, `Failed`) and `AuditOutcome` (`Kept`, `Refused`) here,
`NodeDelivery.Standing` (`Shipped`, `NotShipped`, `Refused`) in
`Xmip.Surface`, whose `ClusterEdit` is a `closed` hierarchy.
`Xmip.Surface` and `Xmip.Surface.Relay` build with runtime async on
Windows. `NativeRuntime` loads the runtime's library, the one way every
caller loads it: on a desktop a copy, on Android by its name, on iOS from the
main program (ADR-0014, amendment 2026-10-14).

`RuntimeRules` is section 7 bound once: scope containment, a scope's
parts and the node and stage it is on, the stage words and the facts of a stage, the role words, a declaration's parse and the stages a role serves,
what a node declaring some roles is for (`Purpose`, `node::NodeRole::purpose`:
each role with its meaning, a line each, or the refusal of a word that is no
role — the one purpose every surface shows a node by; ADR-0056, amendment
2026-11-13), a
node's published capability and a run's entry for it, a mood's word, color
name and rollup, a counted kind's word and the worst-first order, each written
once in the crate that owns it (`observe`, `node`) and forwarded by the
runtime's library, so no .NET surface writes a rule again; and
`PublicationReader`, section 8, which hands a publication's text to the
runtime's one reader and brings back a `Publication` — its records, counts,
topology (its values the header's, `Xmip.Abi`'s `Topology.cs`) and run (ADR-0052, amendments
2026-09-24), and what each topology kind, origin, pattern and relation is called
(`Words`, a `TopologyWord` of the word a publication writes and the name a
person reads, `observe::topology`'s; ADR-0052, amendment 2026-09-25). Since
2026-11-13 a `TopologyNode` says the `Roles` a node of the cluster declares
and a Storage node its `Place` in `[storage] nodes`, a `CommunicationLink`
its `Relation` — how its two ends are related by their roles, never their
names (`TopologyRelation`) — and the snapshot carries `Places`, where each
thing beneath each focus stands on the map as the runtime lays it out
(`xmip_publication_layout_v1`, asked for every focus while the handle is
open; `PlacesAt` answers one, the empty string the top): a `TopologyPlace`'s
`X` and `Y` in the unit square, its `TopologyBand` and its `TopologyColumn`
(ADR-0052 and ADR-0056, amendments 2026-11-13). The header's `XmipTopologyNode`,
`XmipTopologyLink` and `XmipTopologyPlace` are `Xmip.Abi`'s, each held to the Rust
binding's size by its test. And `RuntimeAudit`, section 9, a program's audit record handed
to `xmip-core-audit` through the runtime's library, with `AuditPhase`,
`AuditSeverity` and `AuditKept` as the header defines them (ADR-0062), and
`Read`, the records read back by the capability's one reader and query —
`AuditRead`, its `AuditEntry` page and its `AuditGroup`s one step down the
drill (ADR-0062, amendment 2026-09-29), and, where the read asks `verify`,
its `AuditChain`s, each writer's audit chain walked whole, and its
`AuditFragment`s, each part of the file no write finished, said apart from
the chains (ADR-0070 clause 5). Section 11 — Events, subscribed (ADR-0065) — is bound once as
`RuntimeRules.Events` (`RuntimeEvents`): `Subscribe` returns a disposable
`EventSubscription` drained with `Next(timeout, max)`, `Listen` calls a
handler on the runtime's listener thread, and `Publish` hands an Event to
every matching subscription in the process; `AuthorizeBy` hands the hub
the program's policy of who may subscribe, asked of each attempt as an
`EventAuthorization` and answering allow, deny or no opinion — being in the
process admits nobody (ADR-0065, amendment 2026-09-26) — and every `EventDelivery` carries `Unheard`, the members of
the cluster not heard now as `UnheardRecord`s, with `UnheardChanged`, a
drain waking for that alone (amendment 2026-10-02); `Unheard()` asks the
hub; `EventRecord`, `EventFilter`,
`EventAction` and `EventOutcome` are the header's. What an operator lists and
does of Event subscriptions is `RuntimeRules.EventSubscriptions`
(`RuntimeEventSubscriptions`): `Standing`, the hub's `EventSubscriptionList`
of `EventSubscriptionRecord`s, and `Act`, pause, resume or remove one; a
`Publication` carries its own `EventSubscriptions`, each list with the
members its nodes do not hear (`EventSubscriptionList.Unheard`) (ADR-0065, amendments
2026-09-29 and 2026-09-30). Section 14 — a node's Subscriptions (ADR-0013,
amendment 2026-09-30) — is `RuntimeRules.Subscriptions`
(`RuntimeSubscriptions`): `Standing`, the `SubscriptionList` of
`SubscriptionRecord`s of every node running in the process; a `Publication`
carries its own `Subscriptions`. A pause or resume — there is no remove —
a Replay, a Retry and a Dismiss are each asked at a node's door,
`RemoteNode.Act`, and reach their node through Xmip Storage (ADR-0014,
amendment 2026-11-13); a publication is never acted on.
Section 15 — a node's Dead Message Queue (ADR-0052, amendment 2026-10-01) —
is `RuntimeRules.DeadMessages` (`RuntimeDeadMessages`): `Standing`, the
`DeadMessageList` of `DeadMessageRecord`s every node running in the process
keeps, each with its gate verdicts, promoted properties and declines as name
and value pairs; a `Publication` carries its own `DeadMessages`. Section 16
— a Message that failed (runtime-model.md section 13; ADR-0013) — is
`RuntimeRules.Messages` (`RuntimeMessages`): `Failed`, the `FailedMessageList`
of every Message that failed at the Send Ports of the nodes running in the
process, each Port's `FailedMessagePort` with its count, the place its next
page reads from and its `FailedMessageRecord`s, read from Xmip Storage a page
at a time; a `Publication` carries its own `FailedMessages`, each Port's
count and oldest hundred. `Xmip.Surface` lists them through
`IOperatorSurface.FailedMessages` at a cluster, a node or one Send Port's
scope (`MessageOperation.PortAt`, `MessageOperation.Within`). A Port's page
may be empty with a next place, where the stretch the node read held no
failed Message: the place, not the Messages, says whether more remain. A
list is an answer (`FailedMessageList.Listed`) even when it lists none;
`FailedMessageList.Unlisted` is a surface that cannot list them — no node
reached, no runtime loaded, no publication read yet — and
`FailedMessageList.Unanswered` one that asked and was not answered, its
`Failure` the sentence, opening `FAILED:`, in the runtime's words: where Xmip
Storage does not answer, a surface says so through `MessageOperation.Read`
rather than throwing. The evidence a Send
Port publishes is read in one place each: the last Message that ever failed
there by `MessageOperation.FailedIn`, how many wait in its queue now by
`MessageOperation.FailingIn`.
Section 17 — a node reached at its operate listener (ADR-0014, amendments
2026-10-12 and 2026-10-13) — is `RuntimeRules.Remotes` (`RuntimeRemotes`):
`Open` gives a `RemoteNode` over `host:port` with PEM certificate, key and
anchors, and `OpenLocal` one over the local endpoint of a node on this
machine — a named pipe on Windows, a Unix domain socket elsewhere, no
certificate, the node taking every call as the user this process runs as,
each call waiting at most the `within` it is given or else what the node
declared from its `[tuning] local_timeout`, two seconds by default, then
saying *the node did not answer within* it;
its `Publication`, `Cluster`, `Wait` and `Act` read the node's publication,
read the cluster's view the node gathers (`xmip_remote_cluster_v1`: a node
declaring the monitoring role answers it, any other refuses with
`Unsupported` and its words), wait on the runtime's follow of what its
first wait names (`RemoteFollows`, the header's `XMIP_REMOTE_FOLLOWS_*`: the
node's own, the cluster's, or `Answered`, the cluster's where the node
answers one, else its own, each telling saying which the node's answer
decided; naming another later is a defect, thrown) and ask
an act as the identity the connection proved, and
its `Dispose` ends the follow, lets every call in flight return and frees the
handle; `Serve` and `ServeLocal` give a `ServedNode`, an operate listener
this process serves for what it `Post`s — and, from its first `PostCluster`,
a cluster's gathered view it answers the cluster with — each caller holding
the role of the list naming it — observers, operators, developers — and none
where no list does. Every .NET surface operates a node
through these, on its machine and elsewhere alike; the C ABI is crossed
directly only for what has no node behind it — validating, editing and
slicing configuration, the language server's design exports, module
probing, the audit file, a person's certificate and Modules. Section 18 is
`RuntimeRules.SecurityRoles` (`RuntimeSecurityRoles`): the role a word names,
its words, and `Admitted`, the one gate `GatedOperator` asks. Section 19 — a
person's certificate (ADR-0034, amendment 2026-10-15) — is
`RuntimeRules.Certificates` (`RuntimeCertificates`): `Request` makes a fresh
key and a request naming the person, which every build carries, the phone's
remote build too; `Sign` signs a request by the authority a node
configuration's `[provision.operator]` names, which only a node's build
carries, refused in words in a remote build; the configuration is read as
its trust admits a slice, so one edited by hand is refused (ADR-0031,
amendment 2026-11-19). For a key that never leaves
its platform's keeper, `Information` and `Assemble` are the request's two
halves around the keeper's signature, every build carrying them, and
`RuntimeRemotes.OpenKept` (section 17's `xmip_remote_open_kept_v1`) reaches a
node presenting the chain with the keeper signing each handshake through a
callback. `Xmip.Surface`'s `OperatorCertificate` writes their files once,
for `xmip certificate` and `New-XmipCertificate`, and makes a request
over a keeper's key and keeps the chain signed for it (`RequestKept`,
`Keep`, `KeepAnchors`); `IOperatorKeeper` is the one interface each
platform's keeper implements, and `SecureSurface` names a keeper's key where a
host's document names a `Certificate` and no `PrivateKey`. Section 22 is
`RuntimeCertificates.Issue`, a node's build only: a node's key and
certificate from the cluster's own authority kept in a folder, made there
where none is, written as `node.pem`, `node.key` and `anchors.pem`
(`NodeIssued`), which `Xmip.Surface`'s `NodeCertificate` calls for
`xmip certificate --node` and `New-XmipCertificate -Node` (ADR-0015,
amendment 2026-10-21). Section 23 is `RuntimeCertificates.Authority`, a node's
build only: an act on the cluster's authority kept in a folder — set up,
inspect, revoke, export, back up — or on a node's certificate carried across
an air gap — request, sign, import, read — its arguments and answer JSON,
which `Xmip.Surface`'s `AuthorityCertificate` reads into records for
`xmip certificate`, `New-` and `Get-XmipCertificate` and the desktop's
Authority page (ADR-0034, amendment 2026-10-22). Section 21 — a
web host's secure surface, protected by Xmip itself (ADR-0009, amendment
2026-10-21) — is `RuntimeRules.Hosting` (`RuntimeSecureSurface`), a node's
build only: `Issue` keeps an authority in a folder, made there where none is,
and issues a host a certificate for `127.0.0.1` under it
(`SecureSurfaceIssued`), and `Open` gives a `SecureSurfaceListener` taking
the `Protection` as a parameter (`Protection.Tls`, `XMIP_PROTECTION_TLS`, the
one built — the owner, 2026-10-21: *technology and security should not be in
functions, parameters is ok*), whose `Accept` hands over each browser's
`SecureSurfaceSession` with its handshake finished, or says a certificate the
anchors refused (`SecureSurfaceAccepted`); a session is read on one thread and
written on another, and its `Presented` is the browser's certificate, DER.
Section 10 — the
cluster's `cluster.xmip` read, edited and sliced — is `RuntimeRules.Design`
(`RuntimeDesign`): `TryViews`, `TryEdit`, `TrySlices`, `TryFilterStructure`
and `TryFilterText`, each the header's JSON or text, or the runtime's
refusal; `Xmip.Surface` reads them as `ConfigurationViews`, `ClusterEdit` and
`NodeSlice`, which the desktop's Configure page draws and sends; the cluster's
file it edits — opened, compared, locked and written whole
(`ClusterConfiguration`), sliced and delivered to each node
(`SliceDelivery`, `NodeDelivery`, `SliceDelivery.From` for where), and
validated, saved and planned with every step audited (`RuntimeCommands`) —
is `Xmip.Surface`'s too, beneath the page. Section 12 — the
technologies the runtime carries and what each declares a Location may set —
is `RuntimeRules.Catalogue` (`RuntimeCatalogue`): `TryRead` brings back the
header's JSON, every technology or the one named, for the desktop editor's
Location form. Section 13 — a System Process declared (ADR-0053) — is
`RuntimeRules.Processes` (`RuntimeProcesses`): `Declare` has the runtime write
the calling process's declaration and answers the file, or the node's
refusal of a purpose that is no word; `Read` brings back the declarations
standing in a directory as `ProcessDeclarations` of `ProcessStanding`; and
`Undeclare` has the runtime take one away, refusing a file that is no
declaration's — `ProcessDeclaration.Dispose` and the estate's
`Read-XmipProcessDeclaration` call it, and neither deletes a file itself.
Section 25 — every Report listed and one opened (ADR-0072, amendment
2026-11-10) — is `RuntimeRules.Reports` (`RuntimeReports`): `Ask` hands the
act, `list` or `open`, and its JSON arguments over and brings back the
header's JSON, asking again at the true length until it fits — an open so
asked again is audited once, by the call that receives it — or throws the
runtime's sentence, opening REFUSED or FAILED; `ReportKind` is
`XmipReportKind`.
A node's build only: a remote build refuses it in words (`RemoteBuild`).
Section 26 — a module library probed (ADR-0012, amendment 2026-11-17) — is
`RuntimeRules.Modules` (`RuntimeModules`): `Probe` hands the library's path
over and brings back `ModuleProbed`, the runtime loader's answer — why it
could not be asked, the entrypoint's status, what it declares, why the host
refuses it, whether it conforms and the lines it logged — the one object
`xmip probe` renders and `Get-XmipModuleDescriptor` emits; a node's build
only. Section 27 — a configuration document read, written and one key set
(ADR-0031, amendment 2026-11-17) — is `RuntimeRules.Documents`
(`RuntimeDocuments`), in a remote build too: `TryRead`, `TryWrite` and
`TrySet`, `xmip-core-configure`'s each, through which `TomlDocument` and the
estate's `Read-XmipConfiguration` read every configuration document.
In a composed estate the project copies the runtime's built library beside
everything that references it, and `build/Xmip.Operate.Native.props`
packages it into a phone host. `dotnet/Xmip.Operate.Test` calls each section
against the runtime this estate built, and holds a subscribe and a listen to
the shape the header declares; `dotnet test dotnet/Xmip.Operate.Test` runs
them, after `cargo build` in this repository has left the library the
test assemblies load.

### What every .NET surface shares

`dotnet/Xmip.Surface` (ADR-0052) is the one implementation behind all four
operator surfaces; the GUI hosts, the cli and the PowerShell module are thin
faces over it.

#### What each source answers

A face reads one of three sources, chosen by `Surface = "local" |
"snapshot" | "remote"` in its `[Xmip]` table, or by `--remote`, then
`--local`, then `--snapshot` on a line (`SurfaceChoice`); `--runtime` names
the library every one calls the runtime through and chooses none. They do
not answer alike, and an operator reading a view should know which one it
is:

| | `local` (`LocalOperator`) | `snapshot` (`SnapshotOperator`) | `remote` (`RemoteOperator`) |
| --- | --- | --- | --- |
| Reads | a node's publication on this machine, over gRPC to its operate listener's local endpoint — a named pipe on Windows, a Unix domain socket elsewhere (section 17) | a node's published `snapshot.xmip` | a node's publication on another machine, over gRPC to its operate listener in Xmip's mutual TLS (section 17) |
| Topology | the node's publication's | the publication's, where it carries one | the node's publication's |
| Failed Messages | what the node's publication carries: each Port's count and its oldest hundred; Unlisted while the node is unreachable | Listed: each Port's count and its oldest hundred; Unlisted with no file read | as local |
| A failed Message's Retry or Dismiss; a Subscription's or an Event subscription's pause and resume; a Dead Message's Replay | asked of the node, which takes it as the user this process runs as behind the role its `[operate]` grants (section 18's gate), applies it at once and audits it; refused in its words otherwise | submitted as an order where the publication says its node takes them, taken within a round; declined where it names none | as local, the act the certificate's subject's |
| A scope's pause and resume | declined: a node's operate listener takes the acts on what it publishes, not a scope's pause | declined: a snapshot cannot be paused | declined, as local |

*Listed* is an
answer even when it lists none; *Unlisted* is a source that cannot list;
*Unanswered* one that asked and was not answered, its sentence opening
`FAILED:` (`FailedMessageList`).

- **Surfaces.** `IOperatorSurface` with its implementations —
  `ListenerOperator` over a node's operate listener, the runtime's surface
  (section 17): `LocalOperator` on this machine and `RemoteOperator` on
  another — and `SnapshotOperator` over a published snapshot — and
  `ClusterSurfaces`, the set a face holds when more than one
  cluster is published: one surface per cluster, and nothing added across them
  (ADR-0052, amendments 2026-09-20 and 2026-11-13). Several clusters are named
  one way, each of its own kind: `Clusters = [{ Surface = "local",
  Local = … }, { Surface = "snapshot", Snapshot = … }]` in a host's document,
  or `--Xmip:Clusters:0:Surface=local` on its line; `Snapshot` is one path,
  one cluster. `SurfaceChoice.OpenAll` opens each by the one rule a single
  surface is, refused in its own keys, and the list wins over `Surface`; a
  command and the prompt read the first (`OpenFirst`). A surface that has
  published nothing when the set is opened — a node not answering yet —
  names no cluster and is refused with nothing; it is named by its first
  publication (`ClusterListTest`). A `ListenerOperator` follows what the
  runtime calls answered (`RemoteFollows.Answered`): the cluster's view where
  the node it reaches gathers one (`Gathers`), its own publication where the
  node refuses, the node's answer deciding in the runtime as each connection
  is made, never a name; it reads what the follow says it follows. Its
  `Source` then says, after where the node is reached, what the view says of
  itself — the cluster, gathered by which node, and how many nodes it
  reached, the runtime's words (`GatheredClusterTest`; ADR-0052, amendment
  2026-11-13). A `ListenerOperator` tells every watch
  first and reads the publication by the next question: a change reaches a
  .NET watch in about 0.3 ms on the local endpoint (`LocalOperatorTest`
  holds it under a millisecond). `ConfigurationRuntime` asks the runtime
  what has no node behind it: `Validate`, and `Plan`, which hands a node's
  file to `xmip_start_v1`, which reads, validates and publishes its plan and
  runs nothing; its sentence says *planned … not running*. A node that
  a `ListenerOperator` loses is told at once to every watch, with a `Source` that
  says it is unreachable and why, and every answer is empty until it is back
  — never the publication held from before; the runtime's follow tries it
  again on its own loop (0, 2, 10 and 30 seconds, the last repeated), whether
  it was lost or never reached, and every watch is told the moment it
  answers (`RemoteOperatorTest`, `LocalOperatorTest`; the runtime's
  `tests/plane`). A snapshot is read once per change: a
  followed `SnapshotOperator` reads each publication before the runtime's
  follow tells a watch of it (`HeldSurface`), and answers every question from
  that reading without touching the file; `SurfaceChoice.OpenAll` follows
  every surface its `Clusters` names (`ClusterSurfaces.Follow`), so no render
  reads one (ADR-0052, amendment 2026-10-03; `SnapshotReadCostTest` holds the
  bound). A watch begun before its source recovers when the source arrives
  (`operate::surface::snapshot_watch`): a snapshot whose publication's
  directory is not made yet says its first view, waits on the operating
  system's watch of the nearest folder above it and tells a change the
  moment the source is there — looking each tenth of a second only where not
  even the path's root exists to watch (`SurfaceArrivalTest`).
- **Subscriptions.** `IOperatorSurface.Subscriptions` lists what the nodes
  route by and `IOperatorSurface.Act` pauses or resumes one (`SubscriptionAct`,
  which holds no remove, answered as a `SubscriptionOperation`; a
  Subscription is added and removed in the TOML configuration,
  `SubscriptionOperation.Configured` says so in words): by the node, over
  its operate listener, for a `ListenerOperator`, and for `SnapshotOperator`
  declined in words (`SnapshotOperator.NotActed`): a snapshot is never acted
  on (ADR-0014, amendment 2026-11-13). `SubscriptionQuery` is the
  one drill, pattern and order every surface asks of them (ADR-0013,
  amendment 2026-09-30).
- **Event subscriptions.** `IOperatorSurface.EventSubscriptions` lists what
  the nodes' hubs hold and `IOperatorSurface.Act` pauses, resumes or removes
  one (`EventSubscriptionAct`, answered as an `EventSubscriptionOperation`),
  reached the same three ways. `EventSubscriptionQuery` is their one drill,
  pattern and order (ADR-0065, amendments 2026-09-29 and 2026-09-30), and
  `EventSubscriptionQuery.Unheard` and `Line` the one selection and wording
  of the members the nodes there do not hear — *RX1: not hearing
  `<node>` since `<time>`: `<why>`* — which every surface shows read-only
  beside them (amendment 2026-10-02); the links between nodes are never
  listed or acted on.
- **The tree.** `ScopeTree` — the tree, its rollup and the worst leaf beneath
  a scope, over the runtime's containment, parts, stage words and order;
  `ScopeIndex`, a publication read once as that tree with every answer a
  lookup (ADR-0052, amendment 2026-09-15); `Branch` and `Crumb` for
  the drill-down, and `ScopeItem` for one row of it — its mood, its figures
  and the leaf that explains it (`Worst`), the next scope on the way to the
  cause — whether it `Exists`, and `Selected`, the rows a selection names,
  worst first under a wildcard, which `xmip show` and `Get-XmipScope`
  both answer with. Every drill starts at `IOperatorSurface.Root`, the scope the
  publisher publishes at (an Essentials cluster, never the root above it); a
  stage card counts `IOperatorSurface.Stage` and lists `Locations`, and the
  prompt's letters are `MessagePath`, each stage its own figure (the owner,
  2026-09-26: *drill-down does not work and datapoints are wrong*).
- **Narrowing.** `ScopePattern`, the one wildcard every surface matches with
  (ADR-0059 clauses 7 and 8), `node::wildcard`'s, called
  through `RuntimeRules.Matches` (`xmip_scope_matches_v1`); `ScopeFilter`, that pattern applied to a
  publication so the views narrow alike and none decides for itself what a
  pattern means; and `ScopeSelection`, what a scope argument selects on the
  command line and in a cmdlet — the scope itself, or the topmost scopes a
  wildcard names — with the REFUSED sentence when it names nothing.
- **Figures.** `Figures`, the six at a scope in the order every surface says
  them, each also by the kind counted (`Of`); `FigureFlow`, the five as a rate
  per second between two publications; and `FigureWatch`, the one place a
  face remembers figures between two reads — the prompt's letters and the
  board's stage cards alike — so the rate is computed once.
- **Following.** `SurfaceFollow`, the one follow: the answer now, then a new
  one whenever the publication advances and the answer changed, a wildcard
  matched again at every notice, until cancelled — `xmip --follow` and
  `Get-XmipHealth -Follow` / `Get-XmipScope -Follow` run it.
- **What a run says.** `RunHeader` for the `[run]` table a publisher writes —
  `Hidden` where the run declared itself hidden, and `ClusterSurfaces` lists,
  reaches and names such a cluster only for a face that includes what is
  hidden, by the one rule `observe::run::shown` through `RuntimeRules.Shown`
  (`xmip_run_shown_v1`; ADR-0028 and ADR-0052, amendments 2026-09-30) —
  `NodeCapability` for what one node declared — its roles, and the stages they
  serve — never inferred from what the node is called (ADR-0056 clause 1),
  read by the rule of `node::NodeRole::declared` (lowercase exactly, any
  other word refused and the refusal carried in `Refusal`; amendment
  2026-10-01) — and `Topology` for the communication view.
- **Acts and verdicts.** `ScopeAction`, the two acts `xmip_operate.h` carries
  and no start, stop or restart; `ScopeOperation`, the one shape they answer
  in, so an exit code and a pipeline object agree, and `ScopeOperation.Who`,
  who paused — the name stated, else the user the process runs as; and
  `ConfigurationVerdict` for what came of handing the runtime a node
  configuration — a saved file, or the text an editor holds.
- **Plumbing.** `RuntimeLibrary` (discovery by one rule, written here and
  nowhere else in the estate: a library an operator typed, else
  `XMIP_RUNTIME_LIBRARY`, else beside the executable — never the surface's
  document, which the runtime reads (ADR-0052, amendment 2026-11-17); and
  `Rules`, the one library a process calls the runtime's rules in, and its
  gRPC client — every surface alike),
  `SurfaceChoice` (the surface a host chose in
  its TOML, one per cluster under `Clusters`, and — over a `SurfaceLine` — the one
  precedence the executable, the prompt and the cmdlets share: a remote
  node, a local node or a snapshot stated for the invocation, then the
  document, else no node, said so (`LocalOperator.Unnamed`); a line whose
  remote names no node is refused in `RemoteOperator.Refusal`'s one
  sentence, `SurfaceLine.Refusal`),
  `TomlDocument` (the one way a surface reads a TOML document, a host's
  configuration among them through `TomlConfigurationSource`, and sets one
  key of it, comments and layout kept — each by `xmip-core-configure`, the
  runtime's one reader, through section 27; no TOML package in .NET),
  `ProcessDeclaration` (what a System
  Process Xmip owns says of itself, ADR-0053 clause 3, declared and listed
  through the node — section 13 — and never written here), `English` (a status
  said in words once, a mood's word and color name asked of the runtime,
  a stage's name, a rate on its one K, M and G ladder — `Moving` bare for the
  prompt, `Rate` with its unit, `Flow` for a rate not known yet —
  and what a surface says of a scope it holds nothing at, beneath or measured)
  and `SurfaceChange` (one coalescing change stream that wakes every
  surface when a published snapshot advances).
- **TLS.** `SecureSurface`, what a surface presents and trusts when it
  crosses a network (ADR-0063 clause 1): a PEM chain and key and the anchors
  a peer's chain must reach — `Xmip:Certificate`, `Xmip:PrivateKey` and
  `Xmip:TrustAnchor`, else `XMIP_CERTIFICATE`, `XMIP_PRIVATE_KEY` and
  `XMIP_TRUST_ANCHOR`, else nothing presented and nothing trusted — the
  paths alone (`Paths`), handed to the runtime, whose TLS reads them; no
  .NET TLS stack does — and `Permits`, the one rule for connecting and
  binding alike: HTTPS always, plain HTTP to this machine only.
  `RemoteOperator` presents and checks through it, and says a refused
  certificate in its reason.
- **Events.** `EventFeed`, a .NET program's subscription to the process's
  hub: it finds the runtime by `RuntimeLibrary`'s one rule, subscribes as
  the Party the account it runs as proves to be among the Parties of the
  node serving in its process — it names none — audits where the program's
  `ProgramAudit` records go, and `Follow` yields each Event as it arrives
  until cancelled. Only code in a node's own process can subscribe, and no
  export lets a node serve in a face's process (`xmip_start_v1` runs
  nothing, open problem 20), so in a face it is refused today; no face
  calls it (open problem 66). A node elsewhere sends its Events over the
  wire (`xmip-core-event`'s `wire`).
- **Audit.** `ProgramAudit`, how every .NET program audits (ADR-0062): its
  name, the directory its configuration names (`Xmip:AuditDirectory`), the
  runtime library the program was told to load (`Library`, so `--runtime`
  reaches the audit), `Record`, `Failed` — one exception, one record — `WatchUnhandled`,
  and `Properties`, the one way a value becomes a record's text for every
  .NET program and the estate's script module — each
  a call into `xmip_audit_v1` and never a record of its own; and
  `Read(AuditQuery)`, every .NET surface's read of the records back — the
  Audit view, `xmip audit` and `Get-XmipAudit` — through
  `xmip_audit_read_v1` (ADR-0062, amendment 2026-09-29), `AuditQuery.Verify`
  asking each writer's audit chain walked (ADR-0070 clause 5); and
  `Unreached`, the refusal that says why when the runtime's library cannot
  be loaded at all: .NET writes no operating system's log of its own, and
  reaches it through `xmip_audit_v1` alone.
- **Reports.** `ReportCatalog`, every .NET surface's one list of every
  Report and one opened (ADR-0072, amendment 2026-11-10) — `xmip report`,
  `Get-XmipReport` and the Reports view — over section 25: `List` reads the
  `ReportListing`, the kinds' words (`ReportKindWords`, *Report* and
  *Probing report*, the runtime's) and a `ReportEntry` for each Report, its kind and
  its kind's words (`Words`, set once as the list is read), whether it is
  available, what to expect, the reader that presents a quick one (`ReadBy`,
  an entrypoint of the header each surface maps to its own command) and a
  Quest's parameter, row, levels and last probe, read back from where the
  surface's `ProgramAudit` records go, with the runtime's sentence where
  they cannot be (`Unread`); `Named` asks the runtime for the one Report of
  a name, which finds it and refuses a name that is none in its own
  sentence. `Open` probes the configuration a `ReportProbe` names — a
  node's, or a cluster's `cluster.xmip` with its node — the probe audited
  under the surface's program, and reads the `ReportProbed`: its tally, a
  `ReportCount` per state, what a row is called, a `ReportRow` per
  requirement and, for the row named, the `ReportRequirement` in full, what
  is behind it (`ReportBehind`) and the actions that close it
  (`ReportAction`); what a Quest does not answer is left empty. Every state,
  kind and word is the runtime's string; no list of them is kept in C#, and
  each list is an array, which PowerShell indexes.

`dotnet/Xmip.Surface.Relay` is what a web host adds for its browsers; only a
web host references it, and it serves no surface to other machines — a remote
surface follows a node, not a web host (ADR-0014, amendment 2026-10-12).
`SurfaceBinding` is where such a host listens: plain HTTP beyond loopback and HTTPS with no
certificate or no anchors refused before anything listens, the plain loopback addresses
answered for the host to say it binds them, and `UseSurface(protection, …)`,
every HTTPS address bound by `SecureTransport` — a Kestrel transport over the
runtime's listener (section 21), binding only a `SecureEndPoint` so plain
loopback keeps Kestrel's sockets — each browser's connection a
`SecureConnection` carrying the certificate it presented as Kestrel's
`ITlsConnectionFeature`, one the anchors refused told to the host's audit
(ADR-0063 clause 1; ADR-0009, amendment 2026-10-21), and who the runtime's
authentication gate answered the browser is (`xmip_caller_proven_v1`,
`SecureSurfaceSession.Caller`) as a `ProvenConnection`; and `ProvenCaller`,
who each request proved, read from that feature alone, so a certificate's
subject the host reads proves nothing (ADR-0009, amendment 2026-11-01).
`UseXmipContentSecurity` sets the Operation Web's content security policy
once, on every response (`ContentSecurity`): `default-src 'self'`, scripts
from the host and the one inline element carrying the response's nonce
(`ContentSecurity.Nonce`, which a page's `<ImportMap nonce=…>` carries),
`style-src 'self'`, `frame-ancestors 'none'`, `base-uri` and `form-action`
the host's own.
A caller crosses as an opaque handle the runtime handed out, never a name
(`HeldCaller`, section 18): the session holds the one it proved, a circuit
takes its own (`HeldCaller.Hold`, `xmip_caller_hold_v1`) and releases it
when it ends, the desktop's is the account the runtime read from the
operating system (`RoleAssignment.ProcessCaller`, `xmip_caller_process_v1`),
and a handle released, revoked with its listener, past its certificate or
forged is no one; `HeldCaller.Name` is for display alone. Through a node's
operate listener the node decides again: an act the gate admitted is taken
for its caller, a parameter of every act of `IOperatorSurface` (`RemoteNode.Act`,
`SetSecret` and `Secrets` taking the `HeldCaller`; the command line and the
PowerShell module pass none), the runtime hands the node the caller's
certificate, and the node — where its `[operate] relay` names this host —
acts by the caller's own role and audits who and through (ADR-0009, the
second amendment of 2026-11-01).
Every act a
screen takes goes through `GatedOperator` in `Xmip.Surface`, over the
runtime's one gate (`xmip_act_admitted_v1`, section 18): taken only where the
caller's `RoleContext` may operate, as the caller the runtime holds for the
connection, by its handle — over loopback as from anywhere, since loopback proves where a
caller is and not who (open problem 35) — and refused in words otherwise; taken or
refused, audited through the host's `ProgramAudit` (ADR-0009). `Role`,
`Roles` and `RoleContext` are `Xmip.Surface`'s, beneath both GUIs.

`dotnet/Xmip.Surface.Test` covers the tree and its index, the pattern, the
filter and the selection, the figures and their flow, runtime discovery and
the surface precedence, the English, a program's audit — into a directory,
to the operating system's log when the sink fails, and a refusal that says
why when audit cannot be reached — the
configuration verdict, the cluster's file edited, saved under its lock,
refused when another editor changed it, sliced and delivered to each node and
audited at every step, the process declaration, the snapshot surface and the
cluster set over fixtures, the remote surface against a hub on a loopback
port — plain, and over mutual TLS with certificates a test authority issued,
where a client certificate the host does not trust, a host certificate the
surface does not trust and no certificate at all are each refused — the
hub's role gate, where an Observer host refuses every act, an Operator host
takes each as the certificate's subject, and an act without one is refused,
from loopback as from elsewhere, over
the test cluster's names from `test/cluster.xmip` (`TestCluster`) — and where a
host may bind. It tests no rule the runtime owns: it proves the surface returns what
the runtime's export returns, and those rules are tested once, where they are
written. `dotnet test dotnet/Xmip.Surface.Test` runs them, after `cargo build`
in this repository has left the library the test assemblies load.
