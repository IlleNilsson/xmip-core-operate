using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>
/// One infrastructure endpoint or a component beneath it. <paramref name="ParentId"/>
/// forms the progressive drill-down; a null parent is an infrastructure-level node.
/// Load is normalized from zero to one; <paramref name="Activity"/> is what moved
/// through it per second over its publisher's window
/// (<c>observe::topology::window</c>), zero where nothing moved. A node of the
/// cluster says the <paramref name="Roles"/> it declares, comma separated, empty
/// for none and for anything that is no node; a Storage node listed in
/// <c>[storage] nodes</c> its <paramref name="Place"/> there, from zero, null for
/// anything else (ADR-0052 and ADR-0056, amendments 2026-11-13); and a Storage
/// node of several its <paramref name="Standing"/> as it reports itself running,
/// in the runtime's words — <c>Storage, standby</c> — empty for anything else
/// (<c>observe::topology::StorageStanding</c>; amendment 2026-11-25).
/// </summary>
public sealed record TopologyNode(
    string Id,
    string? ParentId,
    string Label,
    TopologyNodeKind Kind,
    string Scope,
    HealthState State,
    TopologyOrigin Origin,
    double Load,
    double Activity,
    string Evidence,
    string Roles = "",
    int? Place = null,
    string Standing = "");

/// <summary>
/// One semantic communication relationship. Transport acknowledgements are not
/// represented as a reverse application flow; <paramref name="Pattern"/> says
/// what the application exchange means, and <paramref name="Relation"/> how its
/// two ends are related in their cluster.
/// </summary>
public sealed record CommunicationLink(
    string Id,
    string From,
    string To,
    CommunicationPattern Pattern,
    TopologyOrigin Origin,
    string Protocol,
    HealthState State,
    ulong Volume,
    double Rate,
    double LatencyMilliseconds,
    double Progress,
    uint Attempts,
    string Evidence,
    TopologyRelation Relation = TopologyRelation.Traffic);

/// <summary>Where one drawn thing stands on the map, as the runtime lays it
/// out (<c>observe::topology::layout</c>, <c>xmip_publication_layout_v1</c>):
/// <paramref name="X"/> and <paramref name="Y"/> in the unit square, left to
/// right and top to bottom, which a surface scales to its canvas, and the
/// band and column it stands in.</summary>
public sealed record TopologyPlace(
    string Id,
    double X,
    double Y,
    TopologyBand Band,
    TopologyColumn Column);

/// <summary>A point-in-time configured and observed communication topology,
/// where the work at each thing it draws came from and went, by the scope it
/// is drawn at, as the runtime says it (<c>observe::topology::from_to</c>,
/// section 8; ADR-0027, amendment 2026-11-11), and where each thing beneath
/// each focus stands, by the focus's id, the empty string for the top
/// (<c>observe::topology::layout</c>; ADR-0052, amendment 2026-11-13).</summary>
public sealed record TopologySnapshot(
    IReadOnlyList<TopologyNode> Nodes,
    IReadOnlyList<CommunicationLink> Links,
    DateTimeOffset Observed,
    string Source,
    IReadOnlyDictionary<string, FromTo>? Paths = null,
    IReadOnlyDictionary<string, IReadOnlyList<TopologyPlace>>? Places = null)
{
    /// <summary>An empty snapshot from a surface that publishes no topology.</summary>
    public static TopologySnapshot Empty(string source)
    {
        return new TopologySnapshot([], [], DateTimeOffset.UtcNow, source);
    }

    /// <summary>Where each thing beneath <paramref name="focus"/> stands, as the
    /// runtime laid it out — null or empty for the top; an empty list where
    /// nothing was laid out there.</summary>
    public IReadOnlyList<TopologyPlace> PlacesAt(string? focus)
    {
        return Places is not null
            && Places.TryGetValue(focus ?? string.Empty, out IReadOnlyList<TopologyPlace>? placed)
            ? placed
            : [];
    }
}
