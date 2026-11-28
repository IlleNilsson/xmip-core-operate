using System.Runtime.CompilerServices;
using System.Text.Json;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// A cluster as every surface shows it where it shows its name (ADR-0028,
/// amendment 2026-11-28): <see cref="Shown"/>, the short name its file
/// declares, else the name its run gives it, and <see cref="Title"/>, the
/// tooltip beside it, carrying the file's name, the short name, the run's
/// name and the file (the owner, 2026-11-28: <i>Also all UI need to use
/// tooltips for explanation and underlying information</i>). Both are the
/// runtime's (<c>operate::surface::cluster_named</c>, asked through
/// <c>xmip_cluster_said_v1</c>), so the command line, the PowerShell module
/// and every GUI say one thing.
/// </summary>
/// <param name="Name">The cluster's name, as its run gives it.</param>
/// <param name="Naming">How its publisher names it for a person.</param>
public sealed record ClusterNamed(string Name, PublishedNaming Naming)
{
    /// <summary>The cluster <paramref name="name"/> as <paramref name="surface"/>'s
    /// publisher names it.</summary>
    public static ClusterNamed Of(string name, IOperatorSurface surface)
    {
        ArgumentNullException.ThrowIfNull(surface);

        return new ClusterNamed(name, surface.Naming());
    }

    /// <summary>What a surface shows for the cluster: its short name, else
    /// its name.</summary>
    public string Shown => Text("shown");

    /// <summary>The tooltip beside it; empty where nothing is said.</summary>
    public string Title => Text("title");

    // The runtime's answer, asked once per record: a view asks it each time
    // it is drawn.
    private static readonly ConditionalWeakTable<ClusterNamed, StrongBox<JsonElement>> Answers =
        [];

    private string Text(string name)
    {
        JsonElement said = Answers.GetValue(this, static held => held.Ask()).Value;
        return said.GetProperty(name).GetString() ?? string.Empty;
    }

    private StrongBox<JsonElement> Ask()
    {
        string cluster = JsonText.Document(writer =>
        {
            writer.WriteString("name", Name);
            writer.WriteString("short_name", Naming.ShortName);
            writer.WriteString("file", Naming.File);
        });

        using JsonDocument said =
            JsonDocument.Parse(RuntimeLibrary.Rules.Sentences.ClusterSaid(cluster));
        return new StrongBox<JsonElement>(said.RootElement.Clone());
    }
}
