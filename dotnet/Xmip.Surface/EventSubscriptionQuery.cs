using System.Text.Json;
using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// What a surface asks of the Event subscriptions it lists (ADR-0065,
/// amendment 2026-09-29), in the words every surface uses: the Event
/// subscriptions view's address, <c>xmip event-subscriptions</c>'s
/// arguments and <c>Get-XmipEventSubscription</c>'s parameters are these
/// names. Where the drill stands — a cluster, a node, one Event
/// subscription — the scope pattern over
/// each subscription's node and reach, and the order: written once, here,
/// for every surface.
/// </summary>
public sealed record EventSubscriptionQuery
{
    /// <summary>The columns a list is ordered by, in the order a reader shows
    /// them. The first is the default. The runtime's
    /// (<c>xmip_list_columns_v1</c>).</summary>
    public static IReadOnlyList<string> Columns =>
        RuntimeLibrary.Rules.Sentences.Columns(OperateAbi.List.EventSubscriptions);

    /// <summary>The scope pattern, <c>*</c> and <c>?</c>: a subscription
    /// matches when its node or the scope its filter reaches does.</summary>
    public string? Pattern { get; init; }

    /// <summary>Where the drill stands: a cluster or a node, and every
    /// subscription held at or beneath it.</summary>
    public string? Location { get; init; }

    /// <summary>One subscription's number on the node at
    /// <see cref="Location"/>.</summary>
    public ulong? Id { get; init; }

    /// <summary>A column of <see cref="Columns"/>; null is the first.</summary>
    public string? Sort { get; init; }

    /// <summary>ascending or descending; null is ascending.</summary>
    public string? Order { get; init; }

    /// <summary>Whether the order is greatest first.</summary>
    public bool Descending => string.Equals(Order, "descending", StringComparison.Ordinal);

    /// <summary>
    /// Who a subscription's subscriber is, as every surface shows it: the
    /// name its Party was declared with, and its identifier only where it was
    /// declared with none — shown as it is, never a name made from it.
    /// </summary>
    public static string Who(EventSubscriptionRecord subscription)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        return ListQuery.Word(
            OperateAbi.ListWord.EventWho, subscription.Subscriber, subscription.Party);
    }

    /// <summary>The cluster a subscription is in: its node's first
    /// segment.</summary>
    public static string Cluster(EventSubscriptionRecord subscription)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        return ScopeTree.Parts(subscription.Node) is [var cluster, ..] ? cluster : string.Empty;
    }

    /// <summary>The node's name, the segment after <c>node/</c> of
    /// <c>xmip:///&lt;cluster&gt;/node/&lt;node&gt;</c>.</summary>
    public static string NodeName(EventSubscriptionRecord subscription)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        return ScopeTree.Node(subscription.Node);
    }


    /// <summary>
    /// What this query selects of <paramref name="subscriptions"/>, in its
    /// order, with ties by node and number: the runtime's
    /// (<c>operate::surface::event_subscription_query</c>).
    /// </summary>
    /// <exception cref="ArgumentException">REFUSED: a sort that is no column or
    /// an order that is neither word.</exception>
    public IReadOnlyList<EventSubscriptionRecord> Apply(
        IEnumerable<EventSubscriptionRecord> subscriptions)
    {
        ArgumentNullException.ThrowIfNull(subscriptions);

        return ListQuery.Chosen(
            OperateAbi.List.EventSubscriptions, subscriptions, "event_subscriptions", Write, Asked);
    }

    /// <summary>
    /// The read-only line every surface shows for a member not heard: the
    /// node that does not hear it, then the runtime's own words —
    /// <c>&lt;node&gt;: not hearing xmip:///&lt;cluster&gt;/node/&lt;member&gt; since
    /// 2026-10-02T12:00:00Z: connection refused</c>.
    /// </summary>
    public static string Line(UnheardRecord unheard)
    {
        ArgumentNullException.ThrowIfNull(unheard);

        return ListQuery.Word(OperateAbi.ListWord.UnheardLine, unheard.By, unheard.Said);
    }

    /// <summary>
    /// The members not heard by the nodes this query stands at — each said in
    /// its one line, <c>not hearing &lt;node&gt; since &lt;time&gt;: &lt;why&gt;</c>
    /// — by node, then member: what every surface shows read-only beside the
    /// Event subscriptions, so no Event is missing silently (ADR-0065,
    /// amendment 2026-10-02). The links themselves are never listed.
    /// </summary>
    public IReadOnlyList<UnheardRecord> Unheard(IEnumerable<UnheardRecord> unheard)
    {
        ArgumentNullException.ThrowIfNull(unheard);

        UnheardRecord[] given = [.. unheard];
        string listed = ListQuery.Listed(given, "unheard", WriteUnheard);
        using JsonDocument answer =
            ListQuery.Asked(OperateAbi.List.EventSubscriptions, listed, writer =>
                ListQuery.Text(writer, "location", Location));

        return [.. ListQuery.Places(answer.RootElement, "unheard").Select(at => given[at])];
    }

    private void Asked(Utf8JsonWriter writer)
    {
        ListQuery.Text(writer, "pattern", Pattern);
        ListQuery.Text(writer, "location", Location);
        if (Id is { } id)
        {
            writer.WriteNumber("id", id);
        }

        ListQuery.Text(writer, "sort", Sort);
        ListQuery.Text(writer, "order", Order);
    }

    // One Event subscription in section 15's JSON: the members the query reads.
    private static void Write(Utf8JsonWriter writer, EventSubscriptionRecord entry)
    {
        writer.WriteString("node", entry.Node);
        writer.WriteNumber("id", entry.Id);
        writer.WriteString("subscriber", entry.Subscriber);
        writer.WriteString("party", entry.Party);
        writer.WriteString("action", entry.Action);
        writer.WriteString("scope", entry.Scope);
        writer.WriteString("state", entry.State);
        writer.WriteNumber("queued", entry.Queued);
        writer.WriteNumber("delivered", entry.Delivered);
        writer.WriteNumber("missed", entry.Missed);
        writer.WriteNumber("since_unix_nanos", ListQuery.Nanos(entry.Since));
    }

    // One member not heard in section 15's JSON: the members the query reads.
    private static void WriteUnheard(Utf8JsonWriter writer, UnheardRecord entry)
    {
        writer.WriteString("by", entry.By);
        writer.WriteString("node", entry.Node);
    }
}
