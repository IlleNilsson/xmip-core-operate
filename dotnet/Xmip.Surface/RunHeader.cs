using System.Runtime.CompilerServices;
using System.Text.Json;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// What a run was started with, as its publisher says it under <c>[run]</c>:
/// the cluster, the tests, the nodes, the roles and Node capabilities each of
/// them declared, and how hard. The owner, 2026-09-19: the run says
/// what it was started with — a board could not be told from the one before
/// it. A publication with no such table is <see cref="None"/>, and a surface
/// shows nothing for it. <see cref="Real"/> are the nodes on machines of their own, never spawned, and every other
/// node is simulated (ADR-0028, amendment 2026-10-21): a green board means
/// two different things in the two cases, and the line says which. Whether
/// it said anything, its line and what it started each node with are the
/// runtime's (<c>operate::surface::run_header</c>, asked through
/// <c>xmip_run_said_v1</c>), so every surface says the one line.
/// </summary>
public sealed record RunHeader(
    string Cluster,
    IReadOnlyList<string> Tests,
    IReadOnlyList<string> Nodes,
    IReadOnlyList<string> Roles,
    IReadOnlyList<string> Capabilities,
    string Stress,
    IReadOnlyList<string>? Real = null)
{
    /// <summary>What a publication that says nothing of its run is read as.</summary>
    public static RunHeader None { get; } = new(string.Empty, [], [], [], [], string.Empty);

    /// <summary>The nodes on machines of their own; none where the run
    /// simulated every node.</summary>
    public IReadOnlyList<string> RealNodes => Real ?? [];

    /// <summary>Whether the publisher said anything.</summary>
    public bool Said
    {
        get
        {
            JsonElement said = Asked();
            return said.GetProperty("said").GetBoolean();
        }
    }

    /// <summary>
    /// What one node was started with, by name — its roles in
    /// <c>[run].roles</c>, with the Node capabilities <c>[run]</c>
    /// lists apart folded back in. <see cref="NodeCapability.None"/> when this
    /// run does not name the node. It is what the node was *started* with;
    /// what it *published* is <see cref="ScopeIndex.Capability"/>, and that is
    /// the one a surface prefers (ADR-0056 clause 1).
    /// </summary>
    public NodeCapability Capability(string node)
    {
        JsonElement said = Asked();

        foreach (JsonElement started in said.GetProperty("started").EnumerateArray())
        {
            if (string.Equals(Text(started, "node"), node, StringComparison.Ordinal))
            {
                IReadOnlyList<string> roles =
                [
                    .. started.GetProperty("roles").EnumerateArray()
                        .Select(role => role.GetString() ?? string.Empty),
                ];
                IReadOnlyList<string> capabilities =
                [
                    .. started.GetProperty("capabilities").EnumerateArray()
                        .Select(capability => capability.GetString() ?? string.Empty),
                ];

                return new NodeCapability(node, roles, capabilities, false, string.Empty)
                {
                    Refusal = Text(started, "refusal"),
                };
            }
        }

        return NodeCapability.None;
    }

    /// <summary>
    /// The one line every surface shows:
    /// <c>RoundTrip · &lt;cluster&gt; · nodes &lt;node&gt;=receiving
    /// &lt;other&gt;=processing+sending · Node capabilities &lt;node&gt;=online ·
    /// realistic</c>. A node is named with the roles it declared, because bare
    /// names say nothing a board could be told apart by; a publisher that says
    /// no roles leaves the names bare. The names are the operator's and mean
    /// nothing to Xmip (ADR-0053). A part the publisher left out is left out;
    /// no nodes and no Node capability are said in words, because both are
    /// choices.
    /// </summary>
    public string Line()
    {
        JsonElement said = Asked();
        return Text(said, "line");
    }

    /// <summary>The run a publication says it was started with, as the
    /// runtime read it (<c>observe::Run</c>), or <see cref="None"/> when it
    /// says nothing of its run.</summary>
    public static RunHeader From(PublishedRun? run)
    {
        return run is null
            ? None
            : new RunHeader(
                run.Cluster, run.Tests, run.Nodes, run.Roles, run.Capabilities, run.Stress,
                run.Real);
    }

    // The runtime's answer, asked once per record: a record does not change,
    // and a view asks it each time it is drawn.
    private static readonly ConditionalWeakTable<RunHeader, StrongBox<JsonElement>> Answers = [];

    // The run as the runtime says it, from its observe::Run JSON.
    private JsonElement Asked()
    {
        return Answers.GetValue(this, static held => held.Ask()).Value;
    }

    private StrongBox<JsonElement> Ask()
    {
        string run = JsonText.Document(writer =>
        {
            writer.WriteString("cluster", Cluster);
            Write(writer, "tests", Tests);
            Write(writer, "nodes", Nodes);
            Write(writer, "roles", Roles);
            Write(writer, "capabilities", Capabilities);
            Write(writer, "real", RealNodes);
            writer.WriteString("stress", Stress);
        });

        using JsonDocument said = JsonDocument.Parse(RuntimeLibrary.Rules.Sentences.RunSaid(run));
        return new StrongBox<JsonElement>(said.RootElement.Clone());
    }

    private static void Write(Utf8JsonWriter writer, string name, IReadOnlyList<string> texts)
    {
        writer.WriteStartArray(name);
        foreach (string text in texts)
        {
            writer.WriteStringValue(text);
        }

        writer.WriteEndArray();
    }

    private static string Text(JsonElement said, string name)
    {
        return said.GetProperty(name).GetString() ?? string.Empty;
    }
}
