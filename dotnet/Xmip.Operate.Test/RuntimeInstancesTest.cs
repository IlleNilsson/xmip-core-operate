namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="RuntimeInstances"/> crosses section 29 to the runtime this
/// estate built and back (runtime-model.md section 22; ADR-0066, amendment
/// 2026-11-25): no node runs in a test's process, so no Work Process is
/// listed; and a publication's Instances are read with their state, values,
/// history and the Messages that failed in each one's queue. What a turn
/// keeps and how a failed Instance is retried is the runtime's and tested
/// there; these prove the crossing.
/// </summary>
public sealed class RuntimeInstancesTest
{
    private static readonly TestCluster Cluster = TestCluster.Read();

    // The test cluster's processing node, by what it declares.
    private static readonly string Node = $"{Cluster.Scope}/node/{Cluster.WithRole("processing")}";

    private static RuntimeRules Rules => RuntimeRulesTest.Rules;

    [Fact]
    public void NoNodeRunsHereSoNoWorkProcessIsListed()
    {
        Assert.Empty(Rules.Instances.Standing(Node, string.Empty, 0).WorkProcesses);
        Assert.Empty(Rules.Instances.Standing(string.Empty, "Order in parts", 1).WorkProcesses);
    }

    [Fact]
    public void APublicationsInstancesAreRead()
    {
        string text =
            $"node = \"{Cluster.Scope}\"\n"
            + $"[[work_process_instances]]\nnode = \"{Node}\"\n"
            + "work_process = \"Order in parts\"\nfailed_count = 1\n"
            + "[[work_process_instances.instances]]\ninstance = \"i-1\"\nstate = \"waiting\"\n"
            + "waits_for = \"message or time\"\nuntil_unix_nanos = 9000000000\n"
            + "correlations = [[\"Order\", \"A-1001\"]]\n"
            + "values = [[\"Ordered\", \"3\"], [\"Delivered\", \"1\"]]\n"
            + "turns = 2\nupdated_unix_nanos = 5000000000\n"
            + "[[work_process_instances.instances.history]]\nat_unix_nanos = 1000000000\n"
            + "what = \"started\"\nmessage = \"m-1\"\njourney = \"j-0\"\n"
            + "[[work_process_instances.failed]]\njourney = \"j-1\"\nsequence = 7\n"
            + "reason = \"Order in parts threw offline\"\n";

        Publication read = Rules.Publications.Read(text, out _)
            ?? throw new InvalidOperationException("no publication");
        WorkProcessInstances process = Assert.Single(read.WorkProcessInstances.WorkProcesses);
        WorkProcessInstanceRecord one = Assert.Single(process.Instances);
        FailedMessageRecord failed = Assert.Single(process.Failed);

        Assert.Equal((Node, "Order in parts", 1ul), (process.Node, process.WorkProcess,
            process.FailedCount));
        Assert.Equal(("i-1", "waiting", "message or time"), (one.Instance, one.State,
            one.WaitsFor));
        Assert.Equal(DateTimeOffset.FromUnixTimeSeconds(9), one.Until);
        Assert.Equal([new("Order", "A-1001")], one.Correlations);
        Assert.Equal([new("Ordered", "3"), new("Delivered", "1")], one.Values);
        Assert.Equal(2ul, one.Turns);
        InstanceHandledRecord started = Assert.Single(one.History);
        Assert.Equal(("started", "m-1", "j-0"), (started.What, started.Message, started.Journey));
        Assert.Equal(("j-1", 7ul, string.Empty), (failed.Journey, failed.Sequence,
            failed.SendPort));
        Assert.Equal(one, Assert.Single(read.WorkProcessInstances.Instances));
    }
}
