using Microsoft.Extensions.Configuration;

namespace Xmip.Surface.Test;

/// <summary>
/// The test cluster, the one place a .NET test takes a cluster's or a node's
/// name from (the owner, 2026-10-03: names in code and tests are parameters
/// and configuration, never literals; ADR-0056, amendment 2026-10-03). It
/// reads what the Rust tests read through <c>configure::fixture</c>: the
/// Reference Environment, <see cref="EstateFile"/> (the owner, 2026-11-28,
/// naming it by its short name: it <i>should evolve for all tests</i>;
/// ADR-0028, amendment 2026-11-28), named as its file names it, by the
/// runtime's one rule (<c>configure::cluster::cluster_name</c>, asked through
/// <c>xmip_cluster_views_v1</c>). One file, compiled into every .NET test
/// project that names a cluster or a node (each links it), so there is one
/// reader.
/// </summary>
internal sealed class TestCluster
{
    /// <summary>Where the estate keeps the Reference Environment, the test
    /// cluster's file, from the estate's root.</summary>
    public const string EstateFile =
        "test/core/system-test/configuration/Xmip-reference-environment.xmip";

    private TestCluster(
        string path, string name, IReadOnlyList<string> nodes, IReadOnlyList<string> roles)
    {
        Path = path;
        Name = name;
        Nodes = nodes;
        Roles = roles;
    }

    /// <summary>Where it was read from.</summary>
    public string Path { get; }

    /// <summary>The cluster's name its file declares.</summary>
    public string Name { get; }

    /// <summary>Its nodes' names, in ordinal order.</summary>
    public IReadOnlyList<string> Nodes { get; }

    /// <summary>What each node declares, its <c>roles</c>, in the order of
    /// <see cref="Nodes"/>.</summary>
    public IReadOnlyList<string> Roles { get; }

    /// <summary>The cluster's scope.</summary>
    public string Scope => $"{ScopeTree.Root}{Name}";

    /// <summary>The scope of the node at <paramref name="place"/>.</summary>
    public string NodeScope(int place)
    {
        return $"{Scope}/node/{Nodes[place]}";
    }

    /// <summary>The first node, in <see cref="Nodes"/>' order, that declares
    /// <paramref name="role"/>: a test finds a node by what it declares.</summary>
    /// <exception cref="InvalidOperationException">No node declares it.</exception>
    public string WithRole(string role)
    {
        for (int place = 0; place < Nodes.Count; place++)
        {
            if (Roles[place].Split(',', '+').Select(word => word.Trim()).Contains(role))
            {
                return Nodes[place];
            }
        }

        throw new InvalidOperationException($"no node of the test cluster {Path} declares {role}");
    }

    /// <summary>The test cluster: the estate's Reference Environment.</summary>
    /// <exception cref="InvalidOperationException">It is not there, or it
    /// names no cluster or no node: a test cannot run without it.</exception>
    public static TestCluster Read()
    {
        return Read(System.IO.Path.Combine(
            TomlDocument.BasePath(AppContext.BaseDirectory), EstateFile));
    }

    /// <summary>The second cluster a test of two takes beside
    /// <see cref="Read()"/>: its nodes under a name derived from its,
    /// <c>&lt;name&gt;-other</c>, as <c>configure::fixture</c>'s
    /// <c>TestCluster::other</c> derives it; no file is kept for it.</summary>
    /// <exception cref="InvalidOperationException">As <see cref="Read()"/>.</exception>
    public static TestCluster ReadOther()
    {
        TestCluster first = Read();
        return new TestCluster(first.Path, $"{first.Name}-other", first.Nodes, first.Roles);
    }

    private static TestCluster Read(string path)
    {
        if (!File.Exists(path))
        {
            throw new InvalidOperationException($"the test cluster {path} is not there");
        }

        if (!ConfigurationViews.TryRead(
                File.ReadAllText(path), out ConfigurationViews views, out string refusal))
        {
            throw new InvalidOperationException($"the test cluster {path}: {refusal}");
        }

        string name = views.Cluster.Length > 0
            ? views.Cluster
            : throw new InvalidOperationException($"the test cluster {path} names no cluster");
        IConfigurationRoot document = TomlDocument.Read(path);
        IConfigurationSection[] nodes =
        [
            .. document.GetSection("nodes").GetChildren()
                .OrderBy(node => node.Key, StringComparer.Ordinal),
        ];

        return nodes.Length == 0
            ? throw new InvalidOperationException($"the test cluster {path} declares no node")
            : new TestCluster(
                path,
                name,
                [.. nodes.Select(node => node.Key)],
                [.. nodes.Select(node => node["roles"] ?? string.Empty)]);
    }
}
