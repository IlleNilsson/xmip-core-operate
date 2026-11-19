using System.Text.Json;
using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// What a surface asks of the Subscriptions it lists (ADR-0013, amendment
/// 2026-09-30), in the words every surface uses: the Subscriptions view's
/// address, <c>xmip subscriptions</c>'s arguments and
/// <c>Get-XmipSubscription</c>'s parameters are these names. Where the drill
/// stands — a cluster, a node, one Subscription — the scope pattern over
/// each Subscription's node and name, and the order: written once, here, for
/// every surface.
/// </summary>
public sealed record SubscriptionQuery
{
    /// <summary>The columns a list is ordered by, in the order a reader shows
    /// them. The first is the default. The runtime's
    /// (<c>xmip_list_columns_v1</c>).</summary>
    public static IReadOnlyList<string> Columns =>
        RuntimeLibrary.Rules.Sentences.Columns(OperateAbi.List.Subscriptions);

    /// <summary>The scope pattern, <c>*</c> and <c>?</c>: a Subscription
    /// matches when its node, or its node and name as one scope, does.</summary>
    public string? Pattern { get; init; }

    /// <summary>Where the drill stands: a cluster or a node, and every
    /// Subscription routed by at or beneath it.</summary>
    public string? Location { get; init; }

    /// <summary>One Subscription's name on the node at
    /// <see cref="Location"/>.</summary>
    public string? Name { get; init; }

    /// <summary>A column of <see cref="Columns"/>; null is the first.</summary>
    public string? Sort { get; init; }

    /// <summary>ascending or descending; null is ascending.</summary>
    public string? Order { get; init; }

    /// <summary>Whether the order is greatest first.</summary>
    public bool Descending => string.Equals(Order, "descending", StringComparison.Ordinal);

    /// <summary>The cluster a Subscription is in: its node's first
    /// segment.</summary>
    public static string Cluster(SubscriptionRecord subscription)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        return ScopeTree.Parts(subscription.Node) is [var cluster, ..] ? cluster : string.Empty;
    }

    /// <summary>The node's name, the segment after <c>node/</c> of
    /// <c>xmip:///&lt;cluster&gt;/node/&lt;node&gt;</c>.</summary>
    public static string NodeName(SubscriptionRecord subscription)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        return ScopeTree.Node(subscription.Node);
    }


    /// <summary>A Subscription as one scope: its node's, then its name —
    /// what a pattern is matched against beside the node.</summary>
    public static string Scope(SubscriptionRecord subscription)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        return ListQuery.Word(
            OperateAbi.ListWord.SubscriptionScope, subscription.Node, subscription.Name);
    }

    /// <summary>
    /// What this query selects of <paramref name="subscriptions"/>, in its
    /// order, with ties by node and name: the runtime's
    /// (<c>operate::surface::subscription_query</c>).
    /// </summary>
    /// <exception cref="ArgumentException">REFUSED: a sort that is no column or
    /// an order that is neither word.</exception>
    public IReadOnlyList<SubscriptionRecord> Apply(IEnumerable<SubscriptionRecord> subscriptions)
    {
        ArgumentNullException.ThrowIfNull(subscriptions);

        return ListQuery.Chosen(
            OperateAbi.List.Subscriptions, subscriptions, "subscriptions", Write, writer =>
            {
                ListQuery.Text(writer, "pattern", Pattern);
                ListQuery.Text(writer, "location", Location);
                ListQuery.Text(writer, "name", Name);
                ListQuery.Text(writer, "sort", Sort);
                ListQuery.Text(writer, "order", Order);
            });
    }

    // One Subscription in section 14's JSON: the members the query reads.
    private static void Write(Utf8JsonWriter writer, SubscriptionRecord entry)
    {
        writer.WriteString("node", entry.Node);
        writer.WriteString("name", entry.Name);
        writer.WriteString("filter", entry.Filter);
        writer.WriteString("destination", entry.Destination);
        writer.WriteString("state", entry.State);
        writer.WriteNumber("picked_up", entry.PickedUp);
        writer.WriteNumber("held", entry.Held);
        writer.WriteNumber("since_unix_nanos", ListQuery.Nanos(entry.Since));
    }
}
