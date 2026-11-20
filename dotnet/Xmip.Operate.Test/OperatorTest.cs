namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="Operator.Load"/> answers null with a reason rather than throwing.
/// A surface that cannot reach a node shows that reason; an exception would
/// show a stack.
/// </summary>
public sealed class OperatorTest
{
    [Fact]
    public void LoadingAMissingPathAnswersNullWithTheReason()
    {
        using TestFolder folder = new("operate-missing");
        string missing = folder.Inside("no-such-xmip-runtime.dll");

        Operator? loaded = Operator.Load(missing, out string reason);

        Assert.Null(loaded);
        Assert.Contains(missing, reason, StringComparison.Ordinal);
    }

    [Fact]
    public void LoadingAFileThatIsNotALibraryAnswersNullWithTheReason()
    {
        using TestFolder folder = new("operate-bogus");
        string bogus = folder.Inside("bogus.dll");

        File.WriteAllText(bogus, "not a library");

        Operator? loaded = Operator.Load(bogus, out string reason);

        Assert.Null(loaded);
        Assert.Contains("could not be loaded", reason, StringComparison.Ordinal);
    }
}
