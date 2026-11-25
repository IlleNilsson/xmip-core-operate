using Xmip.Abi.Operate;
using Xmip.Surface;

namespace Xmip.Operate.Test;

/// <summary>
/// The runtime's remote build, the phone's (ADR-0014, amendment
/// 2026-10-14), loads: its rules, publication read, client and security
/// roles answer, and a node's binding is refused in words, never a crash
/// (<see cref="RemoteBuild"/>).
/// </summary>
public sealed class RemoteBuildTest
{
    private static readonly Lazy<RuntimeRules> Loaded = new(() =>
    {
        string library = Built();

        Assert.True(
            File.Exists(library),
            $"no remote build at {library}: at the estate's root in PowerShell, " +
            "Import-Module -Name ./Xmip/Xmip.psd1; Build-XmipService -Remote Desktop");

        return RuntimeRules.Load(library, out string reason)
            ?? throw new InvalidOperationException(reason);
    });

    private static RuntimeRules Rules => Loaded.Value;

    [Fact]
    public void TheRemoteBuildLoadsAndItsRulesAnswer()
    {
        Assert.NotEmpty(Rules.StageWords);
        Assert.NotEmpty(Rules.RoleWords);

        string word = Rules.Word(HealthState.Paused)
            ?? throw new InvalidOperationException("no word for Paused");

        Assert.Equal(HealthState.Paused, Rules.Named(word));
    }

    [Fact]
    public void TheRemoteBuildCarriesTheSecurityRoles()
    {
        Assert.Equal(1, Rules.SecurityRoles.Named(" Operator "));
        Assert.Equal(("Developer", "also builds"), Rules.SecurityRoles.Words(2));
        Assert.False(Rules.SecurityRoles.Admitted(null, 2, "pause", "x", out _));

        // A remote build runs no gate of its own: it proves no caller.
        Assert.Null(Rules.SecurityRoles.Process(out string refused));
        Assert.Contains("remote build", refused, StringComparison.Ordinal);
    }

    [Fact]
    public void ANodesBindingIsRefusedInWords()
    {
        InvalidOperationException refused =
            Assert.Throws<InvalidOperationException>(() => Rules.Events);

        Assert.StartsWith(RemoteBuild.Sentence, refused.Message, StringComparison.Ordinal);
        Assert.Contains(
            RuntimeEvents.Entrypoints[0], refused.Message, StringComparison.Ordinal);
        Assert.Throws<InvalidOperationException>(() => Rules.Design);
        Assert.Throws<InvalidOperationException>(() => Rules.Messages);
        Assert.Throws<InvalidOperationException>(() => Rules.Instances);
    }

    [Fact]
    public void TheRemoteBuildListsNoReportsAndSaysSo()
    {
        // The Reports are xmip-core-report's, which reads a node's
        // configuration: a node's build only (ADR-0072, amendment 2026-11-10).
        InvalidOperationException refused =
            Assert.Throws<InvalidOperationException>(() => Rules.Reports);

        Assert.StartsWith(RemoteBuild.Sentence, refused.Message, StringComparison.Ordinal);
        Assert.Contains(OperateAbi.ReportEntrypoint, refused.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void TheRemoteBuildMakesAPersonsRequestAndSignsNone()
    {
        // The phone makes its own key and request (ADR-0034, amendment
        // 2026-10-15); only the authority's node signs.
        using TestFolder folder = new("operate-remote-request");

        Assert.True(Rules.Certificates.Request(
            "alice@example.org", folder.Inside("phone"), out _, out string written, out _));
        string request = File.ReadAllText(written);

        InvalidOperationException refused = Assert.Throws<InvalidOperationException>(
            () => Rules.Certificates.Sign("node.xmip", request, out _, out _));

        Assert.StartsWith(RemoteBuild.Sentence, refused.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void TheRemoteBuildCarriesTheAudit()
    {
        // Every surface, the phone's too, records its own start and acts
        // (ProgramAudit), so section 9 is always there.
        Assert.NotNull(Rules.Audit);
    }

    [Fact]
    public void TheRemoteBuildReadsADocument()
    {
        // The phone reads its own gui.toml by the runtime's one reader
        // (section 27; ADR-0031, amendment 2026-11-17), and probes no module.
        Assert.True(Rules.Documents.TryRead("[Xmip]\nSurface = \"remote\"\n", out string read));
        Assert.Contains("\"remote\"", read, StringComparison.Ordinal);
        Assert.Throws<InvalidOperationException>(() => Rules.Modules);
    }

    [Fact]
    public void ARemoteBuildServesNothingAndSaysSo()
    {
        InvalidOperationException refused = Assert.Throws<InvalidOperationException>(
            () => Rules.Remotes.ServeLocal("xmip-test", [], [], [], "Xmip.Operate.Test", out _));

        Assert.StartsWith(RemoteBuild.Sentence, refused.Message, StringComparison.Ordinal);
    }

    // The estate's remote build, in the working area at its root: the
    // directory holding architecture.toml, found by the one rule every .NET
    // host and test finds the estate by (TomlDocument.BasePath).
    private static string Built()
    {
        return Path.Combine(
            TomlDocument.BasePath(AppContext.BaseDirectory),
            ".ai-interaction", "target-remote", "debug", NativeRuntime.FileName);
    }
}
