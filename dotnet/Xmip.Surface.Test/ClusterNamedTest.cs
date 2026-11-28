namespace Xmip.Surface.Test;

/// <summary>
/// A cluster is shown by the short name its publisher says, with the
/// tooltip carrying the rest, both the runtime's words through
/// <c>xmip_cluster_said_v1</c>; a publisher that says nothing of it shows
/// the cluster's name (ADR-0028, amendment 2026-11-28).
/// </summary>
public sealed class ClusterNamedTest
{
    private static readonly TestCluster Cluster = TestCluster.Read();

    [Fact]
    public void APublicationsShortNameShowsAndItsTooltipCarriesTheRest()
    {
        using TestFolder folder = new("surface-cluster-named");
        string snapshot = folder.Inside("named.xmip");
        File.WriteAllText(snapshot, $"""
            source = "test"
            node = "{Cluster.Scope}"

            [cluster]
            short_name = "Ab"
            file = "c/a-file.xmip"

            [[records]]
            scope = "{Cluster.NodeScope(0)}/receive/x"
            state = "fine"
            severity = 0
            evidence = "ok"
            observed_unix_nanos = 1
            """);

        using SnapshotOperator opened = new(snapshot);
        IOperatorSurface surface = opened;
        ClusterNamed named = ClusterNamed.Of(Cluster.Name, surface);

        Assert.Equal(new Xmip.Operate.PublishedNaming("Ab", "c/a-file.xmip"), surface.Naming());
        Assert.Equal("Ab", named.Shown);
        Assert.Equal(
            $"a-file · short name Ab · cluster {Cluster.Name} · from c/a-file.xmip", named.Title);
    }

    [Fact]
    public void APublisherSayingNothingShowsTheClustersName()
    {
        ClusterNamed named = new(Cluster.Name, Xmip.Operate.PublishedNaming.None);

        Assert.Equal(Cluster.Name, named.Shown);
        Assert.Equal($"cluster {Cluster.Name}", named.Title);
    }
}
