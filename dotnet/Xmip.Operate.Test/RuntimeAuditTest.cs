using Xmip.Abi.Operate;

namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="RuntimeAudit"/> crosses section 9 of <c>xmip_operate.h</c> both
/// ways to the runtime this estate built. The record, its reader and its query
/// are tested in Rust, where they are written (<c>xmip-core-audit</c>); these
/// prove the crossing: a record made from .NET is read back whole, and a
/// refusal is carried as the capability's sentence.
/// </summary>
public sealed class RuntimeAuditTest
{
    [Fact]
    public void ARecordMadeFromDotNetIsReadBackWhole()
    {
        using TestFolder directory = new("operate-audit-read");
        RuntimeAudit audit = RuntimeRulesTest.Rules.Audit;

        audit.Record(
            "xmip-operate-test", directory.Path, "probe", AuditPhase.Failure, AuditSeverity.Error,
            "a failure", new Dictionary<string, string> { ["why"] = "a test" });

        AuditRead read = audit.Read(
            directory.Path, [new("severity", "Error"), new("sort", "program")]);

        Assert.Equal(1, read.Read);
        AuditEntry entry = Assert.Single(read.Records);
        Assert.Equal("xmip-operate-test", entry.Program);
        Assert.Equal("Failure", entry.Phase);
        Assert.Equal("a test", entry.Properties["why"]);
        Assert.Null(entry.Location);
        Assert.Equal("a failure", entry.Summary);
        Assert.Equal("host", Assert.Single(read.Groups).Kind);
        Assert.Equal(["probe"], read.Actions);
        Assert.Equal("at", read.Columns[0]);
        Assert.Equal(["Information", "Warning", "Error"], read.Severities);
    }

    [Fact]
    public void AQueryTheCapabilityDoesNotTakeIsRefusedInItsWords()
    {
        ArgumentException refused = Assert.Throws<ArgumentException>(() =>
            RuntimeRulesTest.Rules.Audit.Read(null, [new("color", "red")]));

        Assert.StartsWith("REFUSED", refused.Message, StringComparison.Ordinal);
    }
}
