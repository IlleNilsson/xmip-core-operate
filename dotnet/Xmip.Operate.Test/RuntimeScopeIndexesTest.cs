using Xmip.Abi.Operate;

namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="RuntimeScopeIndexes"/> crosses section 28.1 of
/// <c>xmip_operate.h</c> to the runtime this estate built. The tree is
/// tested in Rust, where it is written (<c>operate::surface::scope_index</c>);
/// these prove the crossing: a record answered by its place, a scope's
/// children worst first, the lists and a selection, a refusal carried as the
/// runtime's sentence, and the composed rules.
/// </summary>
public sealed class RuntimeScopeIndexesTest
{
    private static readonly TestCluster Cluster = TestCluster.Read();

    // The test cluster's first node, and a Location received on and a Port
    // sent from there.
    private static readonly string Node = Cluster.NodeScope(0);
    private static readonly string Received = $"{Node}/receive/tcp";
    private static readonly string Sent = $"{Node}/send/out";

    private static readonly DateTimeOffset Seen = new(2026, 9, 11, 12, 0, 0, TimeSpan.Zero);

    private static RuntimeScopeIndexes Indexes => RuntimeRulesTest.Rules.ScopeIndexes;

    private static ScopeIndexHandle Built()
    {
        return Indexes.Build(
            [
                new HealthRecord(Received, HealthState.Fine, 0, string.Empty, Seen),
                new HealthRecord(Sent, HealthState.Holding, 70, "disk full", Seen),
            ],
            [(Received, Counted.Messages, 3UL, (DateTimeOffset?)Seen)],
            4,
            "test");
    }

    [Fact]
    public void AScopeIsAnsweredByItsRecordsPlacesAndItsChildrenWorstFirst()
    {
        using ScopeIndexHandle index = Built();
        ScopePlacement? node = Indexes.Placed(index, $"{Node}/");

        Assert.NotNull(node);
        Assert.Equal(Node, node.Scope);
        Assert.Equal([1, 0], node.Leaves);
        Assert.True(node.IsContainer);
        Assert.Equal(3UL, node.Sums[1]);
        Assert.Null(node.Sums[0]);
        Assert.Null(Indexes.Placed(index, TestCluster.ReadOther().Scope));
        Assert.Equal(
            ["send", "receive"], Indexes.Children(index, node.Scope).Select(child => child.Label));

        (ulong revision, int leaves, int scopes, DateTimeOffset? observed, string source) =
            Indexes.Facts(index);

        Assert.Equal((4UL, 2, "test", Seen), (revision, leaves, source, observed));
        Assert.True(scopes > 2);
    }

    [Fact]
    public void ListsLabelsAndASelectionAreTheTrees()
    {
        using ScopeIndexHandle index = Built();

        Assert.Equal(
            [$"{Node}/send"],
            Indexes.List(index, OperateAbi.ScopeListStageScopes, "send"));
        Assert.Equal(
            [Cluster.Name], Indexes.List(index, OperateAbi.ScopeListNodes, string.Empty));
        Assert.Equal([Sent], Indexes.List(index, OperateAbi.ScopeListMatching, "*/out"));
        Assert.Equal("Send", Indexes.Label(index, $"{Node}/send"));
        Assert.Equal(1, Indexes.WorstAtStage(index, "send"));
        Assert.Equal(3UL, Indexes.Measure(index, Received, Counted.Messages, Seen)!.Value);
        Assert.True(
            Indexes.Select(index, $"{Cluster.Scope}/node/*/send", "SRC", out string chosen));
        Assert.Contains("\"patterned\":true", chosen, StringComparison.Ordinal);
        Assert.False(Indexes.Select(index, $"{Cluster.Scope}/none*", "SRC", out string refused));
        Assert.StartsWith("REFUSED  no scope matches", refused, StringComparison.Ordinal);
        Assert.EndsWith("source SRC", refused, StringComparison.Ordinal);
    }

    [Fact]
    public void ARelativeAndASumAreTheTreesRules()
    {
        Assert.Equal($"{Node}/send", Indexes.Relative(Sent, OperateAbi.ScopeRelativeParent));
        Assert.Equal(Node, Indexes.Relative(Sent, OperateAbi.ScopeRelativeNodeScope));
        Assert.Equal(
            $"node/{Cluster.Nodes[0]}/send/out",
            Indexes.Relative(Sent, OperateAbi.ScopeRelativeName));

        XmipFigures sum = Indexes.Sum(
        [
            new XmipFigures { Streams = 2, Published = 1, Observed = 1, ObservedUnixNanos = 5 },
            new XmipFigures { Streams = 3, Failed = 1, Published = 1 | 32 },
        ]);

        Assert.Equal(
            (5UL, 1UL, 33, 5L),
            (sum.Streams, sum.Failed, sum.Published, sum.ObservedUnixNanos));
    }
}
