using System.Text.Json;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="RuntimeWords"/> crosses section 28.3 of <c>xmip_operate.h</c>
/// to the runtime this estate built. The words, the sentences and the order
/// are tested in Rust, where they are written (<c>operate::surface</c>, abi's
/// <c>status_meaning</c> and <c>boundaries</c>); these prove the crossing:
/// each answer whole and in its shape, and a refusal carried as the
/// runtime's sentence.
/// </summary>
public sealed class RuntimeWordsTest
{
    private static readonly TestCluster Cluster = TestCluster.Read();

    private static RuntimeWords Words => RuntimeRulesTest.Rules.Sentences;

    [Fact]
    public void AKnownCodeCarriesTheHeadersNameAndItsEnglish()
    {
        StatusMeaning timeout = Words.Meaning((int)XmipStatus.Timeout);

        Assert.True(timeout.Known);
        Assert.Equal("Timeout", timeout.Name);
        Assert.NotEmpty(timeout.Meaning);
        Assert.Equal(timeout.Meaning, RuntimeWords.Explain((int)XmipStatus.Timeout));
        Assert.True(timeout.Retryable);
        Assert.False(timeout.Terminal);
    }

    [Fact]
    public void ATerminalCodeIsNeverRetryable()
    {
        StatusMeaning panic = Words.Meaning((int)XmipStatus.Panic);

        Assert.True(panic.Terminal);
        Assert.False(panic.Retryable);
    }

    [Fact]
    public void ACodeTheHeaderDoesNotDefineIsUnknownInLowerCaseAndNothingElse()
    {
        StatusMeaning stranger = Words.Meaning(4711);

        Assert.False(stranger.Known);
        Assert.Equal(StatusMeaning.Unknown, stranger.Name);
        Assert.Equal(4711, stranger.Code);
        Assert.False(stranger.Retryable);
        Assert.False(stranger.Terminal);
        Assert.Equal("not a status this build knows", stranger.Meaning);
    }

    [Fact]
    public void BothBoundariesAreTheHeadersConstants()
    {
        AbiBoundaries current = Words.Boundaries();
        string suffix = OperatingSystem.IsWindows() ? ".dll"
            : OperatingSystem.IsMacOS() ? ".dylib" : ".so";

        Assert.Equal(ModuleAbi.AbiVersion, current.ModuleVersion);
        Assert.Equal(ModuleAbi.Entrypoint, current.ModuleEntrypoint);
        Assert.EndsWith(suffix, current.ModuleLibraryFileName, StringComparison.Ordinal);
        Assert.Equal(OperateAbi.Version, current.OperateVersion);
        Assert.Equal(OperateAbi.Entrypoint, current.OperateEntrypoint);
    }

    [Fact]
    public void TheFixedWordsAreReadOnceAndSayTheOutcomes()
    {
        Assert.Equal("taken", Words.OutcomeWord(OperateAbi.Outcome.Taken));
        Assert.Equal("asked", Words.OutcomeWord(OperateAbi.Outcome.Asked));
        Assert.Equal("refused", Words.OutcomeWord(OperateAbi.Outcome.Refused));
        Assert.Equal("pause", Words.ActWord(OperateAbi.Act.Pause));
        Assert.Equal(
            (OperateAbi.Outcome.Asked, "asked"), Words.ActOutcome((int)XmipStatus.Again));
        Assert.Equal("subscription", Words.Columns(OperateAbi.List.Subscriptions)[0]);
        Assert.Equal("received", Words.Columns(OperateAbi.List.DeadMessages)[0]);
    }

    [Fact]
    public void AListIsAnsweredByPlaceOrRefusedInWords()
    {
        string list = $$"""
            {"subscriptions":[
              {"node":"{{Cluster.NodeScope(1)}}","name":"Orders"},
              {"node":"{{Cluster.NodeScope(0)}}","name":"billing"},
              {"node":"{{Cluster.NodeScope(0)}}","name":"Audit"}]}
            """;

        using JsonDocument ordinal = Words.ListQuery(OperateAbi.List.Subscriptions, list, "{}");
        int[] places =
        [
            .. ordinal.RootElement.GetProperty("places").EnumerateArray()
                .Select(at => at.GetInt32()),
        ];
        ArgumentException refused = Assert.Throws<ArgumentException>(
            () => Words.ListQuery(OperateAbi.List.Subscriptions, list, """{"sort":"colour"}"""));

        Assert.Equal([2, 0, 1], places);
        Assert.StartsWith(
            "REFUSED: 'colour' is no column", refused.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void ARowsWordsAndALongSentenceCrossWhole()
    {
        string longer = new('x', 1000);

        string node = Cluster.NodeScope(0);

        Assert.Equal(
            $"{node}/subscription/Orders",
            Words.ListWord(OperateAbi.ListWord.SubscriptionScope, node, "Orders"));
        Assert.Equal("party", Words.ListWord(OperateAbi.ListWord.EventWho, string.Empty, "party"));
        Assert.Contains(
            longer, Words.ActSaid(OperateAbi.Outcome.Refused, longer), StringComparison.Ordinal);
        Assert.Equal("someone", Words.Who("someone", "user"));
        Assert.Equal("user", Words.Who(null, "user"));
    }
}
