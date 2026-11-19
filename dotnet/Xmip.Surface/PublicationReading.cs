using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// A publication as a surface answers from it: what a surface the runtime
/// opened read (section 28.2) — the publication copied out of the handle it
/// lent and its scope tree the runtime built — laid out as the records
/// every screen, command and cmdlet reads. Read once, by the runtime.
/// </summary>
internal sealed record PublicationReading(
    ScopeIndex Index,
    TopologySnapshot Topology,
    RunHeader Run,
    string Root,
    SubscriptionList Subscriptions,
    EventSubscriptionList EventSubscriptions,
    DeadMessageList DeadMessages)
{
    /// <summary>Who published, in the publication's own words: a gathered
    /// view's <c>&lt;cluster&gt;, gathered by &lt;node&gt;</c> line among them.
    /// Empty where it names no one.</summary>
    public string Said { get; init; } = string.Empty;

    /// <summary>
    /// What every view asks of a publication, asked once, here: the run's
    /// line, the tree at the root and at the publication's own, its nodes
    /// and what each declared. A follow that reads ahead asks it before a
    /// view is told, so the view after a change answers from memory, as
    /// every view after it does (ADR-0052, amendment 2026-10-03: a
    /// snapshot is never read by a render).
    /// </summary>
    public void AskAhead()
    {
        _ = Run.Said;
        _ = Index.Placed(ScopeTree.Root);
        _ = Index.Placed(Root);
        _ = Index.Nodes();
        _ = Index.Capabilities();
    }

    /// <summary>Nothing published, said by <paramref name="source"/>.</summary>
    public static PublicationReading Nothing(string source)
    {
        return new PublicationReading(
            ScopeIndex.Empty(source),
            TopologySnapshot.Empty(source),
            RunHeader.None,
            ScopeTree.Root,
            SubscriptionList.Empty,
            EventSubscriptionList.Empty,
            DeadMessageList.Empty);
    }

    /// <summary>
    /// The reading <paramref name="read"/> handed over, its publication and
    /// its tree adopted — each freed or held from now on — the surface
    /// saying <paramref name="source"/>.
    /// </summary>
    public static PublicationReading Of(SurfaceReading read, string source)
    {
        ArgumentNullException.ThrowIfNull(read);

        Publication copied = RuntimeLibrary.Rules.Publications.Adopt(read.Publication);
        string said = read.Said.Length > 0 ? read.Said : source;

        return new PublicationReading(
            ScopeIndex.Over(read.Index, copied.Records),
            copied.Topology ?? TopologySnapshot.Empty(said),
            RunHeader.From(copied.Run),
            read.Root,
            copied.Subscriptions,
            copied.EventSubscriptions,
            copied.DeadMessages)
        {
            Said = read.Said,
        };
    }
}
