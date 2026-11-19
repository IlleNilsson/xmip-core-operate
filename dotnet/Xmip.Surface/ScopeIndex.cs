using System.Collections.Concurrent;
using System.Text.Json;
using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// One publication as the scope tree it is (ADR-0027): every scope with its
/// worst leaf, its rollup, the leaves beneath it, its children worst first
/// and its six figures, computed once by the runtime when the publication is
/// read — <c>operate::surface::scope_index</c>, the one tree every surface
/// reads (<c>xmip_operate.h</c> section 28.1; ADR-0052, amendment
/// 2026-11-17). A record is answered by its place in the records this index
/// was built from, which it keeps, so a record is never read twice; what a
/// scope answers is asked once and kept, since the index is immutable. A
/// new publication is a new index, and the change feed says when.
/// </summary>
public sealed class ScopeIndex
{
    private readonly ScopeIndexHandle handle;
    private readonly IReadOnlyList<HealthRecord> records;
    private readonly ConcurrentDictionary<string, ScopePlacement?> placed =
        new(StringComparer.Ordinal);
    private readonly ConcurrentDictionary<string, IReadOnlyList<Branch>> branches =
        new(StringComparer.Ordinal);
    private readonly ConcurrentDictionary<string, IReadOnlyList<string>> stages =
        new(StringComparer.Ordinal);
    private readonly ConcurrentDictionary<string, string> labels = new(StringComparer.Ordinal);
    private IReadOnlyList<string>? nodes;
    private IReadOnlyList<NodeCapability>? declared;

    private ScopeIndex(ScopeIndexHandle handle, IReadOnlyList<HealthRecord> records)
    {
        this.handle = handle;
        this.records = records;
        (Revision, Leaves, Scopes, Observed, Source) = Runtime.Facts(handle);
    }

    /// <summary>The publication this index is, as the change feed numbers it.</summary>
    public ulong Revision { get; }

    /// <summary>Where the publication came from.</summary>
    public string Source { get; }

    /// <summary>When the newest figure was observed, if any was.</summary>
    public DateTimeOffset? Observed { get; }

    /// <summary>How many leaves the publication holds.</summary>
    public int Leaves { get; }

    /// <summary>How many scopes it holds altogether, the branches and the root
    /// included — what a pattern is matched against.</summary>
    public int Scopes { get; }

    /// <summary>A count at a scope, as a publication states it: the sum beneath
    /// the scope is the index's to compute.</summary>
    public sealed record Count(
        string Scope, Counted Counted, ulong Value, DateTimeOffset? Observed);

    private static RuntimeScopeIndexes Runtime => RuntimeLibrary.Rules.ScopeIndexes;

    /// <summary>An index over nothing.</summary>
    public static ScopeIndex Empty(string source)
    {
        return Build([], [], 0, source);
    }

    /// <summary>The tree <paramref name="records"/> and
    /// <paramref name="counts"/> make, built by the runtime, each record
    /// kept at its place.</summary>
    public static ScopeIndex Build(
        IEnumerable<HealthRecord> records, IEnumerable<Count> counts, ulong revision, string source)
    {
        HealthRecord[] kept = [.. records];
        (string, Counted, ulong, DateTimeOffset?)[] counted =
        [
            .. counts.Select(count => (count.Scope, count.Counted, count.Value, count.Observed)),
        ];

        return new ScopeIndex(Runtime.Build(kept, counted, revision, source), kept);
    }

    /// <summary>
    /// The tree behind <paramref name="index"/>, a handle the runtime handed
    /// over (section 28.2: a surface's reading hands its own), held from now
    /// on; <paramref name="records"/> are the records it was built from, in
    /// their order, whose places it answers by.
    /// </summary>
    internal static ScopeIndex Over(nint index, IReadOnlyList<HealthRecord> records)
    {
        return new ScopeIndex(Runtime.Adopt(index), records);
    }

    /// <summary>
    /// What a node declared — its roles and its online capability — as the
    /// node itself published it at <c>&lt;node&gt;/capability</c> (ADR-0056
    /// clause 1: a node declares its capabilities and nothing is inferred).
    /// <see cref="NodeCapability.None"/> where it published nothing.
    /// </summary>
    public NodeCapability Capability(string node)
    {
        return Capabilities().FirstOrDefault(said => said.Node == node) ?? NodeCapability.None;
    }

    /// <summary>Every node that published what it declares, by name.</summary>
    public IReadOnlyList<NodeCapability> Capabilities()
    {
        return declared ??= Declared(Runtime.Capabilities(handle));
    }

    /// <summary>
    /// Every scope a wildcard names, in ordinal order (ADR-0059 clause 7): the
    /// match is the one wildcard every surface uses.
    /// </summary>
    public IReadOnlyList<string> Matching(string pattern)
    {
        return Runtime.List(handle, OperateAbi.ScopeListMatching, pattern);
    }

    /// <summary>Whether the publication says anything at or beneath a scope.</summary>
    public bool Contains(string scope)
    {
        return Placed(scope) is not null;
    }

    /// <summary>Health at and beneath a scope, worst first, most severe first
    /// within a mood, then by scope — the order every surface answers in.</summary>
    public IReadOnlyList<HealthRecord> Health(string scope)
    {
        return Placed(scope) is { } at ? new PlacedRecords(records, at.Leaves) : [];
    }

    /// <summary>The record at exactly this scope: a Location's own verdict.</summary>
    public HealthRecord? Own(string scope)
    {
        return Placed(scope) is { Own: >= 0 } at ? records[(int)at.Own] : null;
    }

    /// <summary>The worst leaf at or beneath a scope.</summary>
    public HealthRecord? Worst(string scope)
    {
        return Placed(scope) is { Leaves.Length: > 0 } at ? records[at.Leaves[0]] : null;
    }

    /// <summary>The rolled-up mood at a scope (ADR-0041): a leaf's own mood, a
    /// parent's Fine or Holding. Null when nothing is recorded there.</summary>
    public HealthState? Rollup(string scope)
    {
        return Placed(scope)?.Mood;
    }

    /// <summary>The six figures at a scope, summed over everything beneath.</summary>
    public Figures Figures(string scope)
    {
        return Placed(scope) is { } at
            ? new Figures(
                scope,
                at.Sums[0], at.Sums[1], at.Sums[2], at.Sums[3], at.Sums[4], at.Sums[5],
                Observed)
            : Surface.Figures.None(scope);
    }

    /// <summary>One count at a scope as a measurement, or null when the
    /// publication has none of that kind beneath it.</summary>
    public MeasurementRecord? Measure(string scope, Counted counted)
    {
        return Runtime.Measure(handle, scope, counted, DateTimeOffset.UtcNow);
    }

    /// <summary>What sits directly beneath a scope, worst first: a child with
    /// nothing deeper is a leaf with its own mood; anything deeper rolls up.</summary>
    public IReadOnlyList<Branch> Branches(string scope)
    {
        return branches.GetOrAdd(scope, at =>
        [
            .. Runtime.Children(handle, at).Select(child => new Branch(
                child.Scope,
                child.Label,
                child.Mood ?? HealthState.Fine,
                child.IsLeaf,
                child.Leaves.Length,
                child.Leaves.Length > 0
                    ? records[child.Leaves[0]]
                    : new HealthRecord(
                        child.Scope, HealthState.Fine, 0, string.Empty, DateTimeOffset.MinValue))),
        ]);
    }

    /// <summary>Every scope where a stage begins, ordinal: each node's
    /// <c>receive</c> for <c>receive</c>, what a stage card sums.</summary>
    public IReadOnlyList<string> StageScopes(string stage)
    {
        return stages.GetOrAdd(
            stage, word => Runtime.List(handle, OperateAbi.ScopeListStageScopes, word));
    }

    /// <summary>
    /// What a person reads a scope's own segment as, on every surface that
    /// shows it — a crumb, a drill row, a configuration row: a scope where a
    /// stage begins by the stage's title, the Work Process, and any other by
    /// its segment.
    /// </summary>
    public string Label(string scope)
    {
        ArgumentNullException.ThrowIfNull(scope);

        return labels.GetOrAdd(scope, at => Runtime.Label(handle, at));
    }

    /// <summary>The worst leaf on a stage of the message path — receive, process
    /// or send — wherever that stage sits in the tree. Null when no leaf is
    /// on it.</summary>
    public HealthRecord? WorstAtStage(string stage)
    {
        long place = Runtime.WorstAtStage(handle, stage);
        return place >= 0 ? records[(int)place] : null;
    }

    /// <summary>The first segment beneath the root of every scope: the nodes.</summary>
    public IReadOnlyList<string> Nodes()
    {
        return nodes ??= Runtime.List(handle, OperateAbi.ScopeListNodes, string.Empty);
    }

    /// <summary>A scope as the runtime's tree holds it, asked once; null where
    /// it holds no such scope.</summary>
    internal ScopePlacement? Placed(string scope)
    {
        ArgumentNullException.ThrowIfNull(scope);

        return placed.GetOrAdd(scope, at => Runtime.Placed(handle, at));
    }

    /// <summary>What <paramref name="pattern"/> selects among this tree's
    /// scopes (ADR-0059 clauses 7 and 8): the selection, or null with the
    /// refusal naming <paramref name="source"/>.</summary>
    internal ScopeSelection? Select(string pattern, string source, out string refusal)
    {
        if (!Runtime.Select(handle, pattern, source, out string answer))
        {
            refusal = answer;
            return null;
        }

        refusal = string.Empty;
        using JsonDocument chosen = JsonDocument.Parse(answer);
        JsonElement root = chosen.RootElement;

        return new ScopeSelection(
            root.GetProperty("argument").GetString() ?? pattern,
            root.GetProperty("patterned").GetBoolean(),
            [
                .. root.GetProperty("scopes").EnumerateArray()
                    .Select(at => at.GetString() ?? string.Empty),
            ]);
    }

    private static IReadOnlyList<NodeCapability> Declared(string json)
    {
        using JsonDocument said = JsonDocument.Parse(json);

        return
        [
            .. said.RootElement.EnumerateArray().Select(node => new NodeCapability(
                node.GetProperty("node").GetString() ?? string.Empty,
                [
                    .. node.GetProperty("roles").EnumerateArray()
                        .Select(role => role.GetString() ?? string.Empty),
                ],
                node.GetProperty("online").GetBoolean(),
                node.GetProperty("published").GetBoolean(),
                node.GetProperty("evidence").GetString() ?? string.Empty)
            {
                Refusal = node.GetProperty("refusal").GetString() ?? string.Empty,
            }),
        ];
    }
}
