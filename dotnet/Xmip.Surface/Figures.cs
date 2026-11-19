using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// The six measurements at a scope, said in the same order by every surface:
/// the four quantities ADR-0027 clause 5 keeps apart — Streams, Messages,
/// Journeys, bytes — and the two outcome counts its amendment of 2026-09-14
/// added, Retrying and Failed. A figure is null when the publisher has not
/// published it, and a surface shows that as absent rather than as zero.
/// </summary>
public sealed record Figures(
    string Scope,
    ulong? Streams,
    ulong? Messages,
    ulong? Journeys,
    ulong? Bytes,
    ulong? Retrying,
    ulong? Failed,
    DateTimeOffset? Observed)
{
    /// <summary>Nothing published at a scope.</summary>
    public static Figures None(string scope)
    {
        return new Figures(scope, null, null, null, null, null, null, null);
    }

    /// <summary>Read the six figures for a scope from a surface, each kind as
    /// its measurement says it, summed as <see cref="Sum"/> sums.</summary>
    public static Figures Read(IOperatorSurface surface, string scope)
    {
        ArgumentNullException.ThrowIfNull(surface);

        Figures[] each =
        [
            .. Kinds.Select(counted => surface.Measure(scope, counted) is { } measured
                ? One(scope, counted, measured.Value, measured.Observed)
                : None(scope)),
        ];

        return Sum(scope, each);
    }

    /// <summary>
    /// Several scopes figures as one, under <paramref name="scope"/>: each
    /// figure the sum of the ones published, and absent where none of them
    /// published it — a figure nobody published does not become a zero by
    /// being added up; the newest observed wins.
    /// <c>operate::surface::Figures::sum</c>, called in the runtime
    /// (<c>xmip_figures_sum_v1</c>).
    /// </summary>
    public static Figures Sum(string scope, IEnumerable<Figures> parts)
    {
        ArgumentNullException.ThrowIfNull(parts);

        XmipFigures[] wire = [.. parts.Select(Wire)];
        XmipFigures sum = RuntimeLibrary.Rules.ScopeIndexes.Sum(wire);
        ulong?[] values =
        [
            .. new[]
            {
                sum.Streams, sum.Messages, sum.Journeys, sum.Bytes, sum.Retrying, sum.Failed,
            }
                .Select((value, kind) => (sum.Published & (1 << kind)) != 0 ? (ulong?)value : null),
        ];

        return new Figures(
            scope,
            values[0], values[1], values[2], values[3], values[4], values[5],
            sum.Observed != 0
                ? DateTimeOffset.UnixEpoch.AddTicks(sum.ObservedUnixNanos / 100)
                : null);
    }

    /// <summary>The figure of one counted kind — what a stage counts is
    /// <see cref="ScopeTree.CountedAt"/>'s to say, and this answers it without
    /// a surface keeping its own kind-to-figure table. Null where it was not
    /// published, or for a value the runtime does not define.</summary>
    public ulong? Of(Counted counted)
    {
        return counted switch
        {
            Counted.Streams => Streams,
            Counted.Messages => Messages,
            Counted.Journeys => Journeys,
            Counted.Bytes => Bytes,
            Counted.Retrying => Retrying,
            Counted.Failed => Failed,
            _ => null,
        };
    }

    /// <summary>Whether the publisher supplied at least one figure.</summary>
    public bool HasValues =>
        Streams.HasValue || Messages.HasValue || Journeys.HasValue
        || Bytes.HasValue || Retrying.HasValue || Failed.HasValue;

    // Every counted kind, in the order the six figures are said and a
    // published mask numbers them.
    private static readonly Counted[] Kinds =
    [
        Counted.Streams, Counted.Messages, Counted.Journeys,
        Counted.Bytes, Counted.Retrying, Counted.Failed,
    ];

    // One kind's figure at a scope, the rest unpublished.
    private static Figures One(string scope, Counted counted, ulong value, DateTimeOffset at)
    {
        ulong?[] values = [.. Kinds.Select(kind => kind == counted ? (ulong?)value : null)];

        return new Figures(
            scope, values[0], values[1], values[2], values[3], values[4], values[5], at);
    }

    // Figures as the header carries them.
    private static XmipFigures Wire(Figures figures)
    {
        ulong?[] values =
        [
            figures.Streams, figures.Messages, figures.Journeys,
            figures.Bytes, figures.Retrying, figures.Failed,
        ];

        return new XmipFigures
        {
            Streams = figures.Streams ?? 0,
            Messages = figures.Messages ?? 0,
            Journeys = figures.Journeys ?? 0,
            Bytes = figures.Bytes ?? 0,
            Retrying = figures.Retrying ?? 0,
            Failed = figures.Failed ?? 0,
            Published = (byte)values
                .Select((value, kind) => value.HasValue ? 1 << kind : 0)
                .Aggregate(0, (bits, bit) => bits | bit),
            ObservedUnixNanos = figures.Observed is { } at ? RuntimeScopeIndexes.UnixNanos(at) : 0,
            Observed = (byte)(figures.Observed.HasValue ? 1 : 0),
        };
    }
}
