using Xmip.Operate;

namespace Xmip.Surface;

// A Work Process as a surface reads it: whether one is placed, what the Work
// Process card says while the process stage holds Subscriptions alone
// (ADR-0052, amendment 2026-11-11); and its Instances (ADR-0066, amendment
// 2026-11-25).
public partial interface IOperatorSurface
{
    /// <summary>
    /// Whether <paramref name="stage"/> is the stage a node publishes its
    /// Subscriptions beneath and everything configured there is one of them:
    /// no Work Process is placed on any node. The card titled Work Process
    /// then says so, beside the Subscriptions it counts, rather than teach
    /// that a Subscription is a Work Process. A Subscription is known by its
    /// node and name in the Subscriptions the publication lists, never by
    /// its name's words.
    /// </summary>
    public bool NoWorkProcessPlaced(string stage)
    {
        IReadOnlyList<SubscriptionRecord> listed = Subscriptions().Subscriptions;
        HashSet<string> subscriptions =
        [
            .. listed.Select(one =>
                ScopeTree.Join([.. ScopeTree.Parts(one.Node), stage, one.Name])),
        ];
        ScopeIndex index = Index();
        IReadOnlyList<string> configured =
        [
            .. index.StageScopes(stage)
                .SelectMany(index.Branches)
                .Select(branch => branch.Scope),
        ];

        return configured.Count > 0 && configured.All(subscriptions.Contains);
    }

    /// <summary>
    /// The Instances of each Work Process the nodes this surface reads run —
    /// the newest of each its node knows, running, waiting, failed, dismissed
    /// or completed, with their values and history — and the Messages that
    /// failed in each one's queue, each retried or dismissed as every failed
    /// Message is (runtime-model.md section 22; ADR-0066, amendment
    /// 2026-11-25). None where this surface cannot list them.
    /// </summary>
    public WorkProcessInstanceList WorkProcessInstances()
    {
        return WorkProcessInstanceList.Empty;
    }
}
