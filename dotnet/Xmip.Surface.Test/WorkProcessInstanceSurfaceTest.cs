using Xmip.Operate;

namespace Xmip.Surface.Test;

/// <summary>
/// The Instances of each Work Process as every surface lists them
/// (runtime-model.md section 22; ADR-0066, amendment 2026-11-25): a snapshot
/// lists what its publication carries — each Instance's state, keys, values
/// and history, and the Messages that failed in its queue — the gate passes
/// the list through, the one query narrows it by where, Work Process and
/// Instance, and a Message that failed there takes the act every failed
/// Message takes, which a snapshot declines (ADR-0014, amendment
/// 2026-11-13).
/// </summary>
public sealed class WorkProcessInstanceSurfaceTest : IDisposable
{
    private static readonly TestCluster Cluster = TestCluster.Read();

    // The test cluster's processing and receiving nodes, by what each declares.
    private static readonly string Processing =
        $"{Cluster.Scope}/node/{Cluster.WithRole("processing")}";
    private static readonly string Receiving =
        $"{Cluster.Scope}/node/{Cluster.WithRole("receiving")}";

    private readonly TestFolder scratch = new("instances");

    private string Audit => scratch.Inside("audit");

    [Fact]
    public void ASnapshotListsWhatItsPublicationCarries()
    {
        WorkProcessInstanceList listed = Over().WorkProcessInstances();

        WorkProcessInstances order = listed.WorkProcesses.Single(process =>
            process.WorkProcess == "Order in parts");
        Assert.Equal((Processing, 1ul), (order.Node, order.FailedCount));
        Assert.Equal(["i-1", "i-2"], order.Instances.Select(one => one.Instance));
        WorkProcessInstanceRecord waiting = order.Instances[0];
        Assert.Equal(("waiting", "message or time"), (waiting.State, waiting.WaitsFor));
        Assert.Equal([new("Order", "A-1001")], waiting.Correlations);
        Assert.Equal([new("Delivered", "1")], waiting.Values);
        Assert.Equal("started", Assert.Single(waiting.History).What);
        WorkProcessInstanceRecord failed = order.Instances[1];
        Assert.Equal(("failed", "offline", "j-1"), (failed.State, failed.Code, failed.FailedBy));
        Assert.Equal("j-1", Assert.Single(order.Failed).Journey);
        Assert.Equal(3, listed.Instances.Count());
    }

    [Fact]
    public void TheQueryNarrowsByWhereWorkProcessAndInstance()
    {
        WorkProcessInstanceList listed = Over().WorkProcessInstances();

        Assert.Equal(2, new WorkProcessInstanceQuery().Apply(listed).Count);
        Assert.Equal(
            ["Order in parts"],
            new WorkProcessInstanceQuery { Location = Processing }.Apply(listed)
                .Select(process => process.WorkProcess));
        Assert.Equal(
            ["Returns"],
            new WorkProcessInstanceQuery { WorkProcess = "Returns" }.Apply(listed)
                .Select(process => process.WorkProcess));
        (WorkProcessInstances process, WorkProcessInstanceRecord one) =
            new WorkProcessInstanceQuery { Instance = "i-2" }.One(listed)
                ?? throw new InvalidOperationException("not found");
        Assert.Equal(("Order in parts", "failed"), (process.WorkProcess, one.State));
        Assert.Null(new WorkProcessInstanceQuery { Instance = "none" }.One(listed));
        WorkProcessInstanceQuery elsewhere = new()
        {
            Location = Receiving,
            WorkProcess = "Order in parts",
        };
        Assert.Empty(elsewhere.Apply(listed));
    }

    [Fact]
    public void TheGatePassesTheListAndAFailedMessageTakesTheMessageAct()
    {
        ProgramAudit audit = new("Xmip.Surface.Test", Audit);
        SnapshotOperator inner = Over();
        GatedOperator gated = new(
            inner, new RoleContext(Role.Operator, RoleAssignment.ProcessCaller()), audit);

        Assert.Equal(
            inner.WorkProcessInstances().Instances.Count(),
            gated.WorkProcessInstances().Instances.Count());

        MessageOperation taken = gated.Act(Processing, "j-1", MessageAct.Retry, "operator", null);
        Assert.Equal(SnapshotOperator.NotActed, taken.Result);
        Assert.Empty(new SnapshotOperator(scratch.Inside("absent.xmip")).WorkProcessInstances()
            .WorkProcesses);
    }

    public void Dispose()
    {
        scratch.Dispose();
    }

    // Order in parts on the processing node: one Instance waiting, one failed
    // by a Message that waits in its queue; Returns on the receiving node.
    private SnapshotOperator Over()
    {
        string file = scratch.Inside($"snapshot-{Guid.NewGuid():N}.xmip");
        File.WriteAllText(
            file,
            $"node = \"{Cluster.Scope}\"\n"
            + $"[[work_process_instances]]\nnode = \"{Processing}\"\n"
            + "work_process = \"Order in parts\"\nfailed_count = 1\n"
            + Instance("i-1", "state = \"waiting\"\nwaits_for = \"message or time\"\n"
                + "until_unix_nanos = 9000000000\ncorrelations = [[\"Order\", \"A-1001\"]]\n"
                + "values = [[\"Delivered\", \"1\"]]\n")
            + "[[work_process_instances.instances.history]]\nat_unix_nanos = 1000000000\n"
            + "what = \"started\"\nmessage = \"m-1\"\njourney = \"j-0\"\n"
            + Instance("i-2", "state = \"failed\"\ncode = \"offline\"\n"
                + "reason = \"the unit cannot reach its stock\"\nfailed_by = \"j-1\"\n")
            + "[[work_process_instances.failed]]\njourney = \"j-1\"\nsequence = 7\n"
            + "reason = \"Order in parts threw offline\"\n"
            + $"[[work_process_instances]]\nnode = \"{Receiving}\"\nwork_process = \"Returns\"\n"
            + Instance("i-3", "state = \"completed\"\n"));

        return new SnapshotOperator(file);
    }

    private static string Instance(string id, string rest)
    {
        return $"[[work_process_instances.instances]]\ninstance = \"{id}\"\n{rest}"
            + "turns = 2\nupdated_unix_nanos = 5000000000\n";
    }
}
