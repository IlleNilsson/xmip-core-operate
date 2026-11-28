namespace Xmip.Operate;

/// <summary>
/// A publication — a node's or a roll's snapshot file — as the runtime read
/// it (<c>xmip_operate.h</c> section 8). The shape is
/// <c>observe::Publication</c>'s and nobody else's: a surface hands the text
/// to <see cref="PublicationReader"/> and gets this back, and writes no key,
/// word or fallback of its own (ADR-0052, amendment 2026-09-24).
/// </summary>
/// <param name="Source">Who published, in the publisher's words.</param>
/// <param name="Node">The scope it publishes at.</param>
/// <param name="Records">Its health records, worst first.</param>
/// <param name="Counts">Its counts, each at the scope it was recorded at.</param>
/// <param name="Topology">The topology it draws, or null.</param>
/// <param name="Run">What its run was started with, or null.</param>
/// <param name="Subscriptions">The Subscriptions its nodes route by
/// (ADR-0013, amendment 2026-09-30).</param>
/// <param name="EventSubscriptions">The Event subscriptions its nodes hold
/// (ADR-0065, amendment 2026-09-29).</param>
/// <param name="DeadMessages">What its nodes' Dead Message Queues keep, the
/// oldest of each (ADR-0052, amendment 2026-10-01).</param>
/// <param name="FailedMessages">The Messages that failed at its nodes' Send
/// Ports, how many at each and the oldest (runtime-model.md section
/// 13).</param>
/// <param name="WorkProcessInstances">The Instances of the Work Processes its
/// nodes run, the newest of each, and the Messages that failed in each one's
/// queue (runtime-model.md section 22; ADR-0066, amendment 2026-11-25).</param>
public sealed record Publication(
    string Source,
    string Node,
    IReadOnlyList<HealthRecord> Records,
    IReadOnlyList<MeasurementRecord> Counts,
    TopologySnapshot? Topology,
    PublishedRun? Run,
    SubscriptionList Subscriptions,
    EventSubscriptionList EventSubscriptions,
    DeadMessageList DeadMessages,
    FailedMessageList FailedMessages,
    WorkProcessInstanceList WorkProcessInstances)
{
    /// <summary>How its cluster is named for a person, under <c>[cluster]</c>:
    /// <see cref="PublishedNaming.None"/> where it says nothing.</summary>
    public PublishedNaming Naming { get; init; } = PublishedNaming.None;
}

/// <summary>How a publication's cluster is named for a person, as it says it
/// under <c>[cluster]</c>: <c>observe::Naming</c> (ADR-0028, amendment
/// 2026-11-28).</summary>
/// <param name="ShortName">The short name the cluster's file declares; empty
/// where the publisher does not say it.</param>
/// <param name="File">The cluster's file, as the publisher names it; empty
/// where it does not.</param>
public sealed record PublishedNaming(string ShortName, string File)
{
    /// <summary>What a publication that says nothing of it is read as.</summary>
    public static PublishedNaming None { get; } = new(string.Empty, string.Empty);
}

/// <summary>What a run was started with, as a publication says it under
/// <c>[run]</c>: <c>observe::Run</c>.</summary>
/// <param name="Cluster">The cluster's name.</param>
/// <param name="Tests">The tests that run.</param>
/// <param name="Nodes">The nodes spawned.</param>
/// <param name="Roles">The roles each node was started with, as
/// <c>node::Capability::entry</c> writes them.</param>
/// <param name="Capabilities">The Node capabilities each node was started
/// with, as <c>node::Capability::declared_entry</c> writes them,
/// <c>&lt;node&gt;=online+erp-direct</c> (ADR-0056, amendment 2026-11-24).</param>
/// <param name="Stress">The stress level's name.</param>
/// <param name="Real">The nodes on machines of their own, never spawned;
/// every other node is simulated (ADR-0028, amendment 2026-10-21).</param>
public sealed record PublishedRun(
    string Cluster,
    IReadOnlyList<string> Tests,
    IReadOnlyList<string> Nodes,
    IReadOnlyList<string> Roles,
    IReadOnlyList<string> Capabilities,
    string Stress,
    IReadOnlyList<string> Real);
