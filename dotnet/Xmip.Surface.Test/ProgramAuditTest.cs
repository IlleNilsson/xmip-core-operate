using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface.Test;

/// <summary>
/// A .NET program audits through the capability (ADR-0062): into the
/// directory it was told and, when the capability cannot be reached at all,
/// a refusal that says why. No test here writes to the operating system's
/// log.
/// </summary>
public sealed class ProgramAuditTest
{
    [Fact]
    public void AValueIsSaidOneWayWhicheverProgramRecordsIt()
    {
        // Until 2026-09-27 the cmdlets said a table as its type's name and the
        // script module as its pairs.
        System.Collections.Hashtable values = new()
        {
            ["table"] = new System.Collections.Specialized.OrderedDictionary { ["a"] = 1 },
            ["list"] = new[] { "x", "y" },
            ["flag"] = true,
            ["none"] = null,
        };

        Dictionary<string, string> said = ProgramAudit.Properties(values);

        Assert.Equal("a=1", said["table"]);
        Assert.Equal("x, y", said["list"]);
        Assert.Equal("yes", said["flag"]);
        Assert.Equal(string.Empty, said["none"]);
        Assert.Empty(ProgramAudit.Properties(null));
    }

    [Fact]
    public void AnActLandsInTheDirectoryTheProgramWasTold()
    {
        using TestFolder directory = new("surface-audit");
        ProgramAudit audit = new("Xmip.Surface.Test", directory.Path);

        AuditOutcome outcome = audit.Record(
            "start", AuditPhase.Begin, AuditSeverity.Information, "started",
            new Dictionary<string, string> { ["url"] = "http://127.0.0.1:5087" });

        Assert.Equal(new AuditOutcome.Kept(AuditKept.Persisted, string.Empty), outcome.Value);
        string text = File.ReadAllText(directory.Inside("audit.toml"));
        Assert.Contains("program = \"Xmip.Surface.Test\"", text, StringComparison.Ordinal);
        Assert.Contains("action = \"start\"", text, StringComparison.Ordinal);
    }

    [Fact]
    public void AFailureCarriesTheExceptionTypeAndItsDetailOnce()
    {
        using TestFolder directory = new("surface-audit");
        ProgramAudit audit = new("Xmip.Surface.Test", directory.Path);

        InvalidOperationException boom = new("boom");
        AuditOutcome? outcome = audit.Failed("unhandled", boom);

        Assert.Equal(new AuditOutcome.Kept(AuditKept.Persisted, string.Empty), outcome?.Value);
        Assert.Null(audit.Failed("host", boom));
        string text = File.ReadAllText(directory.Inside("audit.toml"));
        Assert.Contains("phase = \"Failure\"", text, StringComparison.Ordinal);
        Assert.Contains("severity = \"Error\"", text, StringComparison.Ordinal);
        Assert.Contains("message = \"boom\"", text, StringComparison.Ordinal);
        Assert.Contains(
            "\"exception\" = \"System.InvalidOperationException\"", text, StringComparison.Ordinal);
        Assert.Single(text.Split("[[record]]", StringSplitOptions.RemoveEmptyEntries));
    }

    [Fact]
    public void WhatAProgramRecordedItReadsBackByTheWordsOfTheQuery()
    {
        using TestFolder directory = new("surface-audit");
        ProgramAudit audit = new("Xmip.Surface.Test", directory.Path);
        audit.Record("start", AuditPhase.Begin, AuditSeverity.Information);
        audit.Record("stop", AuditPhase.Finished, AuditSeverity.Warning, "stopped");

        AuditQuery query = new() { Severity = "Warning", Sort = "action", Order = "ascending" };
        AuditRead read = audit.Read(query);

        Assert.False(query.Descending);
        Assert.Equal(
            [new("severity", "Warning"), new("sort", "action"), new("order", "ascending")],
            query.Pairs());
        Assert.Equal(2, read.Read);
        Assert.Equal("stopped", Assert.Single(read.Records).Summary);
        Assert.Equal(["stop"], read.Actions);
        Assert.EndsWith("audit.toml", read.File, StringComparison.Ordinal);
        Assert.Empty(read.Chains);

        AuditChain chain = Assert.Single(audit.Read(query with { Verify = true }).Chains);
        Assert.Equal("Xmip.Surface.Test", chain.Writer);
        Assert.Equal(2, chain.Records);
        Assert.True(chain.Whole, chain.Said);
        Assert.StartsWith("OK: ", chain.Said, StringComparison.Ordinal);
        Assert.Empty(read.Fragments);
        Assert.Empty(audit.Read(query with { Verify = true }).Fragments);

        // A write left unfinished is said apart from the chain; the next
        // append seals it, and the record after it continues the chain.
        File.AppendAllText(read.File, "[[record]]\naudit_id = \"0");
        AuditRead open = audit.Read(query with { Verify = true });
        AuditFragment end = Assert.Single(open.Fragments);
        Assert.False(end.Sealed, end.Said);
        Assert.StartsWith("UNFINISHED: ", end.Said, StringComparison.Ordinal);
        Assert.True(Assert.Single(open.Chains).Whole);

        audit.Record("stop", AuditPhase.Finished, AuditSeverity.Warning, "again");
        AuditRead sealedRead = audit.Read(query with { Verify = true });
        AuditFragment sealedFragment = Assert.Single(sealedRead.Fragments);
        Assert.True(sealedFragment.Sealed, sealedFragment.Said);
        Assert.StartsWith("FAILED: ", sealedFragment.Said, StringComparison.Ordinal);
        AuditChain after = Assert.Single(sealedRead.Chains);
        Assert.Equal(3, after.Records);
        Assert.True(after.Whole, after.Said);
    }

    [Fact]
    public void AProgramThatCannotReachAuditIsRefusedAndSaysWhyFirst()
    {
        // No .NET writer of the operating system's log: the capability's
        // fallback is reached through the runtime's library alone, and is
        // tested in Rust (ADR-0062, amendment 2026-11-16).
        AuditOutcome.Refused refused = ProgramAudit.Unreached(
            "Xmip.Surface.Test", "probe", "no runtime library at a path this test names");

        Assert.Equal(
            "Xmip audit could not be reached (no runtime library at a path this test names), " +
            "so Xmip.Surface.Test's record of probe is kept nowhere.",
            refused.Said);
    }
}
